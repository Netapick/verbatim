//! Moteur de transcription : whisper.cpp (binaire officiel épinglé) + décodage audio en Rust.

use crate::{engine_paths::*, AppState, Segment, R};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};
use std::sync::{atomic::{AtomicBool, Ordering}, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

const WHISPER_ZIP: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.2/whisper-bin-x64.zip";
/// Build CUDA (NVIDIA) de whisper.cpp : seul binaire GPU officiel pour Windows.
const WHISPER_GPU_ZIP: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/b5454/whisper-bin-win-cuda-12.4.0-x64.zip";
const WHISPER_GPU_BYTES: u64 = 684_913_404;
const VAD_URL: &str = "https://huggingface.co/ggml-org/whisper-vad/resolve/main/ggml-silero-v5.1.2.bin";
const VAD_BYTES: u64 = 885_098;

/// (nom, libellé, description, taille exacte en octets)
pub const MODELS: [(&str, &str, &str, u64); 8] = [
    ("tiny", "Tiny", "Très rapide, précision limitée", 77_691_713),
    ("base", "Base", "Rapide, précision correcte", 147_951_465),
    ("small", "Small", "Bon équilibre vitesse / précision", 487_601_967),
    ("medium-q5_0", "Medium (compressé)", "Précis, ~3× plus lent que Small", 539_212_467),
    ("large-v3-turbo-q5_0", "Large v3 Turbo (compressé Q5)", "Très précis, un peu moins que les versions Q8 et complète", 574_041_195),
    ("large-v3-turbo-q8_0", "Large v3 Turbo (Q8)", "Quasi identique à la version complète, recommandé pour le français", 874_188_075),
    ("large-v3-turbo", "Large v3 Turbo (complet)", "Qualité maximale du Turbo, plus lourd", 1_624_555_275),
    ("large-v3-q5_0", "Large v3 (compressé)", "Précision maximale, le plus lent", 1_081_140_203),
];

/// État du téléchargement en cours : partagé, donc conservé quand on change de page.
#[derive(Default, Clone)]
struct Dl {
    active: bool,
    model: String,
    phase: String,
    done: u64,
    total: u64,
    error: Option<String>,
}
static DL: Mutex<Dl> = Mutex::new(Dl { active: false, model: String::new(), phase: String::new(), done: 0, total: 0, error: None });

/// Demande d'arrêt du téléchargement en cours (les `.part` sont conservés pour une reprise).
static CANCEL: AtomicBool = AtomicBool::new(false);
const CANCELLED: &str = "Téléchargement annulé.";

#[tauri::command]
pub fn download_cancel() {
    CANCEL.store(true, Ordering::SeqCst);
}

/// Message d'erreur lisible pour une erreur d'écriture ; les refus d'accès (5), violations de partage (32)
/// ou de verrouillage viennent presque toujours d'un antivirus qui analyse ou met en quarantaine le fichier.
pub fn fs_err(e: &std::io::Error, p: &std::path::Path) -> String {
    let blocked = matches!(e.raw_os_error(), Some(5) | Some(32) | Some(33) | Some(1224)) || e.kind() == std::io::ErrorKind::PermissionDenied;
    if !blocked {
        return e.to_string();
    }
    let dir = p.parent().unwrap_or(p).display();
    format!(
        "Accès refusé à « {} » ({e}) : votre antivirus (ou Windows Defender) bloque ou verrouille ce fichier. Dans Sécurité Windows › Protection contre les virus › Historique de protection, restaurez-le, ajoutez le dossier « {dir} » aux exclusions, puis relancez le téléchargement.",
        p.display()
    )
}

/// Extrait une archive à plat dans `out` (ignore les chemins douteux) ; renvoie (fichier, taille écrite).
pub fn extract_flat(zip_path: &std::path::Path, out: &std::path::Path) -> R<Vec<(PathBuf, u64)>> {
    let f = fs::File::open(zip_path).map_err(|e| fs_err(&e, zip_path))?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    fs::create_dir_all(out).map_err(|e| fs_err(&e, out))?;
    let mut files = Vec::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        // aplatit « Release/xxx.dll »
        let name = match f.enclosed_name().and_then(|p| p.file_name().map(|n| n.to_owned())) {
            Some(n) => n,
            None => continue,
        };
        let target = out.join(name);
        let mut dst = fs::File::create(&target).map_err(|e| fs_err(&e, &target))?;
        let n = std::io::copy(&mut f, &mut dst).map_err(|e| fs_err(&e, &target))?;
        files.push((target, n));
    }
    Ok(files)
}

/// Un antivirus supprime souvent les moteurs juste après l'extraction : on attend l'analyse puis on vérifie.
pub fn verify_kept(files: &[(PathBuf, u64)]) -> R<()> {
    std::thread::sleep(std::time::Duration::from_secs(4));
    let gone: Vec<String> = files
        .iter()
        .filter(|(p, n)| fs::metadata(p).map(|m| m.len() != *n).unwrap_or(true))
        .map(|(p, _)| p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default())
        .collect();
    if gone.is_empty() {
        return Ok(());
    }
    let dir = files.first().and_then(|f| f.0.parent()).map(|d| d.display().to_string()).unwrap_or_default();
    Err(format!(
        "Votre antivirus a supprimé ou mis en quarantaine : {}. Dans Sécurité Windows › Protection contre les virus › Historique de protection, restaurez ces fichiers, ajoutez le dossier « {dir} » aux exclusions, puis relancez le téléchargement.",
        gone.join(", ")
    ))
}

