//! Enregistrement depuis le PC : micro et/ou son du système (loopback WASAPI).
//! Chaque source est converti à la volée en mono 16 kHz, recalé sur l'horloge réelle
//! (le loopback n'émet rien pendant les silences), puis les sources sont mixées.

use crate::{AppState, R};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde_json::{json, Value};
use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tauri::State;

const RATE: f64 = 16000.0;

/// Horloge active : exclut les durées de pause (les silences du loopback restent comblés).
#[derive(Default)]
pub struct Clock {
    paused_since: Option<Instant>,
    accum: Duration,
}

impl Clock {
    fn active(&self, started: Instant) -> Duration {
        let paused_now = self.paused_since.map(|p| p.elapsed()).unwrap_or_default();
        started.elapsed().saturating_sub(self.accum + paused_now)
    }
}

/// Écriture au fil de l'eau (PCM i16 brut) : rien n'est gardé en mémoire et une
/// coupure brutale laisse un fichier récupérable au prochain lancement.
/// Copie en mémoire des derniers échantillons d'une source, lue par la transcription en direct.
/// `data` couvre les échantillons [base, base + data.len()) de la source.
#[derive(Default)]
pub struct TapBuf {
    pub base: usize,
    pub data: Vec<i16>,
}

pub struct Sink {
    w: BufWriter<File>,
    len: usize,
    unflushed: usize,
    tap: Option<Arc<Mutex<TapBuf>>>,
    pend: Vec<i16>,
}

impl Sink {
    fn new(path: &Path, tap: Option<Arc<Mutex<TapBuf>>>) -> R<Self> {
        let f = File::create(path).map_err(|e| format!("Création du fichier d'enregistrement impossible : {e}"))?;
        Ok(Sink { w: BufWriter::with_capacity(64 * 1024, f), len: 0, unflushed: 0, tap, pend: Vec::new() })
    }
    fn put(&mut self, v: i16) {
        let _ = self.w.write_all(&v.to_le_bytes());
        self.len += 1;
        self.unflushed += 1;
        if self.tap.is_some() {
            self.pend.push(v);
        }
    }
    /// Transmet à la transcription en direct ce qui a été écrit depuis le dernier appel.
    fn publish(&mut self) {
        // jamais d'attente dans le rappel audio : si le lecteur du direct tient le verrou, on réessaie au rappel suivant
        // (attendre ici provoque « A buffer underrun or overrun occurred »)
        if let Some(t) = &self.tap {
            if !self.pend.is_empty() {
                if let Ok(mut g) = t.try_lock() {
                    g.data.extend_from_slice(&self.pend);
                    self.pend.clear();
                }
            }
        }
    }
    fn pad_to(&mut self, target: usize) {
        while self.len < target {
            self.put(0);
        }
    }
    fn flush_if_due(&mut self) {
        if self.unflushed >= 32_000 {
            let _ = self.w.flush();
            self.unflushed = 0;
        }
    }
}

struct Source {
    pcm: Arc<Mutex<Sink>>,
    level: Arc<AtomicU32>, // pic récent 0..1000
    gain: Arc<AtomicU32>,  // sensibilité en millièmes (1000 = ×1), modifiable pendant l'enregistrement
    path: PathBuf,
}

pub struct Recorder {
    stop: mpsc::Sender<()>,
    join: JoinHandle<()>,
    started: Instant,
    clock: Arc<Mutex<Clock>>,
    mic: Option<Source>,
    sys: Option<Source>,
    live: Option<crate::live::LiveHandle>,
}

pub type RecState = Mutex<Option<Recorder>>;

/// Limiteur doux : linéaire jusqu'à 0,8 puis compression progressive (évite l'écrêtage dur quand on amplifie).
fn soft_limit(x: f32) -> f32 {
    let a = x.abs();
    if a <= 0.8 {
        x
    } else {
        x.signum() * (0.8 + 0.2 * ((a - 0.8) / 0.2).tanh())
    }
}

