//! Sauvegarde et restauration des réunions.
//!
//! La sauvegarde est un fichier .zip : `manifest.json` (version, date, comptes), `data.json` (réunions, préparations,
//! glossaire, types de réunion) et, au choix, les fichiers audio dans `audio/`. Les réglages ne sont pas sauvegardés
//! (ils dépendent de la machine et contiennent la clé d'API). La restauration est une fusion : ce qui existe déjà
//! n'est jamais modifié ni remplacé.

use crate::{AppState, Brief, Kind, Meeting, Term, R};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use tauri::State;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const FORMAT: u32 = 1;
/// Taille maximale acceptée pour `data.json` à la lecture (protège d'un fichier forgé).
const MAX_DATA: u64 = 512 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Data {
    meetings: Vec<Meeting>,
    #[serde(default)]
    briefs: Vec<Brief>,
    #[serde(default)]
    glossary: Vec<Term>,
    #[serde(default)]
    kinds: Vec<Kind>,
}

/// Nom de fichier sûr : dernier élément du chemin, sans rien qui permette de sortir du dossier.
fn safe_name(entry: &str) -> Option<String> {
    let n = Path::new(entry).file_name()?.to_str()?;
    if n.is_empty() || n.starts_with('.') || n.contains(['/', '\\', ':']) { None } else { Some(n.to_string()) }
}

fn opts(method: CompressionMethod, size: u64) -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(method).large_file(size >= 3_900_000_000)
}

