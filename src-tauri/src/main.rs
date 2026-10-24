#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{Manager, State};

mod engine;
mod live;
mod llm;
use llm::{generate_report, llm_install_engine, llm_rocm_check, local_gguf_list, llm_remove_file, llm_install, llm_remove, ollama_start, ollama_status};
mod models;
mod procs;
mod recorder;
mod resample;
mod translate;
mod update;
use recorder::{rec_devices, rec_pause, rec_set_gain, rec_start, rec_status};
use engine::{engine_install, download_cancel, engine_install_gpu, engine_remove, engine_status, retry_transcription, spawn_transcription};

pub mod engine_paths {
    use std::path::PathBuf;
    use std::sync::OnceLock;
    /// Dossier « redist » livré avec l'installateur (moteurs prêts à l'emploi) ; renseigné au démarrage.
    pub static REDIST: OnceLock<PathBuf> = OnceLock::new();
    /// `dir/rel` si présent (installé par l'utilisateur), sinon `redist/rel` (livré avec l'application).
    pub fn pick(dir: &PathBuf, rel: &str) -> PathBuf {
        let p = dir.join(rel);
        if !p.exists() {
            if let Some(q) = REDIST.get().map(|r| r.join(rel)).filter(|q| q.exists()) {
                return q;
            }
        }
        p
    }
    pub fn engine_bin(dir: &PathBuf) -> PathBuf { pick(dir, "engine\\whisper-cli.exe") }
    pub fn engine_gpu_bin(dir: &PathBuf) -> PathBuf { dir.join("engine-gpu").join("whisper-cli.exe") }
    pub fn engine_rocm_bin(dir: &PathBuf) -> PathBuf { pick(dir, "engine-rocm/whisper-cli.exe") }
    pub fn engine_rocm_server(dir: &PathBuf) -> PathBuf { pick(dir, "engine-rocm/whisper-server.exe") }
    pub fn model_path(dir: &PathBuf, m: &str) -> PathBuf { dir.join("models").join(format!("ggml-{m}.bin")) }
    pub fn engine_ready(dir: &PathBuf, model: &str) -> bool { engine_bin(dir).exists() && model_path(dir, model).exists() }
}
use engine_paths::engine_ready;

// ─── Modèle ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Segment {
    pub speaker: String,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Meeting {
    pub id: String,
    title: String,
    created_at: DateTime<Utc>,
    pub audio_path: Option<String>,
    brief_id: Option<String>,
    /// "audio" ou "transcript" (importé)
    source: String,
    /// queued | converting | transcribing | ready | error | needs_engine
    #[serde(default = "ready_status")]
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    pub transcript: Vec<Segment>,
    report: Option<Value>,
    /// "" | generating | error
    #[serde(default)]
    pub report_state: String,
    #[serde(default)]
    pub report_error: Option<String>,
    #[serde(default)]
    pub report_progress: String,
    /// avancement de la rédaction du compte-rendu, 0 à 100
    #[serde(default)]
    pub report_pct: u8,
    /// avancement de la transcription Whisper, 0 à 100 (non conservé)
    #[serde(default)]
    pub transcribe_pct: u8,
    /// matériel utilisé pour la transcription et pour la rédaction du compte-rendu
    #[serde(default)]
    pub transcribe_hw: String,
    #[serde(default)]
    pub report_hw: String,
    /// traductions de la transcription, par langue (mêmes horodatages que l'original)
    #[serde(default)]
    pub translations: std::collections::HashMap<String, Vec<Segment>>,
    /// "" | translating | error
    #[serde(default)]
    pub translate_state: String,
    #[serde(default)]
    pub translate_error: Option<String>,
    #[serde(default)]
    pub translate_lang: String,
    #[serde(default)]
    pub translate_pct: u8,
    /// matériel utilisé pour la traduction en cours
    #[serde(default)]
    pub translate_hw: String,
    /// modèle de la traduction en cours
    #[serde(default)]
    pub translate_label: String,
    /// modèle (et matériel) qui a produit chaque traduction, par langue
    #[serde(default)]
    pub translation_models: std::collections::HashMap<String, String>,
    deleted: bool,
}