/// Construit un flux d'entrée sur `dev` et alimente `src`.
fn open_stream(dev: &cpal::Device, src: &Source, started: Instant, clock: Arc<Mutex<Clock>>, input: bool) -> R<cpal::Stream> {
    let cfg = if input { dev.default_input_config() } else { dev.default_output_config() }
        .map_err(|e| format!("Configuration audio indisponible : {e}"))?;
    let (rate, ch) = (cfg.sample_rate() as f64, cfg.channels().max(1) as usize);
    let pcm = src.pcm.clone();
    let level = src.level.clone();
    let gain = src.gain.clone();
    // rééchantillonnage de qualité vers 16 kHz (filtre passe-bas, sans décalage) ; tampons réutilisés d'un rappel à l'autre
    let mut rs = crate::resample::Stream::new(cfg.sample_rate(), RATE as u32);
    let mut mono_buf: Vec<f32> = Vec::new();
    let mut res_buf: Vec<f32> = Vec::new();
    let mut first = true;
    let stream = dev
        .build_input_stream::<f32, _, _>(
            cfg.config(),
            move |data: &[f32], _| {
                let active = {
                    let c = clock.lock().unwrap();
                    if c.paused_since.is_some() {
                        return; // en pause : on jette l'audio
                    }
                    c.active(started)
                };
                let mut out = pcm.lock().unwrap();
                // recalage : comble par du silence l'écart avec l'horloge active
                let expected = (active.as_secs_f64() * RATE) as usize;
                let incoming = (data.len() / ch) as f64 * RATE / rate;
                if first || expected > out.len + incoming as usize + 1600 {
                    let target = expected.saturating_sub(incoming as usize);
                    out.pad_to(target);
                    first = false;
                }
                let mut peak = 0f32;
                let g = gain.load(Ordering::Relaxed) as f32 / 1000.0;
                mono_buf.clear();
                mono_buf.extend(data.chunks(ch).map(|fr| fr.iter().sum::<f32>() / ch as f32));
                res_buf.clear();
                rs.push(&mono_buf, &mut res_buf);
                for s in res_buf.iter() {
                    let v = soft_limit(*s * g);
                    peak = peak.max(v.abs());
                    out.put((v * 32767.0) as i16);
                }
                out.flush_if_due();
                out.publish();
                level.store((peak.min(1.0) * 1000.0) as u32, Ordering::Relaxed);
            },
            |e| eprintln!("erreur flux audio : {e}"),
            None,
        )
        .map_err(|e| format!("Impossible d'ouvrir l'appareil audio : {e}"))?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

#[tauri::command]
pub fn rec_start(app: tauri::AppHandle, st: State<AppState>, mic: bool, system: bool, mic_device: Option<String>, sys_device: Option<String>, mic_gain: Option<f32>, sys_gain: Option<f32>, live: Option<bool>, translate_to: Option<String>) -> R<()> {
    if !mic && !system {
        return Err("Choisissez au moins une source.".into());
    }
    let mut slot = st.rec.lock().unwrap();
    if slot.is_some() {
        return Err("Un enregistrement est déjà en cours.".into());
    }
    let rec_dir = st.dir.join("recordings");
    std::fs::create_dir_all(&rec_dir).map_err(|e| e.to_string())?;
    let rid = uuid::Uuid::new_v4().to_string();
    // transcription en direct : vérifie d'abord que le moteur et un modèle sont disponibles
    // (Whisper doit déjà être chargé : l'interface grise « Démarrer » tant qu'il ne l'est pas)
    let live_cfg = if live.unwrap_or(false) { Some((crate::live::prepare(&st)?, crate::live::ensure_ready()?)) } else { None };
    let mut taps: Vec<Arc<Mutex<TapBuf>>> = Vec::new();
    let to_milli = |g: Option<f32>| (g.unwrap_or(1.0).clamp(0.25, 10.0) * 1000.0) as u32;
    let mk = |on: bool, tag: &str, g: Option<f32>| -> R<Option<Source>> {
        if !on {
            return Ok(None);
        }
        let path = rec_dir.join(format!("{rid}.{tag}.pcm"));
        let tap = live_cfg.as_ref().map(|_| Arc::new(Mutex::new(TapBuf::default())));
        if let Some(tp) = &tap {
            taps.push(tp.clone());
        }
        Ok(Some(Source { pcm: Arc::new(Mutex::new(Sink::new(&path, tap)?)), level: Arc::new(AtomicU32::new(0)), gain: Arc::new(AtomicU32::new(to_milli(g))), path }))
    };
    let mut mk = mk;
    let (m, s) = (mk(mic, "mic", mic_gain)?, mk(system, "sys", sys_gain)?);
    let started = Instant::now();
    let clock = Arc::new(Mutex::new(Clock::default()));
    let clock_t = clock.clone();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (ready_tx, ready_rx) = mpsc::channel::<R<()>>();
    let (mc, sc) = (m.as_ref().map(|x| (x.pcm.clone(), x.level.clone(), x.gain.clone(), x.path.clone())), s.as_ref().map(|x| (x.pcm.clone(), x.level.clone(), x.gain.clone(), x.path.clone())));
    // Les flux WASAPI ne sont pas Send : ils vivent dans ce thread dédié.
    let join = std::thread::spawn(move || {
        let host = cpal::default_host();
        let open = || -> R<Vec<cpal::Stream>> {
            let mut v = Vec::new();
            if let Some((pcm, level, gain, path)) = &mc {
                let d = pick(&host, true, &mic_device).ok_or("Aucun micro détecté.")?;
                v.push(open_stream(&d, &Source { pcm: pcm.clone(), level: level.clone(), gain: gain.clone(), path: path.clone() }, started, clock_t.clone(), true).map_err(|e| {
                    format!("{e} (vérifiez Paramètres Windows › Confidentialité › Microphone › applications de bureau)")
                })?);
            }
            if let Some((pcm, level, gain, path)) = &sc {
                let d = pick(&host, false, &sys_device).ok_or("Aucune sortie audio détectée.")?;
                v.push(open_stream(&d, &Source { pcm: pcm.clone(), level: level.clone(), gain: gain.clone(), path: path.clone() }, started, clock_t.clone(), false)?);
            }
            Ok(v)
        };
        match open() {
            Ok(streams) => {
                let _ = ready_tx.send(Ok(()));
                let _ = stop_rx.recv();
                drop(streams);
            }
            Err(e) => {
                let _ = ready_tx.send(Err(e));
            }
        }
    });
    match ready_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(())) => {
            let live_h = live_cfg.map(|(c, port)| crate::live::start(app, taps, c, port, translate_to.filter(|l| !l.is_empty())));
            *slot = Some(Recorder { stop: stop_tx, join, started, clock, mic: m, sys: s, live: live_h });
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Délai dépassé à l'ouverture de l'appareil audio.".into()),
    }
}

