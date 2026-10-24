//! Transcription en direct pendant l'enregistrement.
//! Un `whisper-server` (modèle chargé une seule fois) est lancé dès l'ouverture du panneau d'enregistrement (`live_prepare`) :
//! l'enregistrement ne démarre qu'une fois Whisper chargé. Pendant l'enregistrement, il reçoit toutes les ~0,6 s la fin de
//! l'audio non encore validé ; les phrases terminées sont validées (événement `live-segment`), la dernière reste provisoire
//! (`live-partial`). Chaque phrase validée peut être traduite par l'IA locale (`live-translation`). À l'arrêt, le serveur est
//! libéré, puis la transcription complète du fichier est refaite par le circuit normal (meilleure qualité).

use crate::recorder::TapBuf;
use crate::{engine_paths, AppState, Settings, R};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, State};

const RATE: usize = 16_000;
/// Fenêtre maximale envoyée à Whisper (il travaille par blocs de 30 s).
const MAX_WINDOW: usize = 28 * RATE;

/// Ce qu'il faut pour lancer le serveur et la boucle (vérifié avant de charger Whisper).
pub struct LiveCfg {
    server: PathBuf,
    /// runtime ROCm à mettre dans le PATH quand le serveur est celui de ROCm
    rocm_path: Vec<PathBuf>,
    hw: String,
    model: PathBuf,
    vad: Option<PathBuf>,
    lang: String,
    terms: String,
    settings: Settings,
}

type Emit = dyn Fn(&str, Value) + Send + Sync;

// ─── Serveur Whisper partagé : chargé à l'ouverture du panneau, libéré à l'arrêt ou à l'annulation ───

struct Running {
    child: Child,
    port: u16,
    hw: String,
    model: String,
}
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

static SRV: Mutex<Option<Running>> = Mutex::new(None);
static LOADING: AtomicBool = AtomicBool::new(false);
/// Incrémenté à chaque libération : un chargement en cours dont le numéro a changé s'abandonne.
static GEN: AtomicU64 = AtomicU64::new(0);

/// Libère Whisper (mémoire vidéo comprise) et abandonne un chargement éventuel.
pub fn release() {
    GEN.fetch_add(1, Ordering::SeqCst);
    let r = SRV.lock().unwrap().take();
    drop(r);
}

