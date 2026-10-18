//! Modèles et moteurs déjà présents sur le disque : détection, puis copie dans le dossier de Verbatim.

use crate::{engine, llm, AppState, R};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, State};

struct Item {
    src: PathBuf,
    dest: PathBuf, // relatif au dossier de Verbatim
    kind: &'static str,
    name: String,
    bytes: u64,
    source: String,
}

fn dir_bytes(p: &Path) -> u64 {
    fs::read_dir(p)
        .map(|rd| {
            rd.flatten()
                .map(|e| {
                    let q = e.path();
                    if q.is_dir() { dir_bytes(&q) } else { e.metadata().map(|m| m.len()).unwrap_or(0) }
                })
                .sum()
        })
        .unwrap_or(0)
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '-' }).collect()
}

fn gguf_item(src: PathBuf, label: &str, bytes: u64, source: &str) -> Item {
    let id = llm::LLMS.iter().find(|m| m.3 == bytes).map(|m| m.0.to_string()).unwrap_or_else(|| sanitize(label));
    Item { dest: PathBuf::from("llm-models").join(format!("{id}.gguf")), name: label.to_string(), kind: "llm", src, bytes, source: source.into() }
}

fn whisper_item(src: PathBuf, source: &str) -> Option<Item> {
    let file = src.file_name()?.to_string_lossy().to_string();
    let stem = file.strip_prefix("ggml-")?.strip_suffix(".bin")?.to_string();
    let bytes = fs::metadata(&src).ok()?.len();
    let known = engine::MODELS.iter().any(|m| m.0 == stem && m.3 == bytes) || stem.starts_with("silero");
    known.then(|| Item { dest: PathBuf::from("models").join(&file), kind: "whisper", name: format!("Whisper {stem}"), src, bytes, source: source.into() })
}

fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth > 4 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, depth + 1, out);
        } else {
            out.push(p);
        }
    }
}

fn scan_path(p: &Path, source: &str, out: &mut Vec<Item>) {
    let mut files = Vec::new();
    if p.is_file() {
        files.push(p.to_path_buf());
    } else {
        for root in [p.to_path_buf(), p.join("models")] {
            if root.join("manifests").is_dir() {
                for (name, blob, bytes) in llm::ollama_blob_models(&root) {
                    out.push(gguf_item(blob, &name, bytes, &format!("{source} (Ollama)")));
                }
            }
        }
        walk(p, 0, &mut files);
    }
    for f in files {
        let fname = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if fname.ends_with(".gguf") && !fname.contains("mmproj") {
            let bytes = fs::metadata(&f).map(|m| m.len()).unwrap_or(0);
            out.push(gguf_item(f.clone(), f.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default().as_str(), bytes, source));
        } else if let Some(it) = whisper_item(f, source) {
            out.push(it);
        }
    }
}

fn scan(st: &AppState, extra: Option<&str>) -> Vec<Item> {
    let mut all: Vec<Item> = Vec::new();
    // anciens emplacements (données de l'application)
    if st.dir != st.mdir {
        for (sub, kind, label, probe) in [("engine", "engine", "Moteur de transcription", "whisper-cli.exe"), ("llm", "llm-engine", "Moteur d'IA locale", "llama-server.exe")] {
            let d = st.dir.join(sub);
            if d.join(probe).exists() {
                all.push(Item { dest: PathBuf::from(sub), kind, name: label.into(), bytes: dir_bytes(&d), src: d, source: "ancien dossier de Verbatim".into() });
            }
        }
        scan_path(&st.dir.join("models"), "ancien dossier de Verbatim", &mut all);
        scan_path(&st.dir.join("llm-models"), "ancien dossier de Verbatim", &mut all);
    }
    for d in llm::ollama_model_dirs() {
        scan_path(&d, &d.to_string_lossy(), &mut all);
    }
    if let Some(e) = extra {
        scan_path(Path::new(e), "emplacement choisi", &mut all);
    }
    // sans doublons ni éléments déjà installés
    let mut seen: Vec<PathBuf> = Vec::new();
    all.retain(|it| {
        if seen.contains(&it.src) {
            return false;
        }
        seen.push(it.src.clone());
        let d = st.mdir.join(&it.dest);
        let present = if it.src.is_dir() { d.join(if it.kind == "engine" { "whisper-cli.exe" } else { "llama-server.exe" }).exists() } else { fs::metadata(&d).map(|m| m.len() == it.bytes).unwrap_or(false) };
        !present
    });
    all
}

#[tauri::command]
pub fn models_dir_info(st: State<AppState>) -> Value {
    json!({ "path": st.mdir.to_string_lossy() })
}

/// Langue choisie dans l'installateur (« fr » ou « en », écrite dans `lang.txt` à côté de l'exe), ou vide.
#[tauri::command]
pub fn install_lang() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|e| std::fs::read_to_string(e.with_file_name("lang.txt")).ok())
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s == "fr" || s == "en")
        .unwrap_or_default()
}

#[tauri::command(async)]
pub fn models_scan(st: State<AppState>, extra: Option<String>) -> Vec<Value> {
    scan(&st, extra.as_deref())
        .iter()
        .map(|i| json!({ "src": i.src.to_string_lossy(), "kind": i.kind, "name": i.name, "bytes": i.bytes, "source": i.source }))
        .collect()
}

fn copy_file(src: &Path, dst: &Path, done: &mut u64, total: u64) -> R<()> {
    if let Some(p) = dst.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let part = dst.with_extension("part");
    let mut r = fs::File::open(src).map_err(|e| format!("Lecture impossible : {e}"))?;
    let mut w = fs::File::create(&part).map_err(|e| format!("Écriture impossible : {e}"))?;
    let mut buf = vec![0u8; 4 * 1024 * 1024];
    loop {
        let n = r.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        w.write_all(&buf[..n]).map_err(|e| format!("Écriture impossible (disque plein ?) : {e}"))?;
        *done += n as u64;
        engine::set_dl("copy", *done, total);
    }
    w.flush().map_err(|e| e.to_string())?;
    drop(w);
    let _ = fs::remove_file(dst);
    fs::rename(&part, dst).map_err(|e| e.to_string())
}

fn copy_tree(src: &Path, dst: &Path, done: &mut u64, total: u64) -> R<()> {
    for e in fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
        let (p, q) = (e.path(), dst.join(e.file_name()));
        if p.is_dir() {
            copy_tree(&p, &q, done, total)?;
        } else {
            copy_file(&p, &q, done, total)?;
        }
    }
    Ok(())
}

/// Copie les éléments choisis (chemins source renvoyés par `models_scan`) dans le dossier de Verbatim.
#[tauri::command(async)]
pub fn models_import(app: AppHandle, st: State<AppState>, paths: Vec<String>, extra: Option<String>) -> R<()> {
    let items: Vec<Item> = scan(&st, extra.as_deref()).into_iter().filter(|i| paths.contains(&i.src.to_string_lossy().to_string())).collect();
    if items.is_empty() {
        return Err("Rien à copier.".into());
    }
    let total: u64 = items.iter().map(|i| i.bytes).sum();
    let mdir = st.mdir.clone();
    engine::run_job(app, String::new(), "copy", move || {
        let mut done = 0u64;
        for it in &items {
            let dst = mdir.join(&it.dest);
            if it.src.is_dir() {
                copy_tree(&it.src, &dst, &mut done, total)?;
            } else {
                copy_file(&it.src, &dst, &mut done, total)?;
            }
        }
        Ok(())
    })
}
