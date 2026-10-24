//! Compte-rendu : génération à partir de la transcription.
//! 1) IA locale (llama.cpp + modèle Qwen 3.8 téléchargé) ; 2) API compatible OpenAI ;
//! 3) à défaut, résumé « extractif » calculé sur la transcription (jamais de texte inventé).
//! Les longues réunions sont découpées (notes par partie, puis fusion).

use crate::engine::{self, download_to, download_to_unsized};
use crate::{AppState, Segment, Settings, R};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, State};

const LLAMA_ZIP: &str =
    "https://github.com/ggml-org/llama.cpp/releases/download/b11700/llama-b11700-bin-win-cpu-x64.zip";

const VULKAN_ZIP: &str =
    "https://github.com/ggml-org/llama.cpp/releases/download/b11700/llama-b11700-bin-win-vulkan-x64.zip";
const VULKAN_BYTES: u64 = 32_240_958;
const ROCM_ZIP: &str =
    "https://github.com/ggml-org/llama.cpp/releases/download/b11700/llama-b11700-bin-win-rocm-10.0-x64.zip";
const ROCM_BYTES: u64 = 258_757_149;
// CUDA : le moteur et les DLL runtime (cudart, cuBLAS) sont deux archives extraites dans le même dossier
const CUDA_ZIP: &str =
    "https://github.com/ggml-org/llama.cpp/releases/download/b11700/llama-b11700-bin-win-cuda-12.4-x64.zip";
const CUDA_RT_ZIP: &str =
    "https://github.com/ggml-org/llama.cpp/releases/download/b11700/cudart-llama-bin-win-cuda-12.4-x64.zip";
const CUDA_BYTES: u64 = 266_814_439 + 391_443_627;

/// (id, libellé, description, taille exacte en octets, URL)
pub const LLMS: [(&str, &str, &str, u64, &str); 6] = [
    (
        "qwen3.5-4b",
        "Qwen 3.5 — 4B (léger)",
        "Très économe : ~4 Go de RAM, rapide même sans carte graphique, compte-rendu correct",
        2_740_937_888,
        "https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/resolve/main/Qwen3.5-4B-Q4_K_M.gguf",
    ),
    (
        "qwen3.5-9b",
        "Qwen 3.5 — 9B (équilibré, recommandé)",
        "Bon compromis qualité / mémoire : ~8 Go de RAM, repère bien décisions et actions",
        5_680_522_464,
        "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/main/Qwen3.5-9B-Q4_K_M.gguf",
    ),
    (
        "qwen3.8-27b-q3",
        "Qwen 3.8 — 27B allégé (IQ3)",
        "Qualité supérieure, ~14 Go de RAM ; lent sans carte graphique",
        10_934_860_704,
        "https://huggingface.co/unsloth/Qwen3.8-27B-GGUF/resolve/main/Qwen3.8-27B-UD-IQ3_XXS.gguf",
    ),
    (
        "qwen3.8-27b-q4",
        "Qwen 3.8 — 27B Q4 (maximum)",
        "Meilleure qualité ; ~24 Go de RAM conseillés, lent sans carte graphique",
        18_973_870_528,
        "https://huggingface.co/ggml-org/Qwen3.8-27B-GGUF/resolve/main/Qwen3.8-27B-Q4_K_M.gguf",
    ),
    (
        "translategemma-4b",
        "TranslateGemma 4B",
        "Modèle de traduction de Google (55 langues) : ~3 Go de RAM, rapide, utilisable en direct",
        2_489_909_312,
        "https://huggingface.co/bullerwins/translategemma-4b-it-GGUF/resolve/main/translategemma-4b-it-Q4_K_M.gguf",
    ),
    (
        "translategemma-12b",
        "TranslateGemma 12B",
        "Meilleure traduction, ~9 Go de RAM ; plus lent, surtout sans carte graphique",
        7_300_793_664,
        "https://huggingface.co/bullerwins/translategemma-12b-it-GGUF/resolve/main/translategemma-12b-it-Q4_K_M.gguf",
    ),
];

/// Modèle dédié à la traduction (invite spécifique, pas de compte-rendu).
pub fn is_translator(id: &str) -> bool {
    id.starts_with("translategemma")
}

fn dev_dir(dev: &str) -> &'static str {
    match dev {
        "vulkan" => "llm-vulkan",
        "rocm" => "llm-rocm",
        "cuda" => "llm-cuda",
        _ => "llm",
    }
}
fn bin_for(dir: &PathBuf, dev: &str) -> PathBuf {
    crate::engine_paths::pick(dir, &format!("{}\\llama-server.exe", dev_dir(dev)))
}
fn bin_path(dir: &PathBuf) -> PathBuf {
    bin_for(dir, "cpu")
}
/// Au moins un moteur llama.cpp (CPU, Vulkan, ROCm ou CUDA) est installé.
fn any_engine(dir: &PathBuf) -> bool {
    ["cpu", "vulkan", "rocm", "cuda"].iter().any(|d| bin_for(dir, d).exists())
}

// ─── Runtime ROCm : DLL HIP / rocBLAS… livrées dans `redist/rocm-runtime` (ou désignées à la main) ───

/// Reconnaît un runtime ROCm dans `root` : dossier plat de DLL, ou site-packages d'un venv AMD
/// (`_rocm_sdk_core`). Renvoie (dossier retenu, dossiers à ajouter au PATH).
fn rocm_find(root: &std::path::Path) -> Option<(PathBuf, Vec<PathBuf>)> {
    let has = |d: &std::path::Path, prefix: &str| {
        fs::read_dir(d).map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().starts_with(prefix))).unwrap_or(false)
    };
    // dossier plat : amdhip64 + rocblas
    if has(root, "amdhip64") && has(root, "rocblas") {
        return Some((root.to_path_buf(), vec![root.to_path_buf()]));
    }
    let cands = [
        root.to_path_buf(),
        root.join("Lib").join("site-packages"),
        root.join("venv").join("Lib").join("site-packages"),
        root.join(".venv").join("Lib").join("site-packages"),
    ];
    for sp in cands {
        let core = sp.join("_rocm_sdk_core").join("bin");
        if !has(&core, "amdhip64") {
            continue;
        }
        let mut dirs = vec![core];
        if let Ok(rd) = fs::read_dir(&sp) {
            for e in rd.flatten() {
                if e.file_name().to_string_lossy().starts_with("_rocm_sdk_libraries") && e.path().join("bin").is_dir() {
                    dirs.push(e.path().join("bin"));
                }
            }
        }
        if dirs.len() > 1 {
            return Some((sp, dirs));
        }
    }
    None
}

/// Ordre : dossier choisi dans les Réglages, `ia/rocm-runtime`, puis `redist/rocm-runtime`.
fn rocm_runtime(dir: &PathBuf, root: &str) -> Option<(PathBuf, Vec<PathBuf>)> {
    if !root.trim().is_empty() {
        if let Some(f) = rocm_find(std::path::Path::new(root)) {
            return Some(f);
        }
    }
    let p = crate::engine_paths::pick(dir, "rocm-runtime");
    rocm_find(&p)
}

/// Vérifie un dossier choisi par l'utilisateur ; renvoie le dossier retenu.
#[tauri::command(async)]
pub fn llm_rocm_check(path: String) -> R<String> {
    rocm_find(std::path::Path::new(&path))
        .map(|f| f.0.to_string_lossy().to_string())
        .ok_or_else(|| "Runtime ROCm introuvable dans ce dossier (attendu : un dossier contenant amdhip64_7.dll et rocblas.dll).".to_string())
}