impl Meeting {
    fn new(id: String, title: String, created_at: DateTime<Utc>, audio_path: Option<String>, source: &str, status: &str, transcript: Vec<Segment>) -> Meeting {
        Meeting {
            id, title, created_at, audio_path, brief_id: None, source: source.into(), status: status.into(),
            error: None, transcript, report: None, report_state: String::new(), report_error: None,
            report_progress: String::new(), report_pct: 0, transcribe_pct: 0, transcribe_hw: String::new(),
            report_hw: String::new(), translations: Default::default(), translate_state: String::new(),
            translate_error: None, translate_lang: String::new(), translate_pct: 0, translate_hw: String::new(), translate_label: String::new(), translation_models: Default::default(), deleted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Brief {
    id: String,
    subject: String,
    kind: String,
    themes: Vec<String>,
    content: Value,
    created_at: DateTime<Utc>,
    deleted: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Term {
    pub term: String,
    definition: String,
}

fn ready_status() -> String { "ready".into() }
fn default_model() -> String { "base".into() }
fn default_lang() -> String { "fr".into() }

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default)]
    llm_base_url: String,
    #[serde(default)]
    llm_api_key: String,
    #[serde(default)]
    llm_model: String,
    /// modèle d'IA locale pour les comptes-rendus ("" = aucun)
    #[serde(default)]
    pub llm_local: String,
    #[serde(default = "default_model")]
    pub whisper_model: String,
    #[serde(default = "default_lang")]
    pub whisper_lang: String,
    /// calcul de la transcription : "auto" (GPU si utilisable, repli CPU) ou "cpu"
    #[serde(default = "default_whisper_device")]
    pub whisper_device: String,
    /// moteur de l'IA de rédaction : "cpu" | "vulkan" | "rocm"
    #[serde(default = "default_device")]
    pub llm_device: String,
    /// dossier (site-packages) contenant le runtime ROCm ; vide = détection automatique
    #[serde(default)]
    pub rocm_root: String,
    /// installe seule les nouvelles versions publiées sur GitHub
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// langue de rédaction du compte-rendu : "auto" (celle de la transcription) ou un code (fr, en, es…)
    #[serde(default = "default_auto")]
    pub report_lang: String,
    /// modèle de traduction ("" = celui du compte-rendu)
    #[serde(default)]
    pub translate_model: String,
    /// calcul de la traduction : "auto" (GPU si possible) ou "cpu_only"
    #[serde(default = "default_auto")]
    pub translate_device: String,
}
fn default_auto() -> String { "auto".into() }
fn default_true() -> bool { true }
fn default_whisper_device() -> String { "auto".into() }
fn default_device() -> String { "auto".into() }

impl Default for Settings {
    fn default() -> Self {
        Settings { llm_base_url: String::new(), llm_api_key: String::new(), llm_model: String::new(),
                   llm_local: String::new(), whisper_model: default_model(), whisper_lang: default_lang(), whisper_device: default_whisper_device(),
                   llm_device: default_device(), rocm_root: String::new(), auto_update: true,
                   report_lang: default_auto(), translate_model: String::new(), translate_device: default_auto() }
    }
}

/// Type de réunion personnalisable (préparations) : `prompt` oriente l'IA.
#[derive(Serialize, Deserialize, Clone)]
pub struct Kind {
    id: String,
    label: String,
    #[serde(default)]
    prompt: String,
}

fn default_kinds() -> Vec<Kind> {
    [
        ("general", "Réunion générale", "Tu prépares une réunion."),
        ("one_on_one", "Entretien 1:1", "Tu prépares un entretien individuel (1:1)."),
        ("project", "Point projet", "Tu prépares un point d'avancement projet."),
        ("committee", "Comité de pilotage", "Tu prépares un comité de pilotage."),
        ("brainstorm", "Brainstorming", "Tu prépares un atelier de brainstorming."),
    ]
    .iter()
    .map(|(i, l, p)| Kind { id: i.to_string(), label: l.to_string(), prompt: p.to_string() })
    .collect()
}

#[derive(Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    kinds: Vec<Kind>,
    pub meetings: Vec<Meeting>,
    briefs: Vec<Brief>,
    pub glossary: Vec<Term>,
    pub settings: Settings,
}

struct AppState {
    dir: PathBuf,
    /// Dossier des moteurs et modèles d'IA : à côté de l'application (« ia »).
    mdir: PathBuf,
    store: Mutex<Store>,
    /// Sérialise les transcriptions (une à la fois).
    job_lock: Mutex<()>,
    rec: recorder::RecState,
}