#[tauri::command]
pub fn live_release() {
    release();
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

pub fn prepare(st: &AppState) -> R<LiveCfg> {
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
    Ok(LiveCfg { server, rocm_path, hw, model, vad, lang: cfg.whisper_lang.clone(), terms, settings: cfg })
}

fn emit_state(app: &AppHandle, st: &str, msg: &str, extra: Value) {
    let mut v = json!({ "state": st, "message": msg });
    if let (Some(o), Some(e)) = (v.as_object_mut(), extra.as_object()) {
        for (k, x) in e {
            o.insert(k.clone(), x.clone());
        }
    }
    let _ = app.emit("live-state", v);
}

/// Lance whisper-server et attend qu'il réponde. S'interrompt si `GEN` change (libération demandée).
fn spawn_server(cfg: &LiveCfg, gen: u64) -> R<Running> {
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
        .arg("-m")
        .arg(&cfg.model)
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
    let child = cmd.spawn().map_err(|e| format!("Lancement de whisper-server impossible : {e}"))?;
    let model = cfg.model.file_stem().map(|s| s.to_string_lossy().replace("ggml-", "")).unwrap_or_default();
    let mut srv = Running { child, port, hw: cfg.hw.clone(), model };
    let t0 = Instant::now();
    loop {
        if GEN.load(Ordering::SeqCst) != gen {
            return Err("Chargement annulé.".into());
        }
        if let Ok(Some(s)) = srv.child.try_wait() {
            return Err(format!("whisper-server s'est arrêté au démarrage ({s})."));
        }
        match ureq::get(&format!("http://127.0.0.1:{port}/")).timeout(Duration::from_secs(2)).call() {
            Ok(_) | Err(ureq::Error::Status(..)) => return Ok(srv),
            Err(_) => {}
        }
        if t0.elapsed() > Duration::from_secs(120) {
            return Err("Délai dépassé au chargement de Whisper.".into());
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

/// Charge Whisper en arrière-plan (ouverture du panneau d'enregistrement). Les événements `live-state`
/// (`loading`, `ready`, `error`) renseignent l'interface ; l'enregistrement en direct n'est possible qu'une fois `ready`.
#[tauri::command]
pub fn live_prepare(app: AppHandle, st: State<AppState>) -> R<()> {
    if let Some(r) = SRV.lock().unwrap().as_ref() {
        emit_state(&app, "ready", &r.hw, json!({ "model": r.model, "prep": true }));
        return Ok(());
    }
    if LOADING.swap(true, Ordering::SeqCst) {
        return Ok(()); // déjà en cours : les événements suivront
    }
    let cfg = match prepare(&st) {
        Ok(c) => c,
        Err(e) => {
            LOADING.store(false, Ordering::SeqCst);
            return Err(e);
        }
    };
    let gen = GEN.load(Ordering::SeqCst);
    let model = cfg.model.file_stem().map(|s| s.to_string_lossy().replace("ggml-", "")).unwrap_or_default();
    emit_state(&app, "loading", "Chargement de Whisper…", json!({ "model": model, "hw": cfg.hw, "prep": true }));
    std::thread::spawn(move || {
        match spawn_server(&cfg, gen) {
            Ok(r) => {
                if GEN.load(Ordering::SeqCst) != gen {
                    drop(r); // libéré entre-temps
                } else {
                    let (hw, model) = (r.hw.clone(), r.model.clone());
                    *SRV.lock().unwrap() = Some(r);
                    emit_state(&app, "ready", &hw, json!({ "model": model, "prep": true }));
                }
            }
            Err(e) => {
                if GEN.load(Ordering::SeqCst) == gen {
                    emit_state(&app, "error", &e, json!({ "prep": true }));
                }
            }
        }
        LOADING.store(false, Ordering::SeqCst);
    });
    Ok(())
}

/// Port du serveur chargé, ou une erreur claire si Whisper n'est pas encore prêt.
pub fn ensure_ready() -> R<u16> {
    SRV.lock()
        .unwrap()
        .as_ref()
        .map(|r| r.port)
        .ok_or_else(|| "Whisper n'est pas encore chargé : attendez la fin du chargement avant de démarrer.".to_string())
}

// ─── Boucle de transcription en direct ───

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
        release(); // libère Whisper et sa mémoire avant la transcription finale
    }
}

pub fn start(app: AppHandle, taps: Vec<Arc<Mutex<TapBuf>>>, cfg: LiveCfg, port: u16, translate_to: Option<String>) -> LiveHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let s = stop.clone();
    let join = std::thread::spawn(move || {
        let a = app.clone();
        let emit = move |ev: &str, v: Value| {
            let _ = a.emit(ev, v);
        };
        // traduction en direct : un thread dédié garde le modèle de traduction chargé
        let (tx, rx) = mpsc::channel::<(usize, String)>();
        let have_tr = translate_to.is_some();
        let tr_app = app.clone();
        let settings = crate::llm::translate_cfg(&cfg.settings);
        let src = if cfg.lang == "auto" { String::new() } else { cfg.lang.clone() };
        let tr_join = translate_to.map(|dst| {
            std::thread::spawn(move || {
                let sel = crate::translate::model_for(&settings);
                let gemma = crate::llm::is_translator(&sel);
                let res = crate::llm::with_backend(&tr_app, &settings, &sel, &|m| emit_state(&tr_app, "translate-loading", m, json!({})), &|be| {
                    emit_state(&tr_app, "translate-ready", "", json!({}));
                    while let Ok((i, text)) = rx.recv() {
                        if let Ok(v) = crate::translate::translate_lines(be, gemma, &src, &dst, &[text]) {
                            let _ = tr_app.emit("live-translation", json!({ "i": i, "text": v.first().cloned().unwrap_or_default() }));
                        }
                    }
                    Ok(())
                });
                if let Err(e) = res {
                    emit_state(&tr_app, "translate-error", &format!("Traduction en direct indisponible : {e}"), json!({}));
                }
            })
        });
        emit_state(&app, "recording", &cfg.hw, json!({}));
        run_loop(&emit, &taps, port, &cfg.lang, &cfg.terms, if have_tr { Some(&tx) } else { None }, &s);
        drop(tx); // le thread de traduction se termine seul après les phrases en attente
        drop(tr_join);
    });
    LiveHandle { stop, join: Some(join) }
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

fn run_loop(emit: &Emit, taps: &[Arc<Mutex<TapBuf>>], port: u16, lang: &str, terms: &str, tx: Option<&mpsc::Sender<(usize, String)>>, stop: &AtomicBool) {
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
                emit("live-partial", json!({ "text": "" }));
            }
            continue;
        }
        let prompt = {
            let mut p = String::new();
            if !terms.is_empty() {
                p.push_str(&format!("Termes : {terms}. "));
            }
            let tail: String = recent.chars().rev().take(200).collect::<Vec<_>>().into_iter().rev().collect();
            p.push_str(&tail);
            p
        };
        let segs = match infer(port, &pcm, lang, &prompt) {
            Ok(s) => s,
            Err(e) => {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                emit("live-state", json!({ "state": "warn", "message": e }));
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
                emit("live-segment", json!({ "i": counter, "start": t_off + s, "end": t_off + e, "text": text }));
                if let Some(tx) = tx {
                    let _ = tx.send((counter, text));
                }
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
            emit("live-partial", json!({ "text": p }));
            last_partial = p;
        }
        trim(taps, committed.saturating_sub(RATE));
    }
    // dernière phrase en cours : on la valide telle quelle
    if !last_partial.is_empty() {
        counter += 1;
        let t = (committed as f64) / RATE as f64;
        emit("live-segment", json!({ "i": counter, "start": t, "end": t, "text": last_partial }));
        if let Some(tx) = tx {
            let _ = tx.send((counter, last_partial));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Banc d'essai sur fichier : `VB_TEST_WAV` (WAV mono 16 kHz), `VB_TEST_MODEL` (ggml-*.bin).
    /// Lancer : cargo test --release live_sur_fichier -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_sur_fichier() {
        let wav = std::env::var("VB_TEST_WAV").expect("VB_TEST_WAV");
        let model = std::env::var("VB_TEST_MODEL").expect("VB_TEST_MODEL");
        let b = std::fs::read(&wav).unwrap();
        let mut i = 12;
        while &b[i..i + 4] != b"data" {
            i += 8 + u32::from_le_bytes([b[i + 4], b[i + 5], b[i + 6], b[i + 7]]) as usize;
        }
        let n = u32::from_le_bytes([b[i + 4], b[i + 5], b[i + 6], b[i + 7]]) as usize / 2;
        let pcm: Vec<i16> = b[i + 8..i + 8 + n * 2].chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
        let redist = PathBuf::from(r"I:\project\mirai\verbatim\redist");
        let cfg = LiveCfg {
            server: redist.join("engine").join("whisper-server.exe"),
            rocm_path: vec![],
            hw: "CPU".into(),
            model: PathBuf::from(model),
            vad: Some(redist.join("models").join("ggml-silero-v5.1.2.bin")),
            lang: "fr".into(),
            terms: String::new(),
            settings: Settings::default(),
        };
        let srv = spawn_server(&cfg, GEN.load(Ordering::SeqCst)).expect("serveur");
        let port = srv.port;
        let tap = Arc::new(Mutex::new(TapBuf::default()));
        let taps = vec![tap.clone()];
        let events: Arc<Mutex<Vec<(String, Value)>>> = Arc::new(Mutex::new(Vec::new()));
        let ev = events.clone();
        let emit = move |e: &str, v: Value| ev.lock().unwrap().push((e.to_string(), v));
        let stop = Arc::new(AtomicBool::new(false));
        let (s2, t2) = (stop.clone(), taps.clone());
        let h = std::thread::spawn(move || run_loop(&emit, &t2, port, "fr", "", None, &s2));
        // l'audio arrive en temps réel par blocs de 100 ms, comme depuis le micro
        for chunk in pcm.chunks(1600) {
            tap.lock().unwrap().data.extend_from_slice(chunk);
            std::thread::sleep(Duration::from_millis(100));
        }
        std::thread::sleep(Duration::from_secs(4));
        stop.store(true, Ordering::Relaxed);
        h.join().unwrap();
        drop(srv);
        let ev = events.lock().unwrap();
        let segs: Vec<String> = ev.iter().filter(|(e, _)| e == "live-segment").map(|(_, v)| v["text"].as_str().unwrap_or("").to_string()).collect();
        for (e, v) in ev.iter() {
            println!("{e} {v}");
        }
        assert!(segs.len() >= 3, "trop peu de phrases validées : {}", segs.len());
        assert!(segs.join(" ").to_lowercase().contains("budget"), "texte inattendu : {segs:?}");
    }
}