/// Moteur à utiliser selon le réglage, avec repli (CUDA → ROCm → Vulkan → CPU).
fn pick_engine(dir: &PathBuf, cfg: &Settings) -> Option<(PathBuf, &'static str, Vec<PathBuf>)> {
    let order: &[&'static str] = match cfg.llm_device.as_str() {
        "cpu_only" => &["cpu"],
        "vulkan" => &["vulkan", "cpu"],
        "rocm" => &["rocm", "vulkan", "cpu"],
        "cuda" => &["cuda", "vulkan", "cpu"],
        // « auto » (et l'ancienne valeur « cpu ») : le plus rapide disponible
        _ => &["cuda", "rocm", "vulkan", "cpu"],
    };
    for d in order {
        let b = bin_for(dir, d);
        if !b.exists() {
            continue;
        }
        if *d == "cuda" && !engine::cuda_available() {
            continue;
        }
        if *d == "rocm" {
            if let Some((_, dirs)) = rocm_runtime(dir, &cfg.rocm_root) {
                return Some((b, "rocm", dirs));
            }
            continue;
        }
        return Some((b, d, vec![]));
    }
    ["cpu", "vulkan"].iter().find(|d| bin_for(dir, d).exists()).map(|d| (bin_for(dir, d), *d, vec![]))
}

/// Matériel qu'utilisera le moteur d'IA locale d'après les réglages.
fn engine_hw(dir: &PathBuf, cfg: &Settings) -> String {
    match pick_engine(dir, cfg) {
        Some((_, dev, _)) => engine::hw_label(dev),
        None => "CPU".into(),
    }
}

pub fn local_ready(dir: &PathBuf, id: &str) -> bool {
    !id.is_empty()
        && any_engine(dir)
        && model_info(id).map(|m| fs::metadata(model_file(dir, id)).map(|f| f.len() == m.3).unwrap_or(false)).unwrap_or(false)
}

/// Les modèles de rédaction sont dans `ia/llm-models`, ceux de traduction dans `ia/translation-models`.
fn model_file(dir: &PathBuf, id: &str) -> PathBuf {
    dir.join(if is_translator(id) { "translation-models" } else { "llm-models" }).join(format!("{id}.gguf"))
}

/// Réglages avec le calcul propre à la traduction (processeur seul, ou meilleur GPU disponible).
pub fn translate_cfg(cfg: &Settings) -> Settings {
    let mut c = cfg.clone();
    c.llm_device = if cfg.translate_device == "cpu_only" { "cpu_only".into() } else { "auto".into() };
    c
}
fn model_info(id: &str) -> Option<&'static (&'static str, &'static str, &'static str, u64, &'static str)> {
    LLMS.iter().find(|m| m.0 == id)
}

/// Fragment ajouté à `engine_status` : état de l'IA locale.
pub fn status_json(dir: &PathBuf, selected: &str, cfg: &Settings) -> Value {
    let models: Vec<Value> = LLMS
        .iter()
        .map(|(id, label, desc, bytes, _)| {
            let mp = model_file(dir, id);
            let installed = fs::metadata(&mp).map(|m| m.len() == *bytes).unwrap_or(false);
            let partial = if installed { 0 } else { fs::metadata(mp.with_extension("part")).map(|m| m.len()).unwrap_or(0) };
            json!({ "name": id, "label": label, "desc": desc, "bytes": bytes, "installed": installed, "translator": is_translator(id),
                    "partial": partial, "selected": *id == selected, "path": mp.to_string_lossy() })
        })
        .collect();
    let rocm = rocm_runtime(dir, &cfg.rocm_root);
    json!({
        "binary": bin_path(dir).exists(), "binary_path": bin_path(dir).to_string_lossy(), "models": models,
        "device": cfg.llm_device,
        "translate_hw": engine_hw(dir, &translate_cfg(cfg)),
        "translation_dir": dir.join("translation-models").to_string_lossy(),
        "gpus": engine::gpu_names(),
        "vram": engine::gpu_vram(),
        "engines": {
            "cpu": { "installed": bin_path(dir).exists(), "path": bin_path(dir).to_string_lossy(), "bytes": 19_593_035u64 },
            "vulkan": { "installed": bin_for(dir, "vulkan").exists(), "path": bin_for(dir, "vulkan").to_string_lossy(), "bytes": VULKAN_BYTES },
            "cuda": { "installed": bin_for(dir, "cuda").exists(), "path": bin_for(dir, "cuda").to_string_lossy(), "bytes": CUDA_BYTES },
            "rocm": { "installed": bin_for(dir, "rocm").exists(), "path": bin_for(dir, "rocm").to_string_lossy(), "bytes": ROCM_BYTES,
                      "runtime": rocm.as_ref().map(|r| r.0.to_string_lossy().to_string()) },
        },
    })
}

fn install(dir: &PathBuf, id: &str) -> R<()> {
    let info = model_info(id).ok_or("Modèle inconnu.")?;
    install_binary(dir, "cpu")?;
    let mp = model_file(dir, id);
    if fs::metadata(&mp).map(|m| m.len() != info.3).unwrap_or(true) {
        download_to(info.4, &mp, info.3, "llm-model", id)?;
    }
    Ok(())
}