impl AppState {
    pub fn save(&self, s: &Store) -> Result<(), String> {
        let txt = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
        // écriture atomique : fichier temporaire puis renommage (une coupure ne tronque pas le magasin)
        let tmp = self.dir.join("store.json.tmp");
        fs::write(&tmp, txt).map_err(|e| e.to_string())?;
        fs::rename(&tmp, self.dir.join("store.json")).map_err(|e| e.to_string())
    }
}

pub type R<T> = Result<T, String>;

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ─── Statut de transcription ───────────────────────────────────────────

fn stage(m: &Meeting) -> (&'static str, &'static str, u8) {
    match m.status.as_str() {
        "queued" => ("queued", "En attente", 5),
        "converting" => ("converting", "Préparation de l'audio", 0),
        "transcribing" => ("transcribing", "Transcription en cours", m.transcribe_pct),
        "error" => ("error", "Échec", 100),
        "needs_engine" => ("needs_engine", "Moteur Whisper requis", 0),
        _ => ("ready", "Prête", 100),
    }
}

// ─── LLM optionnel (API compatible OpenAI) ─────────────────────────────

fn llm_json(cfg: &Settings, system: &str, user: &str) -> Option<Value> {
    if cfg.llm_base_url.trim().is_empty() {
        return None;
    }
    let url = format!("{}/chat/completions", cfg.llm_base_url.trim_end_matches('/'));
    let model = if cfg.llm_model.is_empty() { "gpt-4o-mini" } else { &cfg.llm_model };
    let body = json!({
        "model": model,
        "response_format": {"type": "json_object"},
        "messages": [{"role":"system","content":system},{"role":"user","content":user}]
    });
    let resp: Value = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", cfg.llm_api_key))
        .timeout(std::time::Duration::from_secs(90))
        .send_json(body)
        .ok()?
        .into_json()
        .ok()?;
    let txt = resp["choices"][0]["message"]["content"].as_str()?;
    serde_json::from_str(txt).ok()
}

fn make_brief(cfg: &Settings, subject: &str, system: &str, themes: &[String]) -> Value {
    llm_json(
        cfg,
        &format!(
            "{system} Réponds en JSON : objective (string), agenda (liste de {{title, minutes}}), \
             questions (liste de strings)."
        ),
        &format!("Sujet : {subject}\nThèmes : {}", themes.join(", ")),
    )
    .unwrap_or_else(|| {
        let ag: Vec<Value> = if themes.is_empty() {
            vec![json!({"title":"Point d'avancement","minutes":15})]
        } else {
            themes.iter().map(|t| json!({"title": t, "minutes": 10})).collect()
        };
        json!({
            "objective": format!("Préparer la réunion « {subject} »."),
            "agenda": ag,
            "questions": ["Quels sont les points bloquants ?", "Quelles décisions sont attendues ?"]
        })
    })
}

// ─── Vues sérialisées vers l'UI ─────────────────────────────────────────

fn meeting_view(m: &Meeting) -> Value {
    let (key, label, pct) = stage(m);
    let mut tl: Vec<&String> = m.translations.keys().collect();
    tl.sort();
    json!({
        "id": m.id, "title": m.title, "created_at": m.created_at, "source": m.source,
        "brief_id": m.brief_id, "has_audio": m.audio_path.is_some(), "audio_path": m.audio_path,
        "play_path": m.audio_path.as_deref().map(engine::play_path),
        "stage": key, "stage_label": label, "progress": pct,
        "segments": m.transcript.len(), "error": m.error,
        "report_state": m.report_state, "report_error": m.report_error, "report_progress": m.report_progress, "report_pct": m.report_pct, "transcribe_hw": m.transcribe_hw, "report_hw": m.report_hw,
        "translate_state": m.translate_state, "translate_error": m.translate_error, "translate_lang": m.translate_lang, "translate_pct": m.translate_pct, "translate_hw": m.translate_hw, "translate_label": m.translate_label, "translation_models": m.translation_models,
        "translation_langs": tl
    })
}

// ─── Commandes : réunions ───────────────────────────────────────────────

#[tauri::command]
fn list_meetings(st: State<AppState>, trashed: bool) -> R<Vec<Value>> {
    let s = st.store.lock().unwrap();
    let mut v: Vec<&Meeting> = s.meetings.iter().filter(|m| m.deleted == trashed).collect();
    v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(v.into_iter().map(meeting_view).collect())
}

