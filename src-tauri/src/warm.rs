//! Préchargement au démarrage, en arrière-plan : le modèle Whisper et la détection de parole sont lus une fois
//! (ils restent alors dans le cache du système, la transcription suivante ne relit pas le disque) et les modèles de
//! séparation des intervenants sont chargés en mémoire. L'écran de démarrage suit l'avancement avec `warm_status`.

use crate::{engine, AppState};
use serde_json::{json, Value};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

struct Warm {
    label: &'static str,
    pct: u8,
    done: bool,
}
static WARM: Mutex<Warm> = Mutex::new(Warm { label: "", pct: 0, done: false });

fn set(label: &'static str, pct: u8, done: bool) {
    let mut w = WARM.lock().unwrap_or_else(|e| e.into_inner());
    *w = Warm { label, pct: pct.min(100), done };
}

#[tauri::command]
pub fn warm_status() -> Value {
    let w = WARM.lock().unwrap_or_else(|e| e.into_inner());
    json!({ "label": w.label, "pct": w.pct, "done": w.done })
}

/// Au-delà, la lecture préalable n'a plus de sens (le cache du système ne le garderait pas).
const MAX_BYTES: u64 = 3 << 30;

/// Lit tout le fichier ; `on_read` reçoit le nombre d'octets de chaque bloc lu.
fn read_through(p: &Path, on_read: &mut dyn FnMut(u64)) {
    let Ok(mut f) = fs::File::open(p) else { return };
    let mut buf = vec![0u8; 8 << 20];
    while let Ok(n) = f.read(&mut buf) {
        if n == 0 {
            break;
        }
        on_read(n as u64);
    }
}

fn size(p: &Path) -> u64 {
    fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}

fn run(app: &AppHandle, mdir: &PathBuf, model: &str) {
    let whisper: Vec<PathBuf> = [crate::engine_paths::model_path(mdir, model), engine::vad_path(mdir)]
        .into_iter()
        .filter(|p| p.exists() && size(p) <= MAX_BYTES)
        .collect();
    let diar_ready = crate::diarize::ready(mdir);
    let diar: Vec<PathBuf> = if diar_ready { crate::diarize::model_files(mdir) } else { Vec::new() };
    let total: u64 = whisper.iter().chain(&diar).map(|p| size(p)).sum::<u64>().max(1);
    // lecture des fichiers : 0 → 55 % ; séparation des intervenants : → 60 % ; démarrage de whisper-server : → 99 %
    let span = 55.0;
    let mut done = 0u64;
    for (files, label) in [(&whisper, "Chargement de Whisper…"), (&diar, "Chargement de la séparation des voix…")] {
        for p in files {
            set(label, (done as f64 / total as f64 * span) as u8, false);
            read_through(p, &mut |n| {
                done += n;
                set(label, (done as f64 / total as f64 * span) as u8, false);
            });
        }
    }
    if diar_ready {
        set("Chargement de la séparation des voix…", 57, false);
        if let Err(e) = crate::diarize::preload(mdir) {
            eprintln!("Préchargement de la séparation des intervenants impossible : {e}");
        }
    }
    // whisper-server : le modèle est chargé (mémoire vidéo comprise) et reste prêt jusqu'à la fermeture, pour le direct
    set("Chargement de Whisper…", 60, false);
    let t0 = Instant::now();
    std::thread::scope(|sc| {
        let job = sc.spawn(|| crate::live::boot_load(app));
        while !job.is_finished() {
            // durée inconnue : on s'approche de 99 % sans l'atteindre
            let x = t0.elapsed().as_secs_f64();
            set("Chargement de Whisper…", (60.0 + 39.0 * (1.0 - (-x / 8.0).exp())) as u8, false);
            std::thread::sleep(Duration::from_millis(150));
        }
        if let Ok(Err(e)) = job.join() {
            eprintln!("Préchargement de Whisper impossible : {e}");
        }
    });
    set("", 100, true);
}

/// Lance le préchargement (sans bloquer le démarrage). À appeler une fois les moteurs orphelins d'un lancement précédent tués.
pub fn start(app: &AppHandle) {
    let st = app.state::<AppState>();
    let mdir = st.mdir.clone();
    let model = st.store.lock().unwrap().settings.whisper_model.clone();
    let app = app.clone();
    std::thread::spawn(move || {
        // un plantage ici ne doit jamais empêcher l'écran de démarrage de se terminer
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&app, &mdir, &model)));
        set("", 100, true);
    });
}