fn install_binary(dir: &PathBuf, dev: &str) -> R<()> {
    let url = match dev {
        "vulkan" => VULKAN_ZIP,
        "rocm" => ROCM_ZIP,
        "cuda" => CUDA_ZIP,
        _ => LLAMA_ZIP,
    };
    let bin = bin_for(dir, dev);
    if !bin.exists() {
        let out = dir.join(dev_dir(dev));
        fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        let mut urls = vec![url];
        if dev == "cuda" {
            urls.push(CUDA_RT_ZIP);
        }
        for u in urls {
            let zip_path = dir.join("llm.zip");
            download_to_unsized(u, &zip_path, "llm-binary")?;
            let files = engine::extract_flat(&zip_path, &out)?;
            let _ = fs::remove_file(&zip_path);
            engine::verify_kept(&files)?;
        }
        if !bin.exists() {
            return Err("llama-server.exe absent de l'archive.".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn llm_install_engine(app: AppHandle, st: State<AppState>, device: String) -> R<()> {
    let dir = st.mdir.clone();
    engine::run_job(app, String::new(), "llm-binary", move || install_binary(&dir, &device))
}

#[tauri::command]
pub fn llm_install(app: AppHandle, st: State<AppState>, model: String) -> R<()> {
    model_info(&model).ok_or("Modèle inconnu.")?;
    let dir = st.mdir.clone();
    let m = model.clone();
    engine::run_job(app, model, "llm-binary", move || install(&dir, &m))
}

/// Si le modèle supprimé est celui sélectionné, on revient à « Aucune IA locale ».
fn unselect_llm(st: &AppState, value: &str) {
    let mut g = st.store.lock().unwrap();
    if g.settings.llm_local == value {
        g.settings.llm_local.clear();
        let _ = st.save(&g);
    }
    if g.settings.translate_model == value {
        g.settings.translate_model.clear();
        let _ = st.save(&g);
    }
}

#[tauri::command]
pub fn llm_remove(st: State<AppState>, model: String) -> R<()> {
    model_info(&model).ok_or("Modèle inconnu.")?;
    unselect_llm(&st, &model);
    let mp = model_file(&st.mdir, &model);
    let _ = fs::remove_file(&mp);
    let _ = fs::remove_file(mp.with_extension("part"));
    Ok(())
}

// ─── Serveur llama.cpp local (démarré à la demande, arrêté ensuite) ─────────

struct Server {
    child: Child,
    port: u16,
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Dossiers à ajouter au PATH pour charger le runtime ROCm (utilisé aussi par Whisper).
pub fn rocm_path_dirs(dir: &PathBuf, root: &str) -> Option<Vec<PathBuf>> {
    rocm_runtime(dir, root).map(|(_, p)| p)
}

/// Le moteur GPU s'est arrêté brutalement (connexion coupée) : typiquement un antivirus qui supprime ou bloque le runtime.
fn engine_crashed(e: &str) -> bool {
    let l = e.to_lowercase();
    ["10054", "10053", "10061", "arrêtée au démarrage", "connexion", "connection"].iter().any(|k| l.contains(k))
}

fn crash_hint(e: &str) -> String {
    if engine_crashed(e) {
        format!("{e} — le moteur d'IA a été arrêté, souvent par l'antivirus qui met en quarantaine le runtime (Vulkan, CUDA, ROCm). Restaurez-le dans Sécurité Windows › Historique de protection et ajoutez le dossier « ia » aux exclusions, ou choisissez « CPU uniquement » dans Réglages › Compte-rendu")
    } else {
        e.to_string()
    }
}

/// Lit le modèle en entier depuis le disque (par blocs, sans le garder) en rendant compte de l'avancement en direct :
/// le pourcentage est exact, et le fichier reste ensuite dans le cache de Windows, ce qui accélère le démarrage du serveur.
fn read_model(model: &PathBuf, on_pct: &dyn Fn(u8)) {
    use std::io::Read;
    let Ok(mut f) = fs::File::open(model) else { return };
    let total = f.metadata().map(|m| m.len()).unwrap_or(0).max(1);
    let mut buf = vec![0u8; 16 << 20];
    let (mut done, mut last) = (0u64, 255u8);
    on_pct(0);
    while let Ok(n) = f.read(&mut buf) {
        if n == 0 {
            break;
        }
        done += n as u64;
        let pct = (done * 100 / total).min(100) as u8;
        if pct != last {
            on_pct(pct);
            last = pct;
        }
    }
}

/// Lance llama-server et attend qu'il soit prêt. `on_pct` reçoit l'avancement de la lecture du modèle sur le disque (0 à 100) ;
/// à 100, le serveur démarre.
fn start_server(dir: &PathBuf, model: &PathBuf, cfg: &Settings, on_pct: &dyn Fn(u8)) -> R<Server> {
    let (bin, _dev, extra_path) = pick_engine(dir, cfg).ok_or("Moteur d'IA locale non installé (Réglages).")?;
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        l.local_addr().map_err(|e| e.to_string())?.port()
    };
    let threads = (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8) / 2).clamp(4, 8);
    read_model(model, on_pct);
    let mut cmd = Command::new(bin);
    if !extra_path.is_empty() {
        // runtime ROCm : ses DLL (HIP, rocBLAS…) doivent être trouvables
        let mut p: Vec<std::ffi::OsString> = extra_path.iter().map(|d| d.as_os_str().to_owned()).collect();
        p.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).map(|x| x.into_os_string()));
        if let Ok(j) = std::env::join_paths(p) {
            cmd.env("PATH", j);
        }
    }
    cmd.arg("-m")
        .arg(model)
        .args(["--host", "127.0.0.1", "--port", &port.to_string(), "-c", "8192", "-np", "1", "-t", &threads.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let child = cmd.spawn().map_err(|e| format!("Lancement de l'IA locale impossible : {e}"))?;
    crate::procs::adopt(&child);
    let mut srv = Server { child, port };
    let t0 = Instant::now();
    loop {
        if let Ok(Some(status)) = srv.child.try_wait() {
            return Err(format!("L'IA locale s'est arrêtée au démarrage ({status}). Mémoire insuffisante ?"));
        }
        if ureq::get(&format!("http://127.0.0.1:{port}/health"))
            .timeout(Duration::from_secs(3))
            .call()
            .is_ok()
        {
            return Ok(srv);
        }
        if t0.elapsed() > Duration::from_secs(300) {
            return Err("Délai dépassé au chargement de l'IA locale.".into());
        }
        std::thread::sleep(Duration::from_millis(700));
    }
}

// ─── Appel de modèle (local ou distant) ────────────────────────────────────

pub struct Backend {
    /// matériel de calcul affiché à l'utilisateur
    pub hw: String,
    base: String,
    key: String,
    model: String,
    pub label: String,
    mode: &'static str, // "local" | "api" | "ollama"
    /// "openai" (llama.cpp, API distante) ou "ollama" (API native)
    kind: &'static str,
    /// taille max d'une tranche de transcription, en caractères
    chunk_chars: usize,
    timeout: u64,
}

fn parse_json_loose(txt: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(txt) {
        return Some(v);
    }
    let (a, b) = (txt.find('{')?, txt.rfind('}')?);
    serde_json::from_str(&txt[a..=b]).ok()
}

/// `on_frac` reçoit l'avancement estimé de l'appel (0.0 à 1.0).
fn chat_json(be: &Backend, system: &str, user: &str, max_tokens: u32, on_frac: &dyn Fn(f32)) -> R<Value> {
    if be.kind == "ollama" {
        return ollama_chat(be, system, user, max_tokens);
    }
    let body = json!({
        "model": be.model,
        "temperature": 0.2,
        "max_tokens": max_tokens,
        "stream": true,
        "chat_template_kwargs": { "enable_thinking": false }, // Qwen 3 : pas de phase de réflexion
        "response_format": { "type": "json_object" },
        "messages": [{ "role": "system", "content": system }, { "role": "user", "content": user }]
    });
    let mut req = ureq::post(&format!("{}/chat/completions", be.base.trim_end_matches('/')))
        .timeout(Duration::from_secs(be.timeout));
    if !be.key.is_empty() {
        req = req.set("Authorization", &format!("Bearer {}", be.key));
    }
    let resp = req.send_json(body).map_err(|e| format!("Appel du modèle impossible : {e}"))?;
    // la réponse JSON d'un compte-rendu fait en général ~60 % du maximum autorisé
    let expected = (max_tokens as f32 * 0.6).max(1.0);
    let mut txt = String::new();
    let mut chunks = 0u32;
    for line in std::io::BufRead::lines(std::io::BufReader::new(resp.into_reader())) {
        let line = line.map_err(|e| e.to_string())?;
        let Some(data) = line.strip_prefix("data:") else { continue };
        let data = data.trim();
        if data == "[DONE]" {
            break;
        }
        if let Ok(v) = serde_json::from_str::<Value>(data) {
            if let Some(t) = v["choices"][0]["delta"]["content"].as_str() {
                txt.push_str(t);
                chunks += 1;
                on_frac((chunks as f32 / expected).min(0.97));
            }
        }
    }
    if txt.trim().is_empty() {
        return Err("Réponse vide du modèle.".into());
    }
    on_frac(1.0);
    parse_json_loose(&txt).ok_or_else(|| "Réponse du modèle non exploitable (JSON invalide).".to_string())
}

/// Réponse en texte libre (traduction) ; pas de mode JSON.
pub fn chat_text(be: &Backend, system: &str, user: &str, max_tokens: u32) -> R<String> {
    if be.kind == "ollama" {
        let body = json!({
            "model": be.model, "stream": false, "think": false, "keep_alive": "2m",
            "options": { "temperature": 0.1, "num_ctx": 12288, "num_predict": max_tokens },
            "messages": [{ "role": "system", "content": system }, { "role": "user", "content": user }]
        });
        let r: Value = ureq::post(&format!("{}/api/chat", be.base.trim_end_matches('/')))
            .timeout(Duration::from_secs(be.timeout)).send_json(body).map_err(|e| format!("Appel du modèle impossible : {e}"))?
            .into_json().map_err(|e| e.to_string())?;
        let mut txt = r["message"]["content"].as_str().ok_or("Réponse vide d'Ollama.")?.to_string();
        if let Some(i) = txt.rfind("</think>") { txt = txt[i + 8..].to_string(); }
        return Ok(txt.trim().to_string());
    }
    let body = json!({
        "model": be.model, "temperature": 0.1, "max_tokens": max_tokens, "stream": false,
        "chat_template_kwargs": { "enable_thinking": false },
        "messages": [{ "role": "system", "content": system }, { "role": "user", "content": user }]
    });
    let mut req = ureq::post(&format!("{}/chat/completions", be.base.trim_end_matches('/'))).timeout(Duration::from_secs(be.timeout));
    if !be.key.is_empty() { req = req.set("Authorization", &format!("Bearer {}", be.key)); }
    let r: Value = req.send_json(body).map_err(|e| format!("Appel du modèle impossible : {e}"))?.into_json().map_err(|e| e.to_string())?;
    let mut txt = r["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
    if let Some(i) = txt.rfind("</think>") { txt = txt[i + 8..].to_string(); }
    if txt.trim().is_empty() { return Err("Réponse vide du modèle.".into()); }
    Ok(txt.trim().to_string())
}

/// Complétion brute de llama-server (invite déjà au format du modèle, ex. TranslateGemma).
pub fn raw_complete(be: &Backend, prompt: &str, max_tokens: u32, stop: &[&str]) -> R<String> {
    let url = format!("{}/completion", be.base.trim_end_matches('/').trim_end_matches("/v1"));
    let body = json!({ "prompt": prompt, "n_predict": max_tokens, "temperature": 0.0, "stream": false, "stop": stop, "cache_prompt": true });
    let r: Value = ureq::post(&url).timeout(Duration::from_secs(be.timeout)).send_json(body)
        .map_err(|e| format!("Appel du modèle impossible : {e}"))?.into_json().map_err(|e| e.to_string())?;
    let txt = r["content"].as_str().unwrap_or("").trim().to_string();
    if txt.is_empty() { return Err("Réponse vide du modèle.".into()); }
    Ok(txt)
}

/// Prépare le moteur demandé par `sel` (« ollama:nom », « gguf:chemin », identifiant du catalogue, ou API distante),
/// exécute `f`, puis l'arrête. Relance une fois sur le processeur si le moteur GPU a été coupé.
pub fn with_backend<T>(app: &AppHandle, cfg: &Settings, sel: &str, status: &dyn Fn(&str), f: &dyn Fn(&Backend) -> R<T>) -> R<T> {
    let st = app.state::<AppState>();
    if let Some(name) = sel.strip_prefix("ollama:") {
        status("Démarrage d'Ollama…");
        ensure_ollama()?;
        let be = Backend { base: OLLAMA_URL.into(), key: String::new(), model: name.to_string(), label: format!("Ollama · {name}"),
            hw: "Ollama (matériel géré par Ollama)".to_string(), mode: "ollama", kind: "ollama", chunk_chars: 16_000, timeout: 3600 };
        return f(&be);
    }
    let local: Option<(PathBuf, String)> = if let Some(path) = sel.strip_prefix("gguf:") {
        let path = PathBuf::from(path);
        if !path.exists() { return Err("Le fichier du modèle est introuvable.".into()); }
        if !any_engine(&st.mdir) { return Err("Le moteur d'IA locale (llama.cpp) n'est pas installé : téléchargez d'abord un modèle dans Réglages.".into()); }
        let name = gguf_display_name(&path);
        Some((path, name))
    } else if local_ready(&st.mdir, sel) {
        let info = model_info(sel).unwrap();
        Some((model_file(&st.mdir, sel), info.1.to_string()))
    } else {
        None
    };
    if let Some((path, label)) = local {
        status("Chargement de l'IA locale…");
        let attempt = |cfg: &Settings| -> R<T> {
            let srv = start_server(&st.mdir, &path, cfg, &|p| status(&load_text(p)))?;
            let be = Backend { base: format!("http://127.0.0.1:{}/v1", srv.port), key: String::new(), model: "local".into(), label: label.clone(),
                hw: engine_hw(&st.mdir, cfg), mode: "local", kind: "openai", chunk_chars: 11_000, timeout: 1800 };
            let r = f(&be);
            drop(srv);
            r
        };
        let mut res = attempt(cfg);
        if res.as_ref().err().map_or(false, |e| engine_crashed(e)) && cfg.llm_device != "cpu_only" {
            status("Moteur GPU interrompu : nouvel essai sur le processeur…");
            let mut cpu = cfg.clone();
            cpu.llm_device = "cpu_only".into();
            res = attempt(&cpu);
        }
        return res.map_err(|e| crash_hint(&e));
    }
    if !cfg.llm_base_url.trim().is_empty() {
        let m = if cfg.llm_model.is_empty() { "gpt-4o-mini".to_string() } else { cfg.llm_model.clone() };
        let be = Backend { base: cfg.llm_base_url.clone(), key: cfg.llm_api_key.clone(), model: m.clone(), label: format!("API · {m}"),
            hw: "Serveur distant (API)".to_string(), mode: "api", kind: "openai", chunk_chars: 40_000, timeout: 240 };
        return f(&be);
    }
    Err("Aucun modèle d'IA disponible : installez un modèle dans Réglages › Compte-rendu.".into())
}

// ─── Ollama ────────────────────────────────────────────────────────────────

const OLLAMA_URL: &str = "http://127.0.0.1:11434";

fn ollama_tags() -> Option<Value> {
    ureq::get(&format!("{OLLAMA_URL}/api/tags")).timeout(Duration::from_secs(2)).call().ok()?.into_json().ok()
}

/// Cherche une installation d'Ollama (PATH, emplacements usuels, racine de chaque lecteur).
/// Renvoie (ollama.exe, « ollama app.exe » s'il existe).
fn find_ollama() -> Option<(PathBuf, Option<PathBuf>)> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut wh = Command::new("where.exe");
    wh.arg("ollama").stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        wh.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    if let Ok(o) = wh.output() {
        for l in String::from_utf8_lossy(&o.stdout).lines() {
            if let Some(d) = PathBuf::from(l.trim()).parent() {
                dirs.push(d.to_path_buf());
            }
        }
    }
    if let Ok(l) = std::env::var("LOCALAPPDATA") {
        dirs.push(PathBuf::from(l).join("Programs").join("Ollama"));
    }
    if let Ok(l) = std::env::var("ProgramFiles") {
        dirs.push(PathBuf::from(l).join("Ollama"));
    }
    for c in 'C'..='Z' {
        dirs.push(PathBuf::from(format!("{c}:\\ollama")));
        dirs.push(PathBuf::from(format!("{c}:\\Ollama")));
    }
    let found: Vec<PathBuf> = dirs.into_iter().filter(|d| d.join("ollama.exe").exists()).collect();
    // préfère l'installation qui contient réellement des modèles
    let best = found
        .iter()
        .find(|d| d.join("models").join("manifests").exists())
        .or_else(|| found.iter().find(|d| d.join("ollama app.exe").exists()))
        .or_else(|| found.first())?;
    let app = best.join("ollama app.exe");
    Some((best.join("ollama.exe"), app.exists().then_some(app)))
}

/// Vérifie qu'Ollama répond ; sinon le démarre (application de la barre des tâches, qui applique vos réglages).
fn ensure_ollama() -> R<()> {
    if ollama_tags().is_some() {
        return Ok(());
    }
    let (exe, app) = find_ollama().ok_or("Ollama est introuvable sur cet ordinateur.")?;
    let mut cmd = match app {
        Some(a) => Command::new(a),
        None => {
            let mut c = Command::new(exe);
            c.arg("serve");
            c
        }
    };
    cmd.stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000 | 0x0000_0008); // CREATE_NO_WINDOW | DETACHED_PROCESS
    }
    cmd.spawn().map_err(|e| format!("Démarrage d'Ollama impossible : {e}"))?;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(90) {
        if ollama_tags().is_some() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(700));
    }
    Err("Ollama ne répond pas après son démarrage.".into())
}