#[tauri::command(async)]
fn import_audio(app: tauri::AppHandle, st: State<AppState>, paths: Vec<String>) -> R<usize> {
    let installed = engine_ready(&st.mdir, &st.store.lock().unwrap().settings.whisper_model);
    let mut ids = Vec::new();
    {
        let mut s = st.store.lock().unwrap();
        let audio_dir = st.dir.join("audio");
        fs::create_dir_all(&audio_dir).map_err(|e| e.to_string())?;
        for p in paths {
            let src = PathBuf::from(&p);
            let name = src.file_stem().and_then(|x| x.to_str()).unwrap_or("Réunion").to_string();
            let ext = src.extension().and_then(|x| x.to_str()).unwrap_or("bin");
            let id = new_id();
            let dst = audio_dir.join(format!("{id}.{ext}"));
            fs::copy(&src, &dst).map_err(|e| format!("{p}: {e}"))?;
            s.meetings.push(Meeting::new(id.clone(), name, Utc::now(), Some(dst.to_string_lossy().into()), "audio",
                if installed { "queued" } else { "needs_engine" }, vec![]));
            ids.push(id);
        }
        st.save(&s)?;
    }
    if installed { for id in &ids { spawn_transcription(app.clone(), id.clone()); } }
    Ok(ids.len())
}

#[tauri::command(async)]
fn rec_stop(app: tauri::AppHandle, st: State<AppState>, title: Option<String>) -> R<String> {
    let id = new_id();
    let audio_dir = st.dir.join("audio");
    fs::create_dir_all(&audio_dir).map_err(|e| e.to_string())?;
    let dst = audio_dir.join(format!("{id}.wav"));
    recorder::finish(&st, &dst)?;
    let installed = engine_ready(&st.mdir, &st.store.lock().unwrap().settings.whisper_model);
    {
        let mut s = st.store.lock().unwrap();
        let title = title.filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| format!("Enregistrement du {}", chrono::Local::now().format("%d/%m/%Y %H:%M")));
        s.meetings.push(Meeting::new(id.clone(), title, Utc::now(), Some(dst.to_string_lossy().into()), "audio",
            if installed { "queued" } else { "needs_engine" }, vec![]));
        st.save(&s)?;
    }
    if installed { spawn_transcription(app, id.clone()); }
    Ok(id)
}

fn parse_transcript(text: &str) -> Vec<Segment> {
    // Accepte « Locuteur : texte » ; ignore numéros SRT, horodatages et WEBVTT.
    let mut out = Vec::new();
    let mut t = 0.0;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("WEBVTT") || line.contains("-->") || line.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let (speaker, body) = match line.split_once(':') {
            Some((a, b)) if a.chars().count() <= 30 && !b.trim().is_empty() => (a.trim(), b.trim()),
            _ => ("Intervenant", line),
        };
        out.push(Segment { speaker: speaker.into(), start: t, end: t + 5.0, text: body.into() });
        t += 5.0;
    }
    out
}

#[tauri::command(async)]
fn import_transcript(st: State<AppState>, path: String) -> R<String> {
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let segs = parse_transcript(&text);
    if segs.is_empty() {
        return Err("Aucun texte exploitable dans ce fichier.".into());
    }
    let name = PathBuf::from(&path).file_stem().and_then(|x| x.to_str()).unwrap_or("Transcription").to_string();
    let mut s = st.store.lock().unwrap();
    let id = new_id();
    s.meetings.push(Meeting::new(id.clone(), name, Utc::now(), None, "transcript", "ready", segs));
    st.save(&s)?;
    Ok(id)
}

#[tauri::command]
fn get_meeting(st: State<AppState>, id: String) -> R<Value> {
    let s = st.store.lock().unwrap();
    let m = s.meetings.iter().find(|m| m.id == id).ok_or("Réunion introuvable")?;
    let mut v = meeting_view(m);
    let ready = stage(m).0 == "ready";
    v["transcript"] = if ready { json!(m.transcript) } else { json!([]) };
    v["report"] = json!(m.report);
    v["translations"] = json!(m.translations);
    Ok(v)
}

#[tauri::command]
fn update_meeting(st: State<AppState>, id: String, title: Option<String>, brief_id: Option<String>, trashed: Option<bool>) -> R<()> {
    let mut s = st.store.lock().unwrap();
    let m = s.meetings.iter_mut().find(|m| m.id == id).ok_or("Réunion introuvable")?;
    if let Some(t) = title { m.title = t; }
    if let Some(b) = brief_id { m.brief_id = if b.is_empty() { None } else { Some(b) }; }
    if let Some(d) = trashed { m.deleted = d; }
    st.save(&s)
}