fn model_bytes(name: &str) -> Option<u64> {
    MODELS.iter().find(|m| m.0 == name).map(|m| m.3)
}

fn part_len(p: &PathBuf) -> u64 {
    fs::metadata(p.with_extension("part")).map(|m| m.len()).unwrap_or(0)
}

#[tauri::command]
pub fn engine_status(st: State<AppState>) -> R<Value> {
    let selected = st.store.lock().unwrap().settings.whisper_model.clone();
    let models: Vec<Value> = MODELS
        .iter()
        .map(|(n, label, desc, bytes)| {
            let mp = model_path(&st.mdir, n);
            let installed = fs::metadata(&mp).map(|m| m.len() == *bytes).unwrap_or(false);
            json!({ "name": n, "label": label, "desc": desc, "bytes": bytes, "installed": installed,
                    "partial": if installed { 0 } else { part_len(&mp) }, "selected": *n == selected,
                    "path": mp.to_string_lossy() })
        })
        .collect();
    let dl = DL.lock().unwrap().clone();
    let llm_sel = st.store.lock().unwrap().settings.llm_local.clone();
    Ok(json!({
        "llm": crate::llm::status_json(&st.mdir, &llm_sel, &st.store.lock().unwrap().settings),
        "binary": engine_bin(&st.mdir).exists(),
        "binary_path": engine_bin(&st.mdir).to_string_lossy(),
        "gpu": {
            "cuda": cuda_available(),
            "rocm": rocm_ready(&st.mdir, &st.store.lock().unwrap().settings.rocm_root),
            "rocm_path": engine_rocm_bin(&st.mdir).to_string_lossy(),
            "rocm_engine": engine_rocm_bin(&st.mdir).exists(),
            "names": gpu_names(),
            "installed": engine_gpu_bin(&st.mdir).exists(),
            "path": engine_gpu_bin(&st.mdir).to_string_lossy(),
            "bytes": WHISPER_GPU_BYTES,
            "selected": st.store.lock().unwrap().settings.whisper_device != "cpu",
        },
        "vad": vad_path(&st.mdir).exists(),
        "models": models,
        "download": { "active": dl.active, "model": dl.model, "phase": dl.phase, "done": dl.done, "total": dl.total, "error": dl.error },
    }))
}

pub(crate) fn vad_path(dir: &PathBuf) -> PathBuf {
    pick(dir, "models\\ggml-silero-v5.1.2.bin")
}

/// Téléchargement reprenable : le fichier `.part` est conservé en cas d'échec ou de fermeture
/// de l'app et le téléchargement repart de là où il s'était arrêté.
pub fn download_to(url: &str, dst: &PathBuf, expected: u64, phase: &str, model: &str) -> R<()> {
    let part = dst.with_extension("part");
    if let Some(dir) = dst.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut have = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    if have > expected {
        let _ = fs::remove_file(&part);
        have = 0;
    }
    if have < expected {
        let mut req = ureq::get(url);
        if have > 0 {
            req = req.set("Range", &format!("bytes={have}-"));
        }
        let resp = req.call().map_err(|e| format!("Téléchargement impossible : {e}"))?;
        let mut f = if resp.status() == 206 {
            fs::OpenOptions::new().append(true).open(&part).map_err(|e| fs_err(&e, &part))?
        } else {
            have = 0; // le serveur ne gère pas la reprise : on repart de zéro
            fs::File::create(&part).map_err(|e| fs_err(&e, &part))?
        };
        let mut rd = resp.into_reader();
        let (mut buf, mut done, mut last) = (vec![0u8; 128 * 1024], have, 0u64);
        loop {
            if CANCEL.load(Ordering::SeqCst) {
                return Err(CANCELLED.into());
            }
            let n = rd.read(&mut buf).map_err(|e| format!("Connexion interrompue : {e}"))?;
            if n == 0 {
                break;
            }
            f.write_all(&buf[..n]).map_err(|e| fs_err(&e, &part))?;
            done += n as u64;
            if done - last > 256 * 1024 {
                last = done;
                let mut d = DL.lock().unwrap();
                d.phase = phase.into();
                d.model = model.into();
                d.done = done;
                d.total = expected;
            }
        }
        f.flush().map_err(|e| e.to_string())?;
    }
    let got = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    if got != expected {
        return Err(format!("Téléchargement incomplet ({got} / {expected} octets) — relancez pour reprendre."));
    }
    fs::rename(&part, dst).map_err(|e| fs_err(&e, dst))
}