fn ollama_chat(be: &Backend, system: &str, user: &str, max_tokens: u32) -> R<Value> {
    let build = |with_think: bool| {
        let mut b = json!({
            "model": be.model, "stream": false, "format": "json", "keep_alive": "2m",
            "options": { "temperature": 0.2, "num_ctx": 12288, "num_predict": max_tokens },
            "messages": [{ "role": "system", "content": system }, { "role": "user", "content": user }]
        });
        if with_think {
            b["think"] = json!(false); // modèles « raisonnants » (Qwen 3) : pas de phase de réflexion
        }
        b
    };
    let call = |body: Value| -> Result<Value, String> {
        match ureq::post(&format!("{}/api/chat", be.base.trim_end_matches('/'))).timeout(Duration::from_secs(be.timeout)).send_json(body) {
            Ok(r) => r.into_json::<Value>().map_err(|e| e.to_string()),
            Err(ureq::Error::Status(code, r)) => Err(format!("HTTP {code} : {}", r.into_string().unwrap_or_default())),
            Err(e) => Err(e.to_string()),
        }
    };
    let resp = match call(build(true)) {
        Err(e) if e.to_lowercase().contains("think") => call(build(false))?, // modèle sans mode « réflexion »
        other => other?,
    };
    let mut txt = resp["message"]["content"].as_str().ok_or("Réponse vide d'Ollama.")?.to_string();
    if let Some(i) = txt.rfind("</think>") {
        txt = txt[i + 8..].to_string();
    }
    parse_json_loose(&txt).ok_or_else(|| "Réponse d'Ollama non exploitable (JSON invalide).".to_string())
}