#[tauri::command]
fn purge_meeting(st: State<AppState>, id: String) -> R<()> {
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter().find(|m| m.id == id) {
        if let Some(p) = &m.audio_path { let _ = fs::remove_file(p); let _ = fs::remove_file(engine::play_copy(p)); }
    }
    s.meetings.retain(|m| m.id != id);
    st.save(&s)
}

fn ts(sec: f64, sep: char) -> String {
    let ms = (sec.fract() * 1000.0) as u32;
    let t = sec as u32;
    format!("{:02}:{:02}:{:02}{}{:03}", t / 3600, t % 3600 / 60, t % 60, sep, ms)
}

/// Titres des sections du compte-rendu (points abordés, décisions, actions, prochaines étapes).
pub fn section_labels(lang: &str) -> [&'static str; 4] {
    match lang {
        "en" => ["Topics discussed", "Decisions", "Actions", "Next steps"],
        "es" => ["Temas tratados", "Decisiones", "Acciones", "Próximos pasos"],
        "de" => ["Besprochene Punkte", "Entscheidungen", "Aufgaben", "Nächste Schritte"],
        "it" => ["Argomenti trattati", "Decisioni", "Azioni", "Prossimi passi"],
        "pt" => ["Pontos abordados", "Decisões", "Ações", "Próximos passos"],
        "nl" => ["Besproken punten", "Beslissingen", "Acties", "Volgende stappen"],
        _ => ["Points abordés", "Décisions", "Actions", "Prochaines étapes"],
    }
}

#[tauri::command(async)]
fn export_meeting(st: State<AppState>, id: String, format: String, dest: String, lang: Option<String>) -> R<()> {
    let s = st.store.lock().unwrap();
    let m = s.meetings.iter().find(|m| m.id == id).ok_or("Réunion introuvable")?;
    if format == "audio" {
        let src = m.audio_path.as_ref().ok_or("Pas d'audio")?;
        return fs::copy(src, &dest).map(|_| ()).map_err(|e| e.to_string());
    }
    // transcription d'origine, ou traduction si une langue est demandée
    let segs: &Vec<Segment> = match lang.as_deref().filter(|l| !l.is_empty()) {
        Some(l) => m.translations.get(l).ok_or("Cette traduction n'existe pas encore.")?,
        None => &m.transcript,
    };
    let body = match format.as_str() {
        "srt" => segs.iter().enumerate().map(|(i, g)|
            format!("{}\n{} --> {}\n{}: {}\n", i + 1, ts(g.start, ','), ts(g.end, ','), g.speaker, g.text)
        ).collect::<Vec<_>>().join("\n"),
        "vtt" => format!("WEBVTT\n\n{}", segs.iter().map(|g|
            format!("{} --> {}\n{}: {}\n", ts(g.start, '.'), ts(g.end, '.'), g.speaker, g.text)
        ).collect::<Vec<_>>().join("\n")),
        "json" => serde_json::to_string_pretty(segs).map_err(|e| e.to_string())?,
        "report" => {
            let r = m.report.as_ref().ok_or("Aucun compte-rendu généré.")?;
            let list = |k: &str| r[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).map(|x| format!("- {x}")).collect::<Vec<_>>().join("\n")).unwrap_or_default();
            let l = section_labels(r["lang"].as_str().unwrap_or(""));
            format!("# {}\n\n{}\n\n## {}\n{}\n\n## {}\n{}\n\n## {}\n{}\n\n## {}\n{}\n",
                m.title, r["summary"].as_str().unwrap_or(""), l[0], list("topics"), l[1], list("decisions"), l[2], list("actions"), l[3], list("next_steps"))
        }
        _ => segs.iter().map(|g| format!("{} : {}", g.speaker, g.text)).collect::<Vec<_>>().join("\n"),
    };
    fs::write(dest, body).map_err(|e| e.to_string())
}

// ─── Commandes : briefs ─────────────────────────────────────────────────

#[tauri::command]
fn list_briefs(st: State<AppState>, trashed: bool) -> R<Vec<Brief>> {
    let s = st.store.lock().unwrap();
    let mut v: Vec<Brief> = s.briefs.iter().filter(|b| b.deleted == trashed).cloned().collect();
    v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(v)
}