/// Archive du moteur : petite, taille inconnue à l'avance ; téléchargée en une fois.
pub fn download_to_unsized(url: &str, dst: &PathBuf, phase: &str) -> R<()> {
    let resp = ureq::get(url).call().map_err(|e| format!("Téléchargement impossible : {e}"))?;
    let total: u64 = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut rd = resp.into_reader();
    let mut f = fs::File::create(dst).map_err(|e| fs_err(&e, dst))?;
    let (mut buf, mut done) = (vec![0u8; 128 * 1024], 0u64);
    loop {
        if CANCEL.load(Ordering::SeqCst) {
            drop(f);
            let _ = fs::remove_file(dst);
            return Err(CANCELLED.into());
        }
        let n = rd.read(&mut buf).map_err(|e| format!("Connexion interrompue : {e}"))?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n]).map_err(|e| fs_err(&e, dst))?;
        done += n as u64;
        let mut d = DL.lock().unwrap();
        d.phase = phase.into();
        d.done = done;
        d.total = total.max(done);
    }
    Ok(())
}

/// Nom de la carte graphique la plus pertinente pour un type de calcul ("rocm", "cuda", "vulkan").
fn gpu_name_for(kind: &str) -> String {
    let names = gpu_names();
    let has = |n: &String, k: &str| n.to_uppercase().contains(k);
    let found = match kind {
        "cuda" => names.iter().find(|n| has(n, "NVIDIA")),
        "rocm" => names.iter().find(|n| (has(n, "AMD") || has(n, "RADEON")) && !has(n, "GRAPHICS")).or_else(|| names.iter().find(|n| has(n, "AMD") || has(n, "RADEON"))),
        _ => names.iter().find(|n| has(n, "NVIDIA") || ((has(n, "AMD") || has(n, "RADEON")) && !has(n, "GRAPHICS"))).or_else(|| names.first()),
    };
    found.cloned().unwrap_or_else(|| "GPU".into())
}

/// Libellé du matériel de calcul : « GPU · AMD Radeon RX 7800 XT (ROCm) » ou « CPU ».
pub fn hw_label(dev: &str) -> String {
    match dev {
        "rocm" => format!("GPU · {} (ROCm)", gpu_name_for("rocm")),
        "cuda" => format!("GPU · {} (CUDA)", gpu_name_for("cuda")),
        "vulkan" => format!("GPU · {} (Vulkan)", gpu_name_for("vulkan")),
        _ => "CPU".into(),
    }
}

fn set_transcribe_pct(app: &AppHandle, id: &str, pct: u8) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        m.transcribe_pct = pct.min(100);
    }
}

fn set_hw(app: &AppHandle, id: &str, label: &str) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        m.transcribe_hw = label.to_string();
    }
}

/// whisper-server ROCm utilisable (carte AMD, runtime ROCm, exécutable livré) : dossiers du runtime pour le PATH.
pub fn rocm_server_ready(dir: &PathBuf, root: &str) -> Option<Vec<PathBuf>> {
    if !crate::engine_paths::engine_rocm_server(dir).exists() || !rocm_ready(dir, root) {
        return None;
    }
    crate::llm::rocm_path_dirs(dir, root)
}

/// Carte AMD présente et moteur Whisper ROCm + runtime disponibles.
fn rocm_ready(dir: &PathBuf, root: &str) -> bool {
    engine_rocm_bin(dir).exists()
        && crate::llm::rocm_path_dirs(dir, root).is_some()
        && gpu_names().iter().any(|n| { let u = n.to_uppercase(); u.contains("AMD") || u.contains("RADEON") })
}

/// Pilote NVIDIA présent (nvcuda.dll) : condition pour utiliser le build CUDA.
pub fn cuda_available() -> bool {
    std::env::var("SystemRoot")
        .map(|r| PathBuf::from(r).join("System32").join("nvcuda.dll").exists())
        .unwrap_or(false)
}

pub fn gpu_names() -> Vec<String> {
    static NAMES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    NAMES
        .get_or_init(|| {
            let mut c = std::process::Command::new("powershell");
            c.args(["-NoProfile", "-Command", "(Get-CimInstance Win32_VideoController).Name"]);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                c.creation_flags(0x0800_0000);
            }
            c.output()
                .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
                .unwrap_or_default()
        })
        .clone()
}

/// Mémoire vidéo dédiée de la carte la plus dotée, en octets (0 si inconnue). Lue dans le registre
/// (valeur 64 bits) : `AdapterRAM` de WMI est plafonné à 4 Go.
pub fn gpu_vram() -> u64 {
    static V: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *V.get_or_init(|| {
        let script = r#"$k='HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}'; Get-ChildItem $k -ErrorAction SilentlyContinue | ForEach-Object { $p=Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue; $v=$p.'HardwareInformation.qwMemorySize'; if($v){ [string]$v } }"#;
        let mut c = std::process::Command::new("powershell");
        c.args(["-NoProfile", "-Command", script]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x0800_0000);
        }
        c.output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| l.trim().parse::<u64>().ok()).max().unwrap_or(0))
            .unwrap_or(0)
    })
}