#[tauri::command(async)]
pub fn ollama_status() -> R<Value> {
    let found = find_ollama();
    let tags = ollama_tags();
    let models: Vec<Value> = tags
        .as_ref()
        .and_then(|t| t["models"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(|m| {
            json!({ "name": m["name"], "bytes": m["size"], "params": m["details"]["parameter_size"],
                    "quant": m["details"]["quantization_level"] })
        })
        .collect();
    Ok(json!({
        "installed": found.is_some(),
        "running": tags.is_some(),
        "exe": found.map(|f| f.0.to_string_lossy().to_string()),
        "models": models,
    }))
}

/// Dossiers de modèles Ollama possibles (OLLAMA_MODELS, ~/.ollama, <lecteur>:\ollama).
pub fn ollama_model_dirs() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if let Ok(p) = std::env::var("OLLAMA_MODELS") {
        v.push(PathBuf::from(p));
    }
    if let Ok(h) = std::env::var("USERPROFILE") {
        v.push(PathBuf::from(h).join(".ollama").join("models"));
    }
    for c in 'C'..='Z' {
        v.push(PathBuf::from(format!(r"{c}:\ollama\models")));
    }
    let mut out: Vec<PathBuf> = Vec::new();
    for d in v {
        if d.join("manifests").is_dir() && !out.iter().any(|o| o.to_string_lossy().eq_ignore_ascii_case(&d.to_string_lossy())) {
            out.push(d);
        }
    }
    out
}

fn gguf_display_name(path: &PathBuf) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "modèle local".into())
}

/// Lit les manifestes d'un dossier Ollama : (nom:tag, fichier GGUF, taille). Sans lancer Ollama.
pub fn ollama_blob_models(root: &PathBuf) -> Vec<(String, PathBuf, u64)> {
    let mut out = Vec::new();
    let base = root.join("manifests");
    let mut stack = vec![base.clone()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let Ok(txt) = fs::read_to_string(&p) else { continue };
            let Ok(m) = serde_json::from_str::<Value>(&txt) else { continue };
            let Some(layer) = m["layers"].as_array().and_then(|l| l.iter().find(|x| x["mediaType"] == "application/vnd.ollama.image.model")) else { continue };
            let Some(digest) = layer["digest"].as_str() else { continue };
            let blob = root.join("blobs").join(digest.replace(':', "-"));
            if let Ok(meta) = fs::metadata(&blob) {
                let rel = p.strip_prefix(&base).unwrap_or(&p);
                let parts: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
                let name = match parts.as_slice() {
                    [_, ns, model, tag] if ns == "library" => format!("{model}:{tag}"),
                    [_, ns, model, tag] => format!("{ns}/{model}:{tag}"),
                    _ => parts.join("/"),
                };
                out.push((name, blob, meta.len()));
            }
        }
    }
    out
}

/// Fichiers GGUF installés dans le dossier de Verbatim qui ne font pas partie du catalogue.
#[tauri::command(async)]
pub fn local_gguf_list(st: State<AppState>) -> Vec<Value> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(st.mdir.join("llm-models")) {
        for e in rd.flatten() {
            let p = e.path();
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            if p.extension().map(|x| x == "gguf").unwrap_or(false) && model_info(&stem).is_none() {
                let bytes = e.metadata().map(|m| m.len()).unwrap_or(0);
                out.push(json!({ "name": stem, "path": p.to_string_lossy(), "bytes": bytes }));
            }
        }
    }
    out
}

/// Supprime un GGUF personnalisé (uniquement dans le dossier de Verbatim).
#[tauri::command]
pub fn llm_remove_file(st: State<AppState>, path: String) -> R<()> {
    let p = PathBuf::from(&path);
    if !p.starts_with(st.mdir.join("llm-models")) || p.extension().map(|x| x != "gguf").unwrap_or(true) {
        return Err("Fichier non géré par Verbatim.".into());
    }
    unselect_llm(&st, &format!("gguf:{path}"));
    fs::remove_file(p).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn ollama_start() -> R<()> {
    ensure_ollama()
}

// ─── Prompts ───────────────────────────────────────────────────────────────

/// Nom (en français) de la langue de rédaction ; « auto » = celle de la transcription.
pub fn lang_name(code: &str) -> &'static str {
    match code {
        "fr" => "français", "en" => "anglais", "es" => "espagnol", "de" => "allemand", "it" => "italien",
        "pt" => "portugais", "nl" => "néerlandais", "pl" => "polonais", "ru" => "russe", "ja" => "japonais",
        "zh" => "chinois", "ar" => "arabe", "tr" => "turc", "ko" => "coréen",
        _ => "",
    }
}

