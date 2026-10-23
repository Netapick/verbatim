//! Transcription en direct pendant l'enregistrement.
//! Un `whisper-server` (modèle chargé une seule fois) reçoit toutes les secondes la fin de l'audio non encore validé ;
//! les phrases terminées sont validées (événement `live-segment`), la dernière reste provisoire (`live-partial`).
//! Chaque phrase validée peut être traduite par l'IA locale (`live-translation`). À l'arrêt de l'enregistrement,
//! la transcription complète du fichier audio est refaite par le circuit normal (meilleure qualité).

use crate::recorder::TapBuf;
use crate::{engine_paths, AppState, Settings, R};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};

const RATE: usize = 16_000;
/// Fenêtre maximale envoyée à Whisper (il travaille par blocs de 30 s).
const MAX_WINDOW: usize = 28 * RATE;

/// Ce qu'il faut pour démarrer (vérifié avant de lancer l'enregistrement).
pub struct LiveCfg {
    server: PathBuf,
    /// runtime ROCm à mettre dans le PATH quand le serveur est celui de ROCm
    rocm_path: Vec<PathBuf>,
    hw: String,
    model: PathBuf,
    vad: Option<PathBuf>,
    lang: String,
    terms: String,
    translate_to: Option<String>,
    settings: Settings,
}

pub struct LiveHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl LiveHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

/// Modèle utilisé en direct : le modèle choisi s'il est assez léger, sinon le meilleur « petit » modèle installé.
fn live_model(mdir: &PathBuf, chosen: &str, gpu: bool) -> Option<PathBuf> {
    let light = ["tiny", "base", "small"];
    let mut order: Vec<&str> = Vec::new();
    // sur GPU, le modèle choisi tient en temps réel quelle que soit sa taille
    if gpu || light.contains(&chosen) {
        order.push(chosen);
    }
    order.extend(["small", "base", "tiny"]);
    for m in order {
        let p = engine_paths::model_path(mdir, m);
        if p.exists() {
            return Some(p);
        }
    }
    let p = engine_paths::model_path(mdir, chosen);
    p.exists().then_some(p)
}

pub fn prepare(st: &AppState, translate_to: Option<String>) -> R<LiveCfg> {
    let s = st.store.lock().unwrap();
    let cfg = s.settings.clone();
    // GPU AMD : whisper-server compilé pour ROCm, s'il est livré avec le moteur ROCm
    let rocm = if cfg.whisper_device != "cpu" { crate::engine::rocm_server_ready(&st.mdir, &cfg.rocm_root) } else { None };
    let (server, rocm_path, hw) = match rocm {
        Some(dirs) => (engine_paths::engine_rocm_server(&st.mdir), dirs, crate::engine::hw_label("rocm")),
        None => (engine_paths::engine_bin(&st.mdir).with_file_name("whisper-server.exe"), vec![], crate::engine::hw_label("cpu")),
    };
    if !server.exists() {
        return Err("La transcription en direct nécessite whisper-server, absent de ce moteur Whisper. Réinstallez le moteur de transcription (Réglages › Transcription).".into());
    }
    let model = live_model(&st.mdir, &cfg.whisper_model, !rocm_path.is_empty())
        .ok_or("Aucun modèle Whisper installé : installez au moins le modèle « Base » (Réglages › Transcription).")?;
    let vad = Some(crate::engine::vad_path(&st.mdir)).filter(|p| p.exists());
    let terms = s.glossary.iter().map(|t| t.term.clone()).collect::<Vec<_>>().join(", ");
    Ok(LiveCfg { server, rocm_path, hw, model, vad, lang: cfg.whisper_lang.clone(), terms, translate_to, settings: cfg })
}

pub fn start(app: AppHandle, taps: Vec<Arc<Mutex<TapBuf>>>, cfg: LiveCfg) -> LiveHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let s = stop.clone();
    let join = std::thread::spawn(move || {
        if let Err(e) = run(&app, &taps, &cfg, &s) {
            let _ = app.emit("live-state", json!({ "state": "error", "message": e }));
        }
    });
    LiveHandle { stop, join: Some(join) }
}