/// Télécharge et installe le build GPU (CUDA) de whisper.cpp dans `engine-gpu`.
#[tauri::command]
pub fn engine_install_gpu(app: AppHandle, st: State<AppState>) -> R<()> {
    if !cuda_available() {
        return Err("Aucune carte NVIDIA (pilote CUDA) détectée : le build GPU officiel de whisper.cpp ne fonctionne qu'avec NVIDIA.".into());
    }
    let dir = st.mdir.clone();
    run_job(app, String::new(), "binary-gpu", move || {
        if engine_gpu_bin(&dir).exists() {
            return Ok(());
        }
        let zip_path = dir.join("engine-gpu.zip");
        download_to_unsized(WHISPER_GPU_ZIP, &zip_path, "binary-gpu")?;
        let files = extract_flat(&zip_path, &dir.join("engine-gpu"))?;
        let _ = fs::remove_file(&zip_path);
        verify_kept(&files)?;
        if !engine_gpu_bin(&dir).exists() {
            return Err("whisper-cli.exe absent de l'archive GPU.".into());
        }
        Ok(())
    })
}

#[tauri::command]
pub fn engine_install(app: AppHandle, st: State<AppState>, model: String) -> R<()> {
    if model_bytes(&model).is_none() {
        return Err("Modèle inconnu.".into());
    }
    {
        let mut d = DL.lock().unwrap();
        if d.active {
            return Err("Un téléchargement est déjà en cours.".into());
        }
        CANCEL.store(false, Ordering::SeqCst);
        *d = Dl { active: true, model: model.clone(), phase: "binary".into(), ..Default::default() };
    }
    let dir = st.mdir.clone();
    std::thread::spawn(move || {
        let res = install_all(&dir, &model);
        {
            let mut d = DL.lock().unwrap();
            d.active = false;
            d.error = res.clone().err();
        }
        let _ = app.emit("engine-done", json!({ "ok": res.is_ok(), "error": res.err() }));
        if engine_ready(&dir, &model) {
            let ids: Vec<String> = {
                let s = app.state::<AppState>();
                let g = s.store.lock().unwrap();
                g.meetings.iter().filter(|m| m.status == "needs_engine").map(|m| m.id.clone()).collect()
            };
            for id in ids {
                spawn_transcription(app.clone(), id);
            }
        }
    });
    Ok(())
}

fn install_all(dir: &PathBuf, model: &str) -> R<()> {
    // moteur + VAD, puis modèle (chaque étape reprend là où elle s'est arrêtée)
    let bytes = model_bytes(model).ok_or("Modèle inconnu.")?;
    install_engine_only(dir)?;
    let mp = model_path(dir, model);
    if fs::metadata(&mp).map(|m| m.len() != bytes).unwrap_or(true) {
        download_to(&model_url(model), &mp, bytes, "model", model)?;
    }
    Ok(())
}

fn model_url(model: &str) -> String {
    format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin")
}

fn install_engine_only(dir: &PathBuf) -> R<()> {
    if !engine_bin(dir).exists() {
        let zip_path = dir.join("engine.zip");
        download_to_unsized(WHISPER_ZIP, &zip_path, "binary")?;
        let files = extract_flat(&zip_path, &dir.join("engine"))?;
        let _ = fs::remove_file(&zip_path);
        verify_kept(&files)?;
        if !engine_bin(dir).exists() {
            return Err("whisper-cli.exe absent de l'archive.".into());
        }
    }
    if fs::metadata(vad_path(dir)).map(|m| m.len() != VAD_BYTES).unwrap_or(true) {
        download_to(VAD_URL, &vad_path(dir), VAD_BYTES, "vad", "")?;
    }
    Ok(())
}

/// Met à jour la progression partagée (utilisée aussi par la copie de modèles).
pub fn set_dl(phase: &str, done: u64, total: u64) {
    let mut d = DL.lock().unwrap();
    d.phase = phase.into();
    d.done = done;
    d.total = total;
}

/// Lance une tâche de téléchargement en arrière-plan (un seul à la fois), avec état partagé.
pub fn run_job(app: AppHandle, model: String, phase: &str, job: impl FnOnce() -> R<()> + Send + 'static) -> R<()> {
    {
        let mut d = DL.lock().unwrap();
        if d.active {
            return Err("Un téléchargement est déjà en cours.".into());
        }
        CANCEL.store(false, Ordering::SeqCst);
        *d = Dl { active: true, model, phase: phase.into(), ..Default::default() };
    }
    std::thread::spawn(move || {
        let res = job();
        {
            let mut d = DL.lock().unwrap();
            d.active = false;
            d.error = res.clone().err();
        }
        let _ = app.emit("engine-done", json!({ "ok": res.is_ok(), "error": res.err() }));
    });
    Ok(())
}

#[tauri::command]
pub fn engine_remove(st: State<AppState>, model: String) -> R<()> {
    if model_bytes(&model).is_none() {
        return Err("Modèle inconnu.".into());
    }
    let mp = model_path(&st.mdir, &model);
    let _ = fs::remove_file(&mp);
    let _ = fs::remove_file(mp.with_extension("part"));
    Ok(())
}