fn rules(lang: &str) -> String {
    let write = match lang_name(lang) {
        "" => "Écris dans la langue de la transcription, de façon claire et concise.".to_string(),
        n => format!("Écris le compte-rendu en {n} (traduis les propos si besoin), de façon claire et concise."),
    };
    format!("{RULES}- {write}\n")
}

const RULES: &str = "Règles :\n\
- N'utilise QUE les informations de la transcription ; n'invente ni chiffres, ni noms, ni décisions.\n\
- La transcription vient d'une reconnaissance vocale automatique et peut contenir des erreurs : corrige l'orthographe des termes du glossaire fourni.\n\
- Les intervenants ne sont pas identifiés : déduis les responsables uniquement des prénoms cités dans les propos.\n\
- \"decisions\" : uniquement ce qui a été explicitement décidé ou validé.\n\
- \"actions\" : chaque tâche à faire, au format « Responsable — action — échéance » (omets ce qui n'est pas dit).\n\
- \"next_steps\" : points reportés, prochaine réunion, suites prévues.\n\
- Reprends tels quels les chiffres, dates et noms propres cités.\n";

fn system_full(lang: &str) -> String {
    let rules = rules(lang);
    format!(
        "Tu es un assistant qui rédige des comptes-rendus de réunion professionnels, fidèles à la transcription fournie.\n{rules}\
         Réponds UNIQUEMENT avec un objet JSON : {{\"summary\": \"5 à 8 phrases qui racontent la réunion\", \
         \"topics\": [\"Sujet : ce qui a été dit (chiffres, positions)\"], \"decisions\": [\"…\"], \
         \"actions\": [\"…\"], \"next_steps\": [\"…\"]}}. Liste vide s'il n'y a rien pour une section."
    )
}
fn system_map(lang: &str) -> String {
    let rules = rules(lang);
    format!(
        "Tu prends des notes sur un extrait d'une réunion (une seule partie de la réunion).\n{rules}\
         Réponds UNIQUEMENT avec un objet JSON : {{\"summary\": \"2 à 4 phrases sur cette partie\", \
         \"topics\": [\"…\"], \"decisions\": [\"…\"], \"actions\": [\"…\"], \"next_steps\": [\"…\"]}}."
    )
}
fn system_reduce(lang: &str) -> String {
    let rules = rules(lang);
    format!(
        "On te donne les notes prises sur les parties successives d'une même réunion. Fusionne-les en un seul compte-rendu : \
         ordre chronologique, sans doublons, sans rien ajouter.\n{rules}\
         Réponds UNIQUEMENT avec un objet JSON : {{\"summary\": \"5 à 8 phrases qui racontent toute la réunion\", \
         \"topics\": [\"…\"], \"decisions\": [\"…\"], \"actions\": [\"…\"], \"next_steps\": [\"…\"]}}."
    )
}

fn clock(s: f64) -> String {
    let t = s as u32;
    if t >= 3600 {
        format!("{:02}:{:02}:{:02}", t / 3600, t % 3600 / 60, t % 60)
    } else {
        format!("{:02}:{:02}", t / 60, t % 60)
    }
}

fn transcript_lines(segs: &[Segment]) -> Vec<String> {
    segs.iter()
        .map(|s| {
            if s.speaker.is_empty() || s.speaker == "Intervenant" {
                format!("[{}] {}", clock(s.start), s.text)
            } else {
                format!("[{}] {} : {}", clock(s.start), s.speaker, s.text)
            }
        })
        .collect()
}

fn chunk(lines: &[String], budget: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for l in lines {
        if !cur.is_empty() && cur.len() + l.len() + 1 > budget {
            out.push(std::mem::take(&mut cur));
        }
        cur.push_str(l);
        cur.push('\n');
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

// ─── Normalisation de la sortie du modèle ──────────────────────────────────

fn as_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.trim().to_string(),
        Value::Object(o) => o
            .values()
            .filter_map(|x| x.as_str())
            .map(|x| x.trim())
            .filter(|x| !x.is_empty() && !x.eq_ignore_ascii_case("non précisé") && !x.eq_ignore_ascii_case("pas précisé"))
            .collect::<Vec<_>>()
            .join(" — "),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn list_of(v: &Value) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let items: Vec<&Value> = match v {
        Value::Array(a) => a.iter().collect(),
        Value::Null => vec![],
        other => vec![other],
    };
    items
        .into_iter()
        .map(as_text)
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.to_lowercase()))
        .collect()
}

fn normalize(v: &Value) -> Value {
    let summary = match &v["summary"] {
        Value::Array(a) => a.iter().map(as_text).collect::<Vec<_>>().join(" "),
        other => as_text(other),
    };
    json!({
        "summary": summary,
        "topics": list_of(&v["topics"]),
        "decisions": list_of(&v["decisions"]),
        "actions": list_of(&v["actions"]),
        "next_steps": list_of(&v["next_steps"]),
    })
}

// ─── Repli sans IA : résumé extractif (phrases réelles de la transcription) ─

fn stopwords() -> std::collections::HashSet<&'static str> {
    "alors ainsi aussi avec avait avez avoir bien cela celui cette ceux chez comme dans donc elle elles encore entre \
     était étaient être faire fait faut leur leurs mais même moins notre nous pour puis quand quel quelle quoi sans \
     sera sont sous tout tous très votre vous voilà peut peux plus pas par sur ses son ces est les des une aux \
     oui non voir dit ils ça cela ceci ici où car comme déjà donc ensuite après avant encore peu beaucoup chose choses \
     bonjour merci d'accord parfait notamment vraiment justement effectivement exactement aujourd"
        .split_whitespace()
        .collect()
}

fn contains_any(s: &str, keys: &[&str]) -> bool {
    let l = s.to_lowercase();
    keys.iter().any(|k| l.contains(k))
}