struct Srv(Child);
impl Drop for Srv {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn state(app: &AppHandle, st: &str, msg: &str) {
    let _ = app.emit("live-state", json!({ "state": st, "message": msg }));
}

/// WAV mono 16 kHz en mémoire.
fn wav_bytes(pcm: &[i16]) -> Vec<u8> {
    let data = (pcm.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + pcm.len() * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&(RATE as u32).to_le_bytes());
    b.extend_from_slice(&((RATE * 2) as u32).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for s in pcm {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

fn infer(port: u16, pcm: &[i16], lang: &str, prompt: &str) -> R<Vec<(f64, f64, String)>> {
    let boundary = format!("----verbatim{}", uuid::Uuid::new_v4().simple());
    let mut body: Vec<u8> = Vec::new();
    for (k, v) in [("response_format", "verbose_json"), ("language", lang), ("temperature", "0.0"), ("prompt", prompt)] {
        body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").as_bytes());
    }
    body.extend_from_slice(
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.wav\"\r\nContent-Type: audio/wav\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(&wav_bytes(pcm));
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let r: Value = ureq::post(&format!("http://127.0.0.1:{port}/inference"))
        .set("Content-Type", &format!("multipart/form-data; boundary={boundary}"))
        .timeout(Duration::from_secs(60))
        .send_bytes(&body)
        .map_err(|e| format!("Whisper en direct : {e}"))?
        .into_json()
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for s in r["segments"].as_array().cloned().unwrap_or_default() {
        let text = s["text"].as_str().unwrap_or("").trim().to_string();
        let low = text.to_lowercase();
        // marqueurs de bruit / silence de Whisper
        if text.is_empty() || (text.starts_with('[') && text.ends_with(']')) || (text.starts_with('(') && text.ends_with(')')) || low.contains("blank_audio") {
            continue;
        }
        out.push((s["start"].as_f64().unwrap_or(0.0), s["end"].as_f64().unwrap_or(0.0), text));
    }
    Ok(out)
}

/// Mélange les sources sur [a, b) (même échelle de temps pour toutes) : somme avec léger gain si deux sources, comme le fichier final.
fn mix(taps: &[Arc<Mutex<TapBuf>>], a: usize, b: usize) -> Vec<i16> {
    let mut acc = vec![0i32; b - a];
    for t in taps {
        // copie rapide sous le verrou, le mixage se fait ensuite sans le tenir
        let (base, slice): (usize, Vec<i16>) = {
            let g = t.lock().unwrap();
            let from = a.max(g.base);
            let to = b.min(g.base + g.data.len());
            if from >= to {
                continue;
            }
            (from, g.data[from - g.base..to - g.base].to_vec())
        };
        for (i, v) in slice.iter().enumerate() {
            acc[base - a + i] += *v as i32;
        }
    }
    let gain = if taps.len() > 1 { 0.9 } else { 1.0 };
    acc.into_iter().map(|v| ((v as f32 * gain).clamp(-32768.0, 32767.0)) as i16).collect()
}

fn available(taps: &[Arc<Mutex<TapBuf>>]) -> usize {
    taps.iter().map(|t| { let g = t.lock().unwrap(); g.base + g.data.len() }).min().unwrap_or(0)
}

fn trim(taps: &[Arc<Mutex<TapBuf>>], before: usize) {
    for t in taps {
        let mut g = t.lock().unwrap();
        if before > g.base {
            let k = (before - g.base).min(g.data.len());
            g.data.drain(..k);
            g.base += k;
        }
    }
}

fn run(app: &AppHandle, taps: &[Arc<Mutex<TapBuf>>], cfg: &LiveCfg, stop: &Arc<AtomicBool>) -> R<()> {
    state(app, "loading", "Chargement de Whisper…");
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        l.local_addr().map_err(|e| e.to_string())?.port()
    };
    let mut cmd = Command::new(&cfg.server);
    if !cfg.rocm_path.is_empty() {
        let mut p: Vec<std::ffi::OsString> = cfg.rocm_path.iter().map(|d| d.as_os_str().to_owned()).collect();
        p.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).map(|x| x.into_os_string()));
        if let Ok(j) = std::env::join_paths(p) {
            cmd.env("PATH", j);
        }
    }
    cmd.current_dir(cfg.server.parent().unwrap_or(&PathBuf::from(".")))
        .arg("-m").arg(&cfg.model)
        .args(["--host", "127.0.0.1", "--port", &port.to_string(), "-l", &cfg.lang, "-t", "4"]);
    if let Some(v) = &cfg.vad {
        cmd.arg("--vad").arg("-vm").arg(v);
    }
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut srv = Srv(cmd.spawn().map_err(|e| format!("Lancement de whisper-server impossible : {e}"))?);
    let t0 = Instant::now();
    loop {
        if stop.load(Ordering::Relaxed) {
            return Ok(());
        }
        if let Ok(Some(s)) = srv.0.try_wait() {
            return Err(format!("whisper-server s'est arrêté au démarrage ({s})."));
        }
        match ureq::get(&format!("http://127.0.0.1:{port}/")).timeout(Duration::from_secs(2)).call() {
            Ok(_) | Err(ureq::Error::Status(..)) => break,
            Err(_) => {}
        }
        if t0.elapsed() > Duration::from_secs(90) {
            return Err("Délai dépassé au chargement de Whisper.".into());
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    // traduction en direct : un thread dédié garde le modèle de traduction chargé
    let (tx, rx) = mpsc::channel::<(usize, String)>();
    let tr_join = cfg.translate_to.clone().map(|dst| {
        let app = app.clone();
        let settings = crate::llm::translate_cfg(&cfg.settings);
        let src = if cfg.lang == "auto" { String::new() } else { cfg.lang.clone() };
        std::thread::spawn(move || {
            let sel = crate::translate::model_for(&settings);
            let gemma = crate::llm::is_translator(&sel);
            let res = crate::llm::with_backend(&app, &settings, &sel, &|m| state(&app, "translate-loading", m), &|be| {
                state(&app, "translate-ready", "");
                while let Ok((i, text)) = rx.recv() {
                    if let Ok(v) = crate::translate::translate_lines(be, gemma, &src, &dst, &[text]) {
                        let _ = app.emit("live-translation", json!({ "i": i, "text": v.first().cloned().unwrap_or_default() }));
                    }
                }
                Ok(())
            });
            if let Err(e) = res {
                state(&app, "translate-error", &format!("Traduction en direct indisponible : {e}"));
            }
        })
    });

    state(app, "ready", &cfg.hw);
    let mut committed = 0usize; // échantillons déjà validés
    let mut counter = 0usize;
    let mut recent = String::new();
    let mut last_partial = String::new();
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(600));
        let avail = available(taps);
        if avail < committed + RATE {
            continue; // moins d'une seconde de nouveau
        }
        let start = committed.max(avail.saturating_sub(MAX_WINDOW));
        let pcm = mix(taps, start, avail);
        let peak = pcm.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
        if peak < 150 {
            // silence : on avance sans interroger Whisper (évite les hallucinations)
            committed = avail.saturating_sub(RATE / 2);
            trim(taps, committed.saturating_sub(RATE));
            if !last_partial.is_empty() {
                last_partial.clear();
                let _ = app.emit("live-partial", json!({ "text": "" }));
            }
            continue;
        }
        let prompt = {
            let mut p = String::new();
            if !cfg.terms.is_empty() {
                p.push_str(&format!("Termes : {}. ", cfg.terms));
            }
            let tail: String = recent.chars().rev().take(200).collect::<Vec<_>>().into_iter().rev().collect();
            p.push_str(&tail);
            p
        };
        let segs = match infer(port, &pcm, &cfg.lang, &prompt) {
            Ok(s) => s,
            Err(e) => {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                state(app, "warn", &e);
                continue;
            }
        };
        let win = (avail - start) as f64 / RATE as f64;
        let t_off = start as f64 / RATE as f64;
        // une phrase est stable quand elle s'est terminée bien avant la fin de l'audio reçu
        let limit = win - if win > 20.0 { 2.0 } else { 1.2 };
        let mut last_end = None;
        let mut partial = Vec::new();
        for (s, e, text) in segs {
            if e <= limit {
                counter += 1;
                last_end = Some(e);
                recent.push(' ');
                recent.push_str(&text);
                let _ = app.emit("live-segment", json!({ "i": counter, "start": t_off + s, "end": t_off + e, "text": text }));
                let _ = tx.send((counter, text));
            } else {
                partial.push(text);
            }
        }
        if let Some(e) = last_end {
            committed = start + (e * RATE as f64) as usize;
        } else if win > 6.0 && partial.is_empty() {
            committed = avail.saturating_sub(RATE); // rien d'audible : on ne garde qu'une seconde de contexte
        }
        let p = partial.join(" ");
        if p != last_partial {
            let _ = app.emit("live-partial", json!({ "text": p }));
            last_partial = p;
        }
        trim(taps, committed.saturating_sub(RATE));
    }
    // dernière phrase en cours : on la valide telle quelle
    if !last_partial.is_empty() {
        counter += 1;
        let t = (committed as f64) / RATE as f64;
        let _ = app.emit("live-segment", json!({ "i": counter, "start": t, "end": t, "text": last_partial }));
        let _ = tx.send((counter, last_partial));
    }
    drop(tx); // le thread de traduction se termine seul après les phrases en attente
    drop(tr_join);
    drop(srv);
    Ok(())
}