#[tauri::command]
fn list_kinds(st: State<AppState>) -> Vec<Kind> {
    let s = st.store.lock().unwrap();
    if s.kinds.is_empty() { default_kinds() } else { s.kinds.clone() }
}

#[tauri::command]
fn set_kinds(st: State<AppState>, kinds: Vec<Kind>) -> R<()> {
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<Kind> = Vec::new();
    for k in kinds {
        let label = k.label.trim().to_string();
        if label.is_empty() { continue; }
        let mut id = if k.id.trim().is_empty() { new_id() } else { k.id.trim().to_string() };
        while !seen.insert(id.clone()) { id = new_id(); }
        out.push(Kind { id, label, prompt: k.prompt.trim().to_string() });
    }
    if out.is_empty() { return Err("Il faut au moins un type de réunion.".into()); }
    let mut s = st.store.lock().unwrap();
    s.kinds = out;
    st.save(&s)
}

#[tauri::command]
fn create_brief(st: State<AppState>, subject: String, kind: String, themes: Vec<String>) -> R<Brief> {
    let cfg = st.store.lock().unwrap().settings.clone();
    let system = {
        let s = st.store.lock().unwrap();
        let list = if s.kinds.is_empty() { default_kinds() } else { s.kinds.clone() };
        let k = list.iter().find(|k| k.id == kind);
        match k {
            Some(k) if !k.prompt.trim().is_empty() => k.prompt.clone(),
            Some(k) => format!("Tu prépares une réunion de type « {} ».", k.label),
            None => "Tu prépares une réunion.".to_string(),
        }
    };
    let content = make_brief(&cfg, &subject, &system, &themes); // hors verrou
    let b = Brief { id: new_id(), subject, kind, themes, content, created_at: Utc::now(), deleted: false };
    let mut s = st.store.lock().unwrap();
    s.briefs.push(b.clone());
    st.save(&s)?;
    Ok(b)
}

#[tauri::command]
fn update_brief(st: State<AppState>, id: String, subject: Option<String>, content: Option<Value>, trashed: Option<bool>) -> R<()> {
    let mut s = st.store.lock().unwrap();
    let b = s.briefs.iter_mut().find(|b| b.id == id).ok_or("Brief introuvable")?;
    if let Some(x) = subject { b.subject = x; }
    if let Some(x) = content { b.content = x; }
    if let Some(x) = trashed { b.deleted = x; }
    st.save(&s)
}

#[tauri::command]
fn purge_brief(st: State<AppState>, id: String) -> R<()> {
    let mut s = st.store.lock().unwrap();
    s.briefs.retain(|b| b.id != id);
    for m in s.meetings.iter_mut() {
        if m.brief_id.as_deref() == Some(&id) { m.brief_id = None; }
    }
    st.save(&s)
}

// ─── Commandes : glossaire & réglages ───────────────────────────────────

#[tauri::command]
fn get_glossary(st: State<AppState>) -> R<Vec<Term>> {
    Ok(st.store.lock().unwrap().glossary.clone())
}

#[tauri::command]
fn set_glossary(st: State<AppState>, terms: Vec<Term>) -> R<()> {
    let mut s = st.store.lock().unwrap();
    s.glossary = terms.into_iter().filter(|t| !t.term.trim().is_empty()).collect();
    st.save(&s)
}

#[tauri::command]
fn get_settings(st: State<AppState>) -> R<Settings> {
    Ok(st.store.lock().unwrap().settings.clone())
}

#[tauri::command]
fn set_settings(st: State<AppState>, settings: Settings) -> R<()> {
    let mut s = st.store.lock().unwrap();
    s.settings = settings;
    st.save(&s)
}

/// `<dossier de l'exe>/ia` si on peut y écrire (version portable), sinon dans les données de l'app.
fn find_redist(app: &tauri::AppHandle) -> Option<PathBuf> {
    let ok = |d: &PathBuf| d.join("engine").is_dir() || d.join("llm").is_dir();
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        // à côté de l'exe, ou (développement) dans un dossier parent du projet
        cands.extend(exe.ancestors().skip(1).take(6).map(|a| a.join("redist")));
    }
    if let Ok(r) = app.path().resource_dir() {
        cands.push(r.join("redist"));
    }
    cands.into_iter().find(|d| ok(d))
}