fn extractive(title: &str, segs: &[Segment], note: &str) -> Value {
    let mut sentences: Vec<String> = Vec::new();
    for s in segs {
        let mut cur = String::new();
        for ch in s.text.chars() {
            cur.push(ch);
            if matches!(ch, '.' | '!' | '?' | '…') {
                let t = cur.trim().to_string();
                if t.chars().count() > 3 {
                    sentences.push(t);
                }
                cur.clear();
            }
        }
        let t = cur.trim().to_string();
        if t.chars().count() > 3 {
            sentences.push(t);
        }
    }
    let stop = stopwords();
    let tokens = |s: &str| -> Vec<String> {
        s.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.chars().count() >= 4 && !stop.contains(*w))
            .map(|w| w.to_string())
            .collect()
    };
    let mut freq: HashMap<String, usize> = HashMap::new();
    for s in &sentences {
        for w in tokens(s) {
            *freq.entry(w).or_default() += 1;
        }
    }
    // phrases les plus « porteuses » (fréquence des mots, pénalité de longueur), remises dans l'ordre
    let mut scored: Vec<(usize, f64)> = sentences
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let t = tokens(s);
            let sc: f64 = t.iter().map(|w| freq.get(w).copied().unwrap_or(0) as f64).sum::<f64>() / ((t.len().max(4)) as f64).sqrt();
            (i, sc)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut pick: Vec<usize> = scored.iter().take(5).map(|x| x.0).collect();
    pick.sort();
    let clip = |s: &String| -> String {
        let c: String = s.chars().take(240).collect();
        if s.chars().count() > 240 { format!("{c}…") } else { c }
    };
    let dur = segs.last().map(|s| s.end).unwrap_or(0.0);
    let mut summary = format!("Réunion « {title} » d'environ {} min. ", ((dur / 60.0).round() as u32).max(1));
    summary.push_str(&pick.iter().map(|&i| sentences[i].clone()).collect::<Vec<_>>().join(" "));

    let mut kw: Vec<(&String, &usize)> = freq.iter().filter(|(_, n)| **n >= 2).collect();
    kw.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let topics: Vec<String> = kw.iter().take(6).map(|(w, n)| format!("{} ({} mentions)", w, n)).collect();

    let grab = |keys: &[&str]| -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        sentences
            .iter()
            .filter(|s| contains_any(s, keys))
            .map(clip)
            .filter(|s| seen.insert(s.to_lowercase()))
            .take(10)
            .collect()
    };
    json!({
        "summary": summary,
        "topics": topics,
        "decisions": grab(&["décid", "décision", "validé", "on valide", "on retient", "convenu", "on part sur", "c'est acté", "acté"]),
        "actions": grab(&["je vais", "je m'en charge", "je te", "tu peux", "tu envoies", "tu planifies", "il faut", "il faudra", "doit ", "devra", "d'ici", "à faire", "action", "peux-tu", "pourrais-tu"]),
        "next_steps": grab(&["prochaine réunion", "semaine prochaine", "on en reparle", "reporté", "prochain point", "la prochaine fois", "à la prochaine"]),
        "mode": "extractif",
        "note": note,
    })
}

// ─── Génération ────────────────────────────────────────────────────────────

fn set_progress(app: &AppHandle, id: &str, txt: &str) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        m.report_progress = txt.to_string();
    }
}

fn load_text(p: u8) -> String {
    if p < 100 { format!("Lecture du modèle sur le disque… {p} %") } else { "Démarrage de l'IA locale…".into() }
}

/// Avancement de la lecture du modèle : texte « N % » et barre (de 3 à 10 % de la génération).
fn load_progress<'a>(app: &'a AppHandle, id: &'a str) -> impl Fn(u8) + 'a {
    move |p| {
        set_progress(app, id, &load_text(p));
        set_pct(app, id, 3.0 + p as f32 * 0.07);
    }
}

fn set_report_hw(app: &AppHandle, id: &str, hw: &str) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        m.report_hw = hw.to_string();
    }
}

fn set_pct(app: &AppHandle, id: &str, pct: f32) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        // ne recule jamais
        m.report_pct = m.report_pct.max(pct.clamp(0.0, 99.0) as u8);
    }
}

fn build_context(title: &str, brief: Option<&Value>, glossary: &[String]) -> String {
    let mut c = format!("Titre de la réunion : {title}\n");
    if let Some(b) = brief {
        if let Some(o) = b["objective"].as_str() {
            c.push_str(&format!("Objectif prévu : {o}\n"));
        }
        if let Some(a) = b["agenda"].as_array() {
            let items: Vec<String> = a.iter().filter_map(|x| x["title"].as_str().map(String::from)).collect();
            if !items.is_empty() {
                c.push_str(&format!("Ordre du jour prévu : {}\n", items.join(" ; ")));
            }
        }
    }
    if !glossary.is_empty() {
        c.push_str(&format!("Glossaire : {}\n", glossary.join(" ; ")));
    }
    c
}