fn is_wav(path: &str) -> bool {
    PathBuf::from(path).extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("wav")).unwrap_or(false)
}

/// Copie WAV 16 kHz mono d'un audio importé (`<id>.play.wav`), utilisée pour la lecture.
pub fn play_copy(audio: &str) -> PathBuf {
    PathBuf::from(audio).with_extension("play.wav")
}

/// Fichier à lire dans l'interface : la copie WAV si elle existe, sinon l'audio d'origine.
pub fn play_path(audio: &str) -> String {
    let c = play_copy(audio);
    if !is_wav(audio) && c.exists() { c.to_string_lossy().into() } else { audio.to_string() }
}

/// Décode un audio (mp3, m4a/aac, ogg, flac, wav…) en WAV mono 16 kHz 16 bits.
fn decode_to_wav16k(src: &str, dst: &PathBuf) -> R<()> {
    use symphonia::core::{
        audio::SampleBuffer, codecs::DecoderOptions, errors::Error as SErr, formats::FormatOptions,
        io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
    };
    let file = fs::File::open(src).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = PathBuf::from(src).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("Format audio non reconnu : {e}"))?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("Aucune piste audio.")?.clone();
    let rate = track.codec_params.sample_rate.ok_or("Fréquence d'échantillonnage inconnue.")?;
    let mut dec = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Codec non pris en charge : {e}"))?;
    // deux threads : celui-ci décode, un second rééchantillonne vers 16 kHz au fil de l'eau (même résultat qu'en une passe)
    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(16);
    let worker = std::thread::spawn(move || {
        let mut rs = crate::resample::Stream::new(rate, 16000);
        let (mut n_in, mut out) = (0usize, Vec::<f32>::new());
        for chunk in rx {
            n_in += chunk.len();
            rs.push(&chunk, &mut out);
        }
        if rate != 16000 {
            rs.push(&vec![0.0; rs.flush_len()], &mut out);
            out.resize((n_in as f64 * 16000.0 / rate as f64) as usize, 0.0);
        }
        (n_in, out)
    });
    let mut mono: Vec<f32> = Vec::with_capacity(1 << 16);
    let mut failure: Option<String> = None;
    loop {
        let pkt = match format.next_packet() {
            Ok(p) => p,
            Err(SErr::IoError(_)) | Err(SErr::ResetRequired) => break,
            Err(e) => {
                failure = Some(e.to_string());
                break;
            }
        };
        if pkt.track_id() != track.id {
            continue;
        }
        match dec.decode(&pkt) {
            Ok(buf) => {
                let spec = *buf.spec();
                let ch = spec.channels.count().max(1);
                let mut sb = SampleBuffer::<f32>::new(buf.capacity() as u64, spec);
                sb.copy_interleaved_ref(buf);
                for fr in sb.samples().chunks(ch) {
                    mono.push(fr.iter().sum::<f32>() / ch as f32);
                }
                if mono.len() >= 1 << 16 {
                    let _ = tx.send(std::mem::replace(&mut mono, Vec::with_capacity(1 << 16)));
                }
            }
            Err(SErr::DecodeError(_)) => continue,
            Err(_) => break,
        }
    }
    if !mono.is_empty() {
        let _ = tx.send(mono);
    }
    drop(tx);
    let (n_in, resampled) = worker.join().map_err(|_| "Rééchantillonnage interrompu.".to_string())?;
    if let Some(e) = failure {
        return Err(e);
    }
    if n_in == 0 {
        return Err("Audio vide ou illisible.".into());
    }
    // rééchantillonnage de qualité vers 16 kHz (fait par le second thread) : un filtre passe-bas évite que les aigus se replient dans la voix
    let pcm: Vec<i16> = resampled.into_iter().map(|v| (v.clamp(-1.0, 1.0) * 32767.0) as i16).collect();
    let dl = (pcm.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + pcm.len() * 2);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + dl).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&16000u32.to_le_bytes());
    w.extend_from_slice(&32000u32.to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&dl.to_le_bytes());
    for s in pcm {
        w.extend_from_slice(&s.to_le_bytes());
    }
    fs::write(dst, w).map_err(|e| e.to_string())
}

fn set_status(app: &AppHandle, id: &str, status: &str, error: Option<String>, transcript: Option<Vec<Segment>>) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        m.status = status.into();
        m.error = error;
        if let Some(t) = transcript {
            m.transcript = t;
            m.translations.clear(); // les traductions suivaient l'ancienne transcription
        }
    }
    let _ = st.save(&s);
}