fn add_file(z: &mut ZipWriter<BufWriter<fs::File>>, src: &Path, name: &str) -> R<()> {
    let size = fs::metadata(src).map_err(|e| format!("{}: {e}", src.display()))?.len();
    // le WAV se compresse bien ; mp3, m4a, ogg… sont déjà compressés
    let wav = src.extension().and_then(|x| x.to_str()).is_some_and(|x| x.eq_ignore_ascii_case("wav"));
    let m = if wav { CompressionMethod::Deflated } else { CompressionMethod::Stored };
    z.start_file(name, opts(m, size)).map_err(|e| e.to_string())?;
    let mut f = fs::File::open(src).map_err(|e| format!("{}: {e}", src.display()))?;
    std::io::copy(&mut f, z).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_backup(tmp: &Path, data: &Data, files: &[(PathBuf, String)], with_audio: bool) -> R<()> {
    let f = fs::File::create(tmp).map_err(|e| e.to_string())?;
    let mut z = ZipWriter::new(BufWriter::new(f));
    let manifest = json!({
        "app": "verbatim", "format": FORMAT, "app_version": env!("CARGO_PKG_VERSION"),
        "created_at": chrono::Utc::now(), "meetings": data.meetings.len(), "audio": with_audio,
    });
    z.start_file("manifest.json", opts(CompressionMethod::Deflated, 0)).map_err(|e| e.to_string())?;
    z.write_all(serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;
    z.start_file("data.json", opts(CompressionMethod::Deflated, 0)).map_err(|e| e.to_string())?;
    z.write_all(serde_json::to_string(data).map_err(|e| e.to_string())?.as_bytes()).map_err(|e| e.to_string())?;
    for (src, name) in files {
        add_file(&mut z, src, name)?;
    }
    z.finish().map_err(|e| e.to_string())?.flush().map_err(|e| e.to_string())
}

/// Crée la sauvegarde `dest` ; `with_audio` ajoute les fichiers audio (volumineux). Renvoie `{ meetings, audio, bytes }`.
#[tauri::command(async)]
pub fn backup_export(st: State<AppState>, dest: String, with_audio: bool) -> R<Value> {
    // copie des données sous verrou, puis écriture sans le garder (l'audio peut prendre du temps)
    let mut data = {
        let s = st.store.lock().unwrap();
        Data { meetings: s.meetings.clone(), briefs: s.briefs.clone(), glossary: s.glossary.clone(), kinds: s.kinds.clone() }
    };
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    let mut used = HashSet::new();
    for m in data.meetings.iter_mut() {
        let Some(p) = m.audio_path.take() else { continue };
        let src = PathBuf::from(&p);
        let name = src.file_name().and_then(|x| x.to_str()).map(str::to_string);
        if let (true, true, Some(name)) = (with_audio, src.is_file(), name) {
            if used.insert(name.clone()) {
                files.push((src.clone(), format!("audio/{name}")));
                let play = crate::engine::play_copy(&p);
                if play.is_file() {
                    if let Some(pn) = play.file_name().and_then(|x| x.to_str()) {
                        files.push((play.clone(), format!("audio/{pn}")));
                    }
                }
            }
            m.audio_path = Some(format!("audio/{}", src.file_name().and_then(|x| x.to_str()).unwrap_or_default()));
        }
    }
    let dest = PathBuf::from(dest);
    let tmp = dest.with_extension("zip.tmp");
    let res = write_backup(&tmp, &data, &files, with_audio);
    if let Err(e) = res {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    fs::rename(&tmp, &dest).map_err(|e| { let _ = fs::remove_file(&tmp); e.to_string() })?;
    let bytes = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    Ok(json!({ "meetings": data.meetings.len(), "audio": files.len(), "bytes": bytes }))
}

fn open(path: &str) -> R<ZipArchive<fs::File>> {
    let f = fs::File::open(path).map_err(|e| e.to_string())?;
    ZipArchive::new(f).map_err(|_| "Ce fichier n'est pas une sauvegarde Verbatim.".to_string())
}

fn read_json<T: serde::de::DeserializeOwned>(z: &mut ZipArchive<fs::File>, name: &str) -> R<T> {
    let f = z.by_name(name).map_err(|_| "Ce fichier n'est pas une sauvegarde Verbatim.".to_string())?;
    let mut buf = Vec::new();
    f.take(MAX_DATA).read_to_end(&mut buf).map_err(|e| e.to_string())?;
    serde_json::from_slice(&buf).map_err(|_| "La sauvegarde est illisible ou endommagée.".to_string())
}

fn check_format(z: &mut ZipArchive<fs::File>) -> R<Value> {
    let m: Value = read_json(z, "manifest.json")?;
    if m["app"] != "verbatim" {
        return Err("Ce fichier n'est pas une sauvegarde Verbatim.".into());
    }
    if m["format"].as_u64().unwrap_or(0) > FORMAT as u64 {
        return Err("Cette sauvegarde vient d'une version plus récente de Verbatim : mettez l'application à jour.".into());
    }
    Ok(m)
}

/// Contenu d'une sauvegarde, avant restauration : `{ created_at, app_version, audio, meetings, new_meetings, ... }`.
#[tauri::command(async)]
pub fn backup_inspect(st: State<AppState>, path: String) -> R<Value> {
    let mut z = open(&path)?;
    let man = check_format(&mut z)?;
    let data: Data = read_json(&mut z, "data.json")?;
    let s = st.store.lock().unwrap();
    let have: HashSet<&str> = s.meetings.iter().map(|m| m.id.as_str()).collect();
    let have_b: HashSet<&str> = s.briefs.iter().map(|b| b.id.as_str()).collect();
    Ok(json!({
        "created_at": man["created_at"], "app_version": man["app_version"], "audio": man["audio"],
        "meetings": data.meetings.len(),
        "new_meetings": data.meetings.iter().filter(|m| !have.contains(m.id.as_str())).count(),
        "briefs": data.briefs.len(),
        "new_briefs": data.briefs.iter().filter(|b| !have_b.contains(b.id.as_str())).count(),
    }))
}

/// Fusionne la sauvegarde dans les données actuelles. Renvoie `{ meetings, briefs, terms }` (éléments ajoutés).
#[tauri::command(async)]
pub fn backup_restore(st: State<AppState>, path: String) -> R<Value> {
    let mut z = open(&path)?;
    check_format(&mut z)?;
    let data: Data = read_json(&mut z, "data.json")?;
    let audio_dir = st.dir.join("audio");
    fs::create_dir_all(&audio_dir).map_err(|e| e.to_string())?;

    let have: HashSet<String> = st.store.lock().unwrap().meetings.iter().map(|m| m.id.clone()).collect();
    let fresh: Vec<Meeting> = data.meetings.into_iter().filter(|m| !have.contains(&m.id)).collect();

    // fichiers audio des seules réunions ajoutées (le nom contient l'identifiant : pas de collision entre réunions)
    let mut wanted: HashSet<String> = HashSet::new();
    for m in &fresh {
        if let Some(n) = m.audio_path.as_deref().and_then(safe_name) {
            wanted.insert(format!("audio/{n}"));
            wanted.insert(format!("audio/{}", crate::engine::play_copy(&n).to_string_lossy()));
        }
    }
    for i in 0..z.len() {
        let mut e = z.by_index(i).map_err(|e| e.to_string())?;
        if e.is_dir() || !wanted.contains(e.name()) { continue; }
        let Some(name) = safe_name(e.name()) else { continue };
        let dst = audio_dir.join(name);
        let tmp = dst.with_extension("part");
        let r = fs::File::create(&tmp).and_then(|mut f| std::io::copy(&mut e, &mut f).map(|_| ()));
        match r {
            Ok(()) => fs::rename(&tmp, &dst).map_err(|e| e.to_string())?,
            Err(err) => { let _ = fs::remove_file(&tmp); return Err(format!("Écriture de l'audio impossible : {err}")); }
        }
    }

    let mut s = st.store.lock().unwrap();
    let mut added = 0;
    for mut m in fresh {
        // l'audio n'existe que s'il a été restauré ; les travaux en cours à la sauvegarde ne reprennent pas seuls
        m.audio_path = m.audio_path.as_deref().and_then(safe_name).map(|n| audio_dir.join(n)).filter(|p| p.is_file())
            .map(|p| p.to_string_lossy().into_owned());
        if matches!(m.status.as_str(), "queued" | "converting" | "transcribing" | "diarizing") {
            m.status = if m.transcript.is_empty() { "error".into() } else { "ready".into() };
            if m.status == "error" { m.error = Some("Transcription interrompue avant la sauvegarde. Relancez-la.".into()); }
        }
        if m.status == "needs_engine" && m.audio_path.is_none() { m.status = "error".into(); m.error = Some("Fichier audio absent de la sauvegarde.".into()); }
        for state in [&mut m.report_state, &mut m.translate_state] { state.clear(); }
        m.report_progress.clear();
        m.report_pct = 0;
        m.report_ckpt = None;
        m.translate_pct = 0;
        s.meetings.push(m);
        added += 1;
    }
    let have_b: HashSet<String> = s.briefs.iter().map(|b| b.id.clone()).collect();
    let nb: Vec<Brief> = data.briefs.into_iter().filter(|b| !have_b.contains(&b.id)).collect();
    let briefs = nb.len();
    s.briefs.extend(nb);
    let have_t: HashSet<String> = s.glossary.iter().map(|t| t.term.to_lowercase()).collect();
    let nt: Vec<Term> = data.glossary.into_iter().filter(|t| !have_t.contains(&t.term.to_lowercase())).collect();
    let terms = nt.len();
    s.glossary.extend(nt);
    // liste vide = types par défaut (voir `list_kinds`) : on part d'eux pour ne pas les masquer en ajoutant les nôtres
    let mut kinds = if s.kinds.is_empty() { crate::default_kinds() } else { s.kinds.clone() };
    let have_k: HashSet<String> = kinds.iter().map(|k| k.id.clone()).collect();
    let nk: Vec<Kind> = data.kinds.into_iter().filter(|k| !have_k.contains(&k.id)).collect();
    if !nk.is_empty() {
        kinds.extend(nk);
        s.kinds = kinds;
    }
    st.save(&s)?;
    Ok(json!({ "meetings": added, "briefs": briefs, "terms": terms }))
}