fn models_dir(fallback: &PathBuf) -> PathBuf {
    let try_dir = |d: PathBuf| -> Option<PathBuf> {
        fs::create_dir_all(&d).ok()?;
        let probe = d.join(".write-test");
        fs::write(&probe, b"").ok()?;
        let _ = fs::remove_file(&probe);
        Some(d)
    };
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.join("ia")))
        .and_then(try_dir)
        .or_else(|| try_dir(fallback.join("ia")))
        .unwrap_or_else(|| fallback.clone())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            fs::create_dir_all(&dir)?;
            let mut store: Store = fs::read_to_string(dir.join("store.json"))
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_default();
            for mt in store.meetings.iter_mut() {
                if mt.translate_state == "translating" {
                    mt.translate_state = "error".into();
                    mt.translate_error = Some("Traduction interrompue (application fermée). Relancez-la.".into());
                    mt.translate_pct = 0;
                }
                if mt.report_state == "generating" {
                    mt.report_state = "error".into();
                    mt.report_error = Some("Génération interrompue (application fermée). Relancez-la.".into());
                    mt.report_progress.clear();
                    mt.report_pct = 0;
                }
            }
            std::thread::spawn(|| { let _ = engine::gpu_names(); });
            let mdir = models_dir(&dir);
            if let Some(r) = find_redist(app.handle()) {
                let _ = engine_paths::REDIST.set(r);
            }
            app.manage(AppState { dir, mdir, store: Mutex::new(store), job_lock: Mutex::new(()), rec: Mutex::new(None) });
            // reprend les transcriptions interrompues par une fermeture de l'app
            let pending: Vec<String> = app.state::<AppState>().store.lock().unwrap().meetings.iter()
                .filter(|m| matches!(m.status.as_str(), "queued" | "converting" | "transcribing"))
                .map(|m| m.id.clone()).collect();
            // avant de reprendre, on tue les moteurs orphelins d'un lancement précédent (fermeture pendant le chargement de Whisper)
            let (h, mdir_c) = (app.handle().clone(), app.state::<AppState>().mdir.clone());
            std::thread::spawn(move || {
                let mut dirs = vec![mdir_c];
                if let Some(r) = engine_paths::REDIST.get() { dirs.push(r.clone()); }
                procs::kill_orphans(&dirs);
                for id in pending { spawn_transcription(h.clone(), id); }
            });
            // récupère les enregistrements interrompus
            let recovered = recorder::recover(&app.state::<AppState>().dir);
            if !recovered.is_empty() {
                let h = app.handle().clone();
                let st = h.state::<AppState>();
                let ready = engine_ready(&st.mdir, &st.store.lock().unwrap().settings.whisper_model);
                let mut new_ids = Vec::new();
                {
                    let mut g = st.store.lock().unwrap();
                    for (id, wav, when) in recovered {
                        let dt: DateTime<Utc> = when.into();
                        g.meetings.push(Meeting::new(id.clone(),
                            format!("Enregistrement récupéré ({})", dt.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M")),
                            dt, Some(wav.to_string_lossy().into()), "audio",
                            if ready { "queued" } else { "needs_engine" }, vec![]));
                        new_ids.push(id);
                    }
                    let _ = st.save(&g);
                }
                if ready { for id in new_ids { spawn_transcription(h.clone(), id); } }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_meetings, import_audio, import_transcript, get_meeting, generate_report,
            update_meeting, purge_meeting, export_meeting,
            list_briefs, list_kinds, set_kinds, create_brief, update_brief, purge_brief,
            get_glossary, set_glossary, get_settings, set_settings,
            engine_status, engine_install, engine_install_gpu, download_cancel, engine_remove, llm_install, llm_remove, ollama_status, ollama_start, local_gguf_list, llm_install_engine, llm_rocm_check, llm_remove_file, models::models_scan, models::models_import, models::models_dir_info, models::install_lang, retry_transcription,
            rec_start, rec_status, rec_stop, rec_devices, rec_pause, rec_set_gain,
            live::live_prepare, live::live_release,
            translate::translate_meeting, translate::translate_stop, translate::translation_remove,
            update::update_check, update::update_install
        ])
        .build(tauri::generate_context!())
        .expect("erreur au lancement de Verbatim")
        .run(|_, event| {
            if let tauri::RunEvent::Exit = event {
                procs::kill_all();
            }
        });
}