/// Écrit dans `dst` la partie [`from_ms`, `from_ms + len_ms`[ d'un WAV 16 kHz mono 16 bits.
fn write_wav_part(src: &std::path::Path, dst: &std::path::Path, from_ms: u64, len_ms: u64) -> R<()> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let mut f = fs::File::open(src).map_err(|e| e.to_string())?;
    f.seek(SeekFrom::Start(44 + from_ms * 32)).map_err(|e| e.to_string())?;
    let bytes = len_ms * 32;
    let mut out = std::io::BufWriter::new(fs::File::create(dst).map_err(|e| e.to_string())?);
    let mut h = Vec::with_capacity(44);
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&((36 + bytes) as u32).to_le_bytes());
    h.extend_from_slice(b"WAVEfmt ");
    h.extend_from_slice(&16u32.to_le_bytes());
    h.extend_from_slice(&1u16.to_le_bytes());
    h.extend_from_slice(&1u16.to_le_bytes());
    h.extend_from_slice(&16000u32.to_le_bytes());
    h.extend_from_slice(&32000u32.to_le_bytes());
    h.extend_from_slice(&2u16.to_le_bytes());
    h.extend_from_slice(&16u16.to_le_bytes());
    h.extend_from_slice(b"data");
    h.extend_from_slice(&(bytes as u32).to_le_bytes());
    out.write_all(&h).map_err(|e| e.to_string())?;
    std::io::copy(&mut f.take(bytes), &mut out).map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())
}

/// Au-delà de cette durée (1 h), la transcription est découpée en `LONG_CHUNKS` morceaux traités en parallèle.
const LONG_AUDIO_MS: u64 = 3_600_000;
const LONG_CHUNKS: usize = 3;

/// Points de coupe (en ms) en `n` morceaux égaux, décalés vers le moment le plus silencieux (±30 s)
/// pour ne pas couper une phrase. `wav` : WAV 16 kHz mono 16 bits (en-tête de 44 octets).
fn split_points(wav: &std::path::Path, total_ms: u64, n: usize) -> Vec<u64> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = match fs::File::open(wav) { Ok(f) => f, Err(_) => return Vec::new() };
    let mut cuts = Vec::new();
    for k in 1..n as u64 {
        let target = total_ms * k / n as u64;
        let from = target.saturating_sub(30_000);
        let len = (60_000u64).min(total_ms - from);
        let mut buf = vec![0u8; (len * 32) as usize];
        if f.seek(SeekFrom::Start(44 + from * 32)).is_err() || f.read_exact(&mut buf).is_err() {
            cuts.push(target);
            continue;
        }
        // énergie par fenêtre de 200 ms (3 200 échantillons) ; on garde la plus calme
        let best = buf
            .chunks_exact(6400)
            .enumerate()
            .map(|(i, w)| (i, w.chunks_exact(2).map(|b| (i16::from_le_bytes([b[0], b[1]]) as i64).abs()).sum::<i64>()))
            .min_by_key(|&(_, e)| e)
            .map(|(i, _)| i as u64)
            .unwrap_or(0);
        cuts.push(from + best * 200 + 100);
    }
    cuts
}