/// Règle la sensibilité d'une source pendant l'enregistrement ("mic" ou "system"), de ×0,25 à ×10.
#[tauri::command]
pub fn rec_set_gain(st: State<AppState>, source: String, gain: f32) -> R<()> {
    let g = st.rec.lock().unwrap();
    let r = g.as_ref().ok_or("Aucun enregistrement en cours.")?;
    let src = if source == "mic" { r.mic.as_ref() } else { r.sys.as_ref() };
    if let Some(s) = src {
        s.gain.store((gain.clamp(0.25, 10.0) * 1000.0) as u32, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn rec_status(st: State<AppState>) -> R<Value> {
    let g = st.rec.lock().unwrap();
    Ok(match g.as_ref() {
        None => json!({ "recording": false }),
        Some(r) => {
            let (seconds, paused) = {
                let c = r.clock.lock().unwrap();
                (c.active(r.started).as_secs(), c.paused_since.is_some())
            };
            json!({
            "recording": true,
            "live": r.live.is_some(),
            "seconds": seconds,
            "paused": paused,
            "mic": r.mic.as_ref().map(|s| s.level.load(Ordering::Relaxed)),
            "mic_gain": r.mic.as_ref().map(|s| s.gain.load(Ordering::Relaxed) as f32 / 1000.0),
            "system_gain": r.sys.as_ref().map(|s| s.gain.load(Ordering::Relaxed) as f32 / 1000.0),
            "system": r.sys.as_ref().map(|s| s.level.load(Ordering::Relaxed)),
            })
        }
    })
}

/// Arrête l'enregistrement, mixe les sources depuis le disque et écrit le WAV.
pub fn finish(st: &AppState, dst: &Path) -> R<()> {
    let mut rec = st.rec.lock().unwrap().take().ok_or("Aucun enregistrement en cours.")?;
    if let Some(l) = rec.live.take() {
        l.stop(); // libère Whisper et le modèle de traduction avant la transcription finale
    }
    let _ = rec.stop.send(());
    let _ = rec.join.join();
    let total = (rec.clock.lock().unwrap().active(rec.started).as_secs_f64() * RATE) as usize;
    for s in [&rec.mic, &rec.sys].into_iter().flatten() {
        let _ = s.pcm.lock().unwrap().w.flush();
    }
    let paths: Vec<PathBuf> = [&rec.mic, &rec.sys].into_iter().flatten().map(|s| s.path.clone()).collect();
    let res = mix_to_wav(&paths, total, dst);
    if res.is_ok() {
        for p in &paths {
            let _ = std::fs::remove_file(p);
        }
    }
    res
}

/// Mixe des fichiers PCM i16 mono 16 kHz (alignés au début) en un WAV, par blocs.
fn mix_to_wav(paths: &[PathBuf], min_samples: usize, dst: &Path) -> R<()> {
    let n = paths
        .iter()
        .map(|p| std::fs::metadata(p).map(|m| m.len() as usize / 2).unwrap_or(0))
        .max()
        .unwrap_or(0)
        .max(min_samples);
    if n < 16000 {
        return Err("Enregistrement trop court (moins d'une seconde).".into());
    }
    let gain = if paths.len() > 1 { 0.9f32 } else { 1.0 };
    let mut files: Vec<File> = paths.iter().map(|p| File::open(p).map_err(|e| e.to_string())).collect::<R<_>>()?;
    let mut out = BufWriter::new(File::create(dst).map_err(|e| e.to_string())?);
    let dl = (n * 2) as u32;
    out.write_all(b"RIFF").map_err(|e| e.to_string())?;
    out.write_all(&(36 + dl).to_le_bytes()).map_err(|e| e.to_string())?;
    out.write_all(b"WAVEfmt ").map_err(|e| e.to_string())?;
    for v in [16u32, 0x0001_0001, 16000, 32000] {
        out.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())?;
    }
    out.write_all(&2u16.to_le_bytes()).map_err(|e| e.to_string())?;
    out.write_all(&16u16.to_le_bytes()).map_err(|e| e.to_string())?;
    out.write_all(b"data").map_err(|e| e.to_string())?;
    out.write_all(&dl.to_le_bytes()).map_err(|e| e.to_string())?;
    const CH: usize = 32_768; // échantillons par bloc
    let mut bufs: Vec<Vec<u8>> = files.iter().map(|_| vec![0u8; CH * 2]).collect();
    let mut done = 0;
    while done < n {
        let take = CH.min(n - done);
        let mut acc = vec![0f32; take];
        for (f, buf) in files.iter_mut().zip(bufs.iter_mut()) {
            let mut got = 0;
            while got < take * 2 {
                match f.read(&mut buf[got..take * 2]) {
                    Ok(0) | Err(_) => break,
                    Ok(k) => got += k,
                }
            }
            for i in 0..got / 2 {
                acc[i] += i16::from_le_bytes([buf[2 * i], buf[2 * i + 1]]) as f32 * gain;
            }
        }
        for v in acc {
            out.write_all(&(v.clamp(-32768.0, 32767.0) as i16).to_le_bytes()).map_err(|e| e.to_string())?;
        }
        done += take;
    }
    out.flush().map_err(|e| e.to_string())
}

/// Au démarrage : reconstitue en WAV les enregistrements interrompus (crash, coupure).
/// Renvoie (id, chemin du WAV, date de dernière écriture).
pub fn recover(dir: &Path) -> Vec<(String, PathBuf, std::time::SystemTime)> {
    let rec_dir = dir.join("recordings");
    let mut groups: std::collections::HashMap<String, Vec<PathBuf>> = Default::default();
    if let Ok(rd) = std::fs::read_dir(&rec_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("pcm") {
                if let Some(id) = p.file_name().and_then(|n| n.to_str()).and_then(|n| n.split('.').next()) {
                    groups.entry(id.to_string()).or_default().push(p);
                }
            }
        }
    }
    let audio = dir.join("audio");
    let _ = std::fs::create_dir_all(&audio);
    let mut out = Vec::new();
    for (id, paths) in groups {
        let when = paths
            .iter()
            .filter_map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
            .max()
            .unwrap_or_else(std::time::SystemTime::now);
        let dst = audio.join(format!("{id}.wav"));
        match mix_to_wav(&paths, 0, &dst) {
            Ok(()) => out.push((id, dst, when)),
            Err(_) => {} // trop court : on laisse de côté
        }
        for p in &paths {
            let _ = std::fs::remove_file(p);
        }
    }
    out
}

fn device_name(d: &cpal::Device) -> String {
    d.description().map(|x| x.name().to_string()).unwrap_or_default()
}

/// Appareil par nom, sinon celui par défaut.
fn pick(host: &cpal::Host, input: bool, name: &Option<String>) -> Option<cpal::Device> {
    if let Some(n) = name.as_ref().filter(|n| !n.is_empty()) {
        let found = if input {
            host.input_devices().ok()?.find(|d| &device_name(d) == n)
        } else {
            host.output_devices().ok()?.find(|d| &device_name(d) == n)
        };
        if found.is_some() {
            return found;
        }
    }
    if input { host.default_input_device() } else { host.default_output_device() }
}

#[tauri::command(async)]
pub fn rec_devices() -> R<Value> {
    let host = cpal::default_host();
    let list = |input: bool| -> Vec<String> {
        let mut v: Vec<String> = if input {
            host.input_devices().map(|i| i.map(|d| device_name(&d)).collect()).unwrap_or_default()
        } else {
            host.output_devices().map(|i| i.map(|d| device_name(&d)).collect()).unwrap_or_default()
        };
        v.retain(|n| !n.is_empty());
        v.dedup();
        v
    };
    let def = |input: bool| -> Option<String> {
        (if input { host.default_input_device() } else { host.default_output_device() }).map(|d| device_name(&d))
    };
    Ok(json!({ "inputs": list(true), "outputs": list(false), "default_input": def(true), "default_output": def(false) }))
}

#[tauri::command]
pub fn rec_pause(st: State<AppState>, paused: bool) -> R<()> {
    let g = st.rec.lock().unwrap();
    let r = g.as_ref().ok_or("Aucun enregistrement en cours.")?;
    let mut c = r.clock.lock().unwrap();
    match (paused, c.paused_since) {
        (true, None) => c.paused_since = Some(Instant::now()),
        (false, Some(since)) => {
            c.accum += since.elapsed();
            c.paused_since = None;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Capture réelle du micro par défaut pendant 3 s : vérifie que la copie destinée au direct se remplit.
    /// Lancer : cargo test --release capture_micro_vers_direct -- --ignored --nocapture
    #[test]
    #[ignore]
    fn capture_micro_vers_direct() {
        let tap = Arc::new(Mutex::new(TapBuf::default()));
        let path = std::env::temp_dir().join("vb-test-mic.pcm");
        let src = Source {
            pcm: Arc::new(Mutex::new(Sink::new(&path, Some(tap.clone())).unwrap())),
            level: Arc::new(AtomicU32::new(0)),
            gain: Arc::new(AtomicU32::new(1000)),
            path: path.clone(),
        };
        let host = cpal::default_host();
        let dev = pick(&host, true, &None).expect("aucun micro");
        let started = Instant::now();
        let clock = Arc::new(Mutex::new(Clock::default()));
        let stream = open_stream(&dev, &src, started, clock, true).expect("flux");
        std::thread::sleep(Duration::from_secs(3));
        drop(stream);
        let g = tap.lock().unwrap();
        let peak = g.data.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
        let written = src.pcm.lock().unwrap().len;
        println!("échantillons du direct : {} (fichier : {}), crête {}", g.data.len(), written, peak);
        assert!(g.data.len() > 16000 * 2, "la copie du direct ne se remplit pas");
        assert!(g.data.len() + 4000 >= written);
    }
}