/// Le chargement du modèle occupe 0-10 %, la rédaction 10-99 % répartis entre les étapes.
fn run_with_backend(app: &AppHandle, id: &str, be: &Backend, ctx: &str, lines: &[String], lang: &str) -> R<Value> {
    let parts = chunk(lines, be.chunk_chars);
    let n = parts.len();
    let steps = if n <= 1 { 1 } else { n + 1 } as f32;
    let at = |step: usize, f: f32| 10.0 + 89.0 * (step as f32 + f) / steps;
    set_pct(app, id, 10.0);
    set_report_hw(app, id, &be.hw);
    if n <= 1 {
        set_progress(app, id, "Rédaction du compte-rendu…");
        let body = format!("{ctx}\nTranscription :\n{}", parts.first().cloned().unwrap_or_default());
        return Ok(normalize(&chat_json(be, &system_full(lang), &body, 1400, &|f| set_pct(app, id, at(0, f)))?));
    }
    let mut notes = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        set_progress(app, id, &format!("Analyse de la partie {}/{}…", i + 1, n));
        let body = format!("{ctx}\nPartie {}/{} de la réunion. Transcription :\n{p}", i + 1, n);
        notes.push(normalize(&chat_json(be, &system_map(lang), &body, 900, &|f| set_pct(app, id, at(i, f)))?));
    }
    set_progress(app, id, "Fusion des parties…");
    let merged = notes
        .iter()
        .enumerate()
        .map(|(i, v)| format!("--- Notes partie {}/{} ---\n{}", i + 1, n, serde_json::to_string(v).unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!("{ctx}\n{merged}");
    Ok(normalize(&chat_json(be, &system_reduce(lang), &body, 1600, &|f| set_pct(app, id, at(n, f)))?))
}

fn produce(app: &AppHandle, id: &str, lang: &str) -> R<Value> {
    let st = app.state::<AppState>();
    let (cfg, title, segs, glossary, brief): (Settings, String, Vec<Segment>, Vec<String>, Option<Value>) = {
        let s = st.store.lock().unwrap();
        let m = s.meetings.iter().find(|m| m.id == id).ok_or("Réunion introuvable")?;
        let brief = m
            .brief_id
            .as_ref()
            .and_then(|bid| s.briefs.iter().find(|b| &b.id == bid))
            .map(|b| b.content.clone());
        (
            s.settings.clone(),
            m.title.clone(),
            m.transcript.clone(),
            s.glossary
                .iter()
                .map(|t| if t.definition.is_empty() { t.term.clone() } else { format!("{} ({})", t.term, t.definition) })
                .collect(),
            brief,
        )
    };
    if segs.is_empty() {
        return Err("La transcription est vide.".into());
    }
    let lines = transcript_lines(&segs);
    let ctx = build_context(&title, brief.as_ref(), &glossary);
    let stamp = |mut v: Value, mode: &str, model: &str, hw: &str| {
        v["mode"] = json!(mode);
        v["lang"] = json!(lang);
        v["model"] = json!(model);
        v["generated_at"] = json!(chrono::Utc::now());
        v["hardware"] = json!(hw);
        v
    };

    // 0) Ollama (modèle choisi sous la forme « ollama:nom »)
    if let Some(name) = cfg.llm_local.strip_prefix("ollama:") {
        set_progress(app, id, "Démarrage d'Ollama…");
        let be = Backend {
            base: OLLAMA_URL.into(),
            key: String::new(),
            model: name.to_string(),
            label: format!("Ollama · {name}"),
            hw: "Ollama (matériel géré par Ollama)".to_string(),
            mode: "ollama",
            kind: "ollama",
            chunk_chars: 16_000,
            timeout: 3600,
        };
        let res = ensure_ollama().and_then(|_| {
            set_progress(app, id, "Chargement du modèle (la première fois peut être long)…");
            run_with_backend(app, id, &be, &ctx, &lines, lang)
        });
        return match res {
            Ok(v) => Ok(stamp(v, be.mode, &be.label, &be.hw)),
            Err(e) => Ok(extractive(&title, &segs, &format!("Ollama a échoué ({e}). Résumé simplifié affiché."))),
        };
    }
    // 0b) GGUF existant sur le disque (ex. modèle déjà téléchargé par Ollama), lancé via llama.cpp
    if let Some(path) = cfg.llm_local.strip_prefix("gguf:") {
        let path = PathBuf::from(path);
        let name = gguf_display_name(&path);
        if !any_engine(&st.mdir) {
            return Ok(extractive(&title, &segs, "Le moteur d'IA locale (llama.cpp) n'est pas installé : téléchargez d'abord un modèle dans Réglages. Résumé simplifié affiché."));
        }
        if !path.exists() {
            return Ok(extractive(&title, &segs, "Le fichier du modèle est introuvable. Résumé simplifié affiché."));
        }
        set_progress(app, id, "Chargement de l'IA locale (gros modèle : plusieurs minutes possibles)…");
        set_pct(app, id, 3.0);
        let attempt = |cfg: &Settings| {
            start_server(&st.mdir, &path, cfg, &load_progress(app, id)).and_then(|srv| {
                let be = Backend {
                    base: format!("http://127.0.0.1:{}/v1", srv.port),
                    key: String::new(),
                    model: "local".into(),
                    label: name.clone(),
                    hw: engine_hw(&st.mdir, cfg),
                    mode: "local",
                    kind: "openai",
                    chunk_chars: 11_000,
                    timeout: 7200,
                };
                let r = run_with_backend(app, id, &be, &ctx, &lines, lang);
                drop(srv);
                r.map(|v| stamp(v, be.mode, &be.label, &be.hw))
            })
        };
        let mut res = attempt(&cfg);
        if res.as_ref().err().map_or(false, |e| engine_crashed(e)) && cfg.llm_device != "cpu_only" {
            // le moteur GPU a été coupé : on retente une fois sur le processeur
            set_progress(app, id, "Moteur GPU interrompu : nouvel essai sur le processeur…");
            let mut cpu = cfg.clone();
            cpu.llm_device = "cpu_only".into();
            res = attempt(&cpu).map_err(|e2| format!("{} (essai sur le processeur : {e2})", res.as_ref().err().cloned().unwrap_or_default()));
        }
        return match res {
            Ok(v) => Ok(v),
            Err(e) => Ok(extractive(&title, &segs, &format!("L'IA locale a échoué ({}). Résumé simplifié affiché.", crash_hint(&e)))),
        };
    }
    // 1) IA locale
    if local_ready(&st.mdir, &cfg.llm_local) {
        let info = model_info(&cfg.llm_local).unwrap();
        set_progress(app, id, "Chargement de l'IA locale…");
        set_pct(app, id, 3.0);
        let attempt = |cfg: &Settings| {
            start_server(&st.mdir, &model_file(&st.mdir, &cfg.llm_local), cfg, &load_progress(app, id)).and_then(|srv| {
                let be = Backend {
                    base: format!("http://127.0.0.1:{}/v1", srv.port),
                    key: String::new(),
                    model: "local".into(),
                    label: info.1.to_string(),
                    hw: engine_hw(&st.mdir, cfg),
                    mode: "local",
                    kind: "openai",
                    chunk_chars: 11_000,
                    timeout: 1800,
                };
                let r = run_with_backend(app, id, &be, &ctx, &lines, lang);
                drop(srv);
                r.map(|v| stamp(v, be.mode, &be.label, &be.hw))
            })
        };
        let mut res = attempt(&cfg);
        if res.as_ref().err().map_or(false, |e| engine_crashed(e)) && cfg.llm_device != "cpu_only" {
            // le moteur GPU a été coupé : on retente une fois sur le processeur
            set_progress(app, id, "Moteur GPU interrompu : nouvel essai sur le processeur…");
            let mut cpu = cfg.clone();
            cpu.llm_device = "cpu_only".into();
            res = attempt(&cpu).map_err(|e2| format!("{} (essai sur le processeur : {e2})", res.as_ref().err().cloned().unwrap_or_default()));
        }
        match res {
            Ok(v) => return Ok(v),
            Err(e) => {
                return Ok(extractive(&title, &segs, &format!("L'IA locale a échoué ({}). Résumé simplifié affiché.", crash_hint(&e))));
            }
        }
    }
    // 2) API distante
    if !cfg.llm_base_url.trim().is_empty() {
        let be = Backend {
            base: cfg.llm_base_url.clone(),
            key: cfg.llm_api_key.clone(),
            model: if cfg.llm_model.is_empty() { "gpt-4o-mini".into() } else { cfg.llm_model.clone() },
            label: format!("API · {}", if cfg.llm_model.is_empty() { "gpt-4o-mini" } else { &cfg.llm_model }),
            hw: "Serveur distant (API)".to_string(),
            mode: "api",
            kind: "openai",
            chunk_chars: 40_000,
            timeout: 240,
        };
        return match run_with_backend(app, id, &be, &ctx, &lines, lang) {
            Ok(v) => Ok(stamp(v, be.mode, &be.label, &be.hw)),
            Err(e) => Ok(extractive(&title, &segs, &format!("L'API a échoué ({e}). Résumé simplifié affiché."))),
        };
    }
    // 3) aucun modèle : résumé extractif
    Ok(extractive(
        &title,
        &segs,
        "Résumé automatique simplifié (aucune IA de rédaction installée). Installez un modèle dans Réglages › Compte-rendu pour obtenir un compte-rendu rédigé.",
    ))
}

#[tauri::command]
pub fn generate_report(app: AppHandle, st: State<AppState>, id: String, lang: Option<String>) -> R<()> {
    {
        let mut s = st.store.lock().unwrap();
        let m = s.meetings.iter_mut().find(|m| m.id == id).ok_or("Réunion introuvable")?;
        if m.status != "ready" {
            return Err("La transcription n'est pas encore terminée.".into());
        }
        if m.report_state == "generating" {
            return Err("Un compte-rendu est déjà en cours de génération.".into());
        }
        m.report_state = "generating".into();
        m.report_error = None;
        m.report_progress = "Démarrage…".into();
        m.report_pct = 0;
        st.save(&s)?;
    }
    let lang = {
        let s = st.store.lock().unwrap();
        lang.filter(|l| !l.is_empty()).unwrap_or_else(|| s.settings.report_lang.clone())
    };
    std::thread::spawn(move || {
        let res = produce(&app, &id, &lang);
        let st = app.state::<AppState>();
        let mut s = st.store.lock().unwrap();
        if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
            m.report_progress.clear();
            m.report_pct = 0;
            m.report_hw.clear();
            match res {
                Ok(v) => {
                    m.report = Some(v);
                    m.report_state = String::new();
                }
                Err(e) => {
                    m.report_state = "error".into();
                    m.report_error = Some(e);
                }
            }
        }
        let _ = st.save(&s);
    });
    Ok(())
}