fn run_whisper(app: &AppHandle, id: &str) -> R<Vec<Segment>> {
    let st = app.state::<AppState>();
    let (audio, cfg, glossary) = {
        let s = st.store.lock().unwrap();
        let m = s.meetings.iter().find(|m| m.id == id).ok_or("Réunion introuvable")?;
        (m.audio_path.clone().ok_or("Pas d'audio")?, s.settings.clone(), s.glossary.clone())
    };
    if !engine_ready(&st.mdir, &cfg.whisper_model) {
        return Err("Moteur Whisper non installé.".into());
    }
    set_status(app, id, "converting", None, None);
    let tmp = st.dir.join(format!("{id}.16k.wav"));
    decode_to_wav16k(&audio, &tmp)?;
    set_status(app, id, "transcribing", None, None);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 8);
    let try_gpu = cfg.whisper_device != "cpu";
    // Au-delà d'une heure, une seule instance n'occupe pas tout le GPU : on découpe aux silences
    // et on transcrit les morceaux en parallèle (CPU : une seule instance, le parallélisme n'y gagne rien).
    let total_ms = fs::metadata(&tmp).map(|m| m.len().saturating_sub(44) / 32).unwrap_or(0);
    let cuts = if total_ms > LONG_AUDIO_MS { split_points(&tmp, total_ms, LONG_CHUNKS) } else { Vec::new() };
    let chunk_json = |i: usize| st.dir.join(format!("{id}.p{i}.json"));
    let chunk_wav = |i: usize| st.dir.join(format!("{id}.p{i}.wav"));
    // `gpu` : moteur ROCm / CUDA s'il est utilisable, sinon CPU
    let run = |gpu: bool| -> R<(bool, String)> {
        // morceaux : (début en ms, durée en ms ; 0 = jusqu'à la fin)
        let parts: Vec<(u64, u64)> = if gpu && !cuts.is_empty() {
            let mut b = vec![0u64];
            b.extend(&cuts);
            b.push(0);
            (0..b.len() - 1).map(|i| (b[i], if b[i + 1] == 0 { 0 } else { b[i + 1] - b[i] })).collect()
        } else {
            vec![(0, 0)]
        };
        let pcts: Vec<std::sync::atomic::AtomicU8> = parts.iter().map(|_| std::sync::atomic::AtomicU8::new(0)).collect();
        for i in 0..LONG_CHUNKS {
            let _ = fs::remove_file(chunk_json(i));
        }
        // plusieurs morceaux : on découpe le WAV nous-mêmes, aux points de silence calculés
        if parts.len() > 1 {
            for (i, &(off, dur)) in parts.iter().enumerate() {
                write_wav_part(&tmp, &chunk_wav(i), off, if dur == 0 { total_ms - off } else { dur })?;
            }
        }
        let use_rocm = gpu && rocm_ready(&st.mdir, &cfg.rocm_root);
        let use_cuda = gpu && !use_rocm && cuda_available() && engine_gpu_bin(&st.mdir).exists();
        set_hw(app, id, &hw_label(if use_rocm { "rocm" } else if use_cuda { "cuda" } else { "cpu" }));
        let build = |i: usize| -> std::process::Command {
            let mut cmd = std::process::Command::new(if use_rocm { engine_rocm_bin(&st.mdir) } else if use_cuda { engine_gpu_bin(&st.mdir) } else { engine_bin(&st.mdir) });
            if use_rocm {
                if let Some(dirs) = crate::llm::rocm_path_dirs(&st.mdir, &cfg.rocm_root) {
                    let mut p: Vec<std::ffi::OsString> = dirs.iter().map(|d| d.as_os_str().to_owned()).collect();
                    p.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).map(|x| x.into_os_string()));
                    if let Ok(j) = std::env::join_paths(p) { cmd.env("PATH", j); }
                }
            }
            let of = st.dir.join(format!("{id}.p{i}"));
            cmd.arg("-m").arg(model_path(&st.mdir, &cfg.whisper_model)).arg("-f").arg(if parts.len() > 1 { chunk_wav(i) } else { tmp.clone() })
                .args(["-l", &cfg.whisper_lang, "-oj", "-np", "-of"]).arg(&of);
            // -bs 1 : décodage glouton, ~30 % plus rapide que -bs 5 pour un texte quasi identique
            cmd.args(["-t", &threads.to_string(), "-bs", "1"]);
            if vad_path(&st.mdir).exists() {
                // ignore les silences : moins d'hallucinations (« (musique) », répétitions)
                cmd.arg("--vad").arg("-vm").arg(vad_path(&st.mdir));
            }
            if !glossary.is_empty() {
                let p: Vec<&str> = glossary.iter().map(|t| t.term.as_str()).collect();
                cmd.arg("--prompt").arg(format!("Termes : {}.", p.join(", ")));
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
            }
            // -pp : whisper.cpp écrit « progress =  42% » sur stderr
            cmd.arg("-pp").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped());
            cmd
        };
        set_transcribe_pct(app, id, 0);
        let mut children = Vec::new();
        for i in 0..parts.len() {
            let child = build(i).spawn().map_err(|e| format!("Lancement de whisper impossible : {e}"));
            match child {
                Ok(c) => {
                    crate::procs::adopt(&c);
                    children.push(c);
                }
                Err(e) => {
                    for mut c in children { let _ = c.kill(); }
                    return Err(e);
                }
            }
        }
        let last = std::sync::Mutex::new(String::new());
        let mut ok = true;
        std::thread::scope(|sc| {
            let hs: Vec<_> = children
                .into_iter()
                .enumerate()
                .map(|(i, mut child)| {
                    let (pcts, last) = (&pcts, &last);
                    sc.spawn(move || {
                        if let Some(err) = child.stderr.take() {
                            for line in std::io::BufRead::lines(std::io::BufReader::new(err)).map_while(Result::ok) {
                                if let Some(p) = line.split("progress =").nth(1) {
                                    if let Ok(n) = p.trim().trim_end_matches('%').trim().parse::<u8>() {
                                        pcts[i].store(n.min(100), std::sync::atomic::Ordering::Relaxed);
                                        let sum: u32 = pcts.iter().map(|a| a.load(std::sync::atomic::Ordering::Relaxed) as u32).sum();
                                        set_transcribe_pct(app, id, (sum / pcts.len() as u32) as u8);
                                    }
                                } else if !line.trim().is_empty() {
                                    *last.lock().unwrap_or_else(|e| e.into_inner()) = line;
                                }
                            }
                        }
                        child.wait().map(|s| s.success()).unwrap_or(false)
                    })
                })
                .collect();
            for h in hs {
                ok &= h.join().unwrap_or(false);
            }
        });
        let last = last.into_inner().unwrap_or_default();
        Ok((ok && (0..parts.len()).all(|i| chunk_json(i).exists()), last))
    };
    let mut out = run(try_gpu);
    // échec du GPU (pilote, mémoire…) : on réessaie sur le CPU (une seule instance)
    if try_gpu && !matches!(&out, Ok((true, _))) {
        out = run(false);
    }
    let (ok, last_err) = out?;
    set_transcribe_pct(app, id, 100);
    // les fichiers JSON présents : un par morceau lancé (1 sur CPU)
    for i in 0..LONG_CHUNKS {
        let _ = fs::remove_file(chunk_wav(i));
    }
    let mut files: Vec<(usize, PathBuf)> = (0..LONG_CHUNKS).map(|i| (i, chunk_json(i))).filter(|(_, p)| p.exists()).collect();
    files.sort();
    // début de chaque morceau (s) : whisper-cli donne des temps relatifs au WAV du morceau
    let starts: Vec<f64> = std::iter::once(0.0).chain(cuts.iter().map(|&c| c as f64 / 1000.0)).collect();
    if !ok || files.is_empty() {
        let _ = fs::remove_file(&tmp);
        for (_, f) in &files { let _ = fs::remove_file(f); }
        return Err(format!("Whisper a échoué : {last_err}"));
    }
    // Lecture : le positionnement d'un mp3 à débit variable est approximatif dans le lecteur
    // de la fenêtre (écarts de plusieurs secondes), alors qu'il est exact sur un WAV. On garde
    // donc le WAV transcrit comme copie de lecture, sur la même ligne de temps que le texte.
    if is_wav(&audio) || fs::rename(&tmp, play_copy(&audio)).is_err() {
        let _ = fs::remove_file(&tmp);
    }
    let mut segs: Vec<Segment> = Vec::new();
    for (ci, f) in &files {
        let shift = if files.len() > 1 { starts.get(*ci).copied().unwrap_or(0.0) } else { 0.0 };
        let raw = fs::read_to_string(f).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(f);
        let v: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        segs.extend(v["transcription"].as_array().cloned().unwrap_or_default().iter().filter_map(|t| {
            let text = t["text"].as_str()?.trim().to_string();
            if text.is_empty() || text == "[BLANK_AUDIO]" {
                return None;
            }
            Some(Segment {
                speaker: "Intervenant".into(),
                start: t["offsets"]["from"].as_f64()? / 1000.0 + shift,
                end: t["offsets"]["to"].as_f64()? / 1000.0 + shift,
                text,
            })
        }));
    }
    // décodage glouton : un segment peut se répéter en boucle (« Extreme. Extreme. Extreme. »). On en garde 2 de suite au plus.
    let mut run = 0;
    let mut prev = String::new();
    segs.retain(|g| {
        let k = g.text.trim().to_lowercase();
        run = if k == prev { run + 1 } else { 1 };
        prev = k;
        run <= 2
    });
    // le décodage glouton coupe à chaque marque de temps (~3 mots par ligne) : on recolle les fragments
    // d'une même phrase (pas de ponctuation finale, pause < 1 s, 6 s au plus), comme avec le faisceau
    let mut merged: Vec<Segment> = Vec::new();
    for g in segs {
        if let Some(l) = merged.last_mut() {
            let ended = l.text.trim_end().ends_with(['.', '?', '!', '…']);
            if !ended && g.start - l.end < 1.0 && g.end - l.start <= 6.0 {
                l.text.push(' ');
                l.text.push_str(&g.text);
                l.end = g.end;
                continue;
            }
        }
        merged.push(g);
    }
    let segs = merged;
    if segs.is_empty() {
        return Err("Aucune parole détectée.".into());
    }
    Ok(segs)
}

pub fn spawn_transcription(app: AppHandle, id: String) {
    set_status(&app, &id, "queued", None, None);
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        // un verrou « empoisonné » par un plantage précédent ne doit pas bloquer la file
        let _guard = st.job_lock.lock().unwrap_or_else(|e| e.into_inner());
        // un plantage (décodage audio, par exemple) ne doit pas laisser la réunion bloquée en « préparation »
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_whisper(&app, &id)))
            .unwrap_or_else(|_| Err("La transcription a planté (fichier audio illisible ?). Réessayez ou réimportez le fichier.".into()));
        match res {
            Ok(segs) => set_status(&app, &id, "ready", None, Some(segs)),
            Err(e) => set_status(&app, &id, "error", Some(e), None),
        }
    });
}

#[tauri::command]
pub fn retry_transcription(app: AppHandle, st: State<AppState>, id: String) -> R<()> {
    let model = st.store.lock().unwrap().settings.whisper_model.clone();
    if !engine_ready(&st.mdir, &model) {
        return Err("Installez d'abord le moteur Whisper (Réglages).".into());
    }
    spawn_transcription(app, id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coupe_dans_le_silence() {
        // 90 s de bruit, avec un silence de 1 s entre 40,0 s et 41,0 s : la coupe visée (45 s) doit s'y décaler
        let total_ms = 90_000u64;
        let mut w = vec![0u8; 44];
        for i in 0..(total_ms * 16) {
            let silent = (i >= 40_000 * 16) && (i < 41_000 * 16);
            let v: i16 = if silent { 0 } else { 8000 + (i % 7) as i16 * 100 };
            w.extend_from_slice(&v.to_le_bytes());
        }
        let p = std::env::temp_dir().join("verbatim-split-test.wav");
        fs::write(&p, &w).unwrap();
        let cuts = split_points(&p, total_ms, 2);
        let _ = fs::remove_file(&p);
        assert_eq!(cuts.len(), 1);
        assert!((40_000..41_000).contains(&cuts[0]), "coupe à {} ms", cuts[0]);
    }
}
