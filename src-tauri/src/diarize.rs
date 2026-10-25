//! Séparation des intervenants (diarisation) : pyannote-segmentation-3.0 repère qui parle et quand,
//! CAM++ (3D-Speaker) donne l'empreinte de chaque voix, sherpa-onnx regroupe le tout. Tout tourne sur le CPU.

use crate::{engine, AppState, Segment, R};
use serde_json::json;
use sherpa_onnx::{OfflineSpeakerDiarization, OfflineSpeakerDiarizationConfig, SpeakerEmbeddingExtractor, SpeakerEmbeddingExtractorConfig};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, State};

const SEG_URL: &str = "https://huggingface.co/csukuangfj/sherpa-onnx-pyannote-segmentation-3-0/resolve/main/model.onnx";
const SEG_BYTES: u64 = 5_992_913;
const EMB_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx";
const EMB_BYTES: u64 = 28_281_164;

/// Seuil de regroupement des voix : plus il est haut, moins on distingue d'intervenants.
/// Mesuré sur une réunion Zoom de 17 personnes (extrait de 10 min, ~8 voix) : 5 voix principales stables
/// de 0,6 à 0,9 avec les trois modèles 3D-Speaker ; WeSpeaker (anglais) en fusionnait plusieurs.
const THRESHOLD: f32 = 0.9;
/// Décalage de la fenêtre d'analyse (fraction de sa longueur) : 0,1 → 0,3 divise le temps de calcul par ~3.
const WINDOW_SHIFT: f32 = 0.2;
/// En mode automatique, une voix qui parle moins de 5 s et de 0,5 % du temps est une erreur de regroupement : on la fusionne.
const MIN_SECS: f64 = 5.0;
const MIN_SHARE: f64 = 0.005;

/// (fichier, URL, taille exacte en octets)
fn files(dir: &Path) -> [(PathBuf, &'static str, u64); 2] {
    [
        (dir.join("diarization").join("segmentation-3.0.onnx"), SEG_URL, SEG_BYTES),
        (dir.join("diarization").join("campplus-zh-en.onnx"), EMB_URL, EMB_BYTES),
    ]
}

pub fn total_bytes() -> u64 {
    SEG_BYTES + EMB_BYTES
}

pub fn dir_path(dir: &Path) -> PathBuf {
    dir.join("diarization")
}

pub fn ready(dir: &Path) -> bool {
    files(dir).iter().all(|(p, _, n)| fs::metadata(p).map(|m| m.len() == *n).unwrap_or(false))
}

/// Télécharge les modèles manquants (reprise possible).
pub fn install(dir: &Path) -> R<()> {
    for (p, url, n) in files(dir) {
        if fs::metadata(&p).map(|m| m.len() != n).unwrap_or(true) {
            engine::download_to(url, &p, n, "diarization", "diarization")?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn diarize_install(app: AppHandle, st: State<AppState>) -> R<()> {
    let dir = st.mdir.clone();
    engine::run_job(app, "diarization".into(), "diarization", move || {
        install(&dir)?;
        let _ = preload(&dir); // prêt à l'emploi sans attendre la première transcription
        Ok(())
    })
}

/// Modèles gardés en mémoire (chargés au démarrage ou à la première séparation, libérés à la fermeture de l'application) :
/// ils évitent de relire et d'initialiser les modèles à chaque transcription. La clé est le dossier des modèles.
static DIAR: Mutex<Option<(PathBuf, OfflineSpeakerDiarization)>> = Mutex::new(None);
static EMB: Mutex<Option<(PathBuf, Arc<SpeakerEmbeddingExtractor>)>> = Mutex::new(None);

fn make_diarizer(dir: &Path) -> R<OfflineSpeakerDiarization> {
    let [(seg, _, _), (emb, _, _)] = files(dir);
    let threads = std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4).clamp(2, 8);
    let mut cfg = OfflineSpeakerDiarizationConfig::default();
    cfg.segmentation.pyannote.model = Some(seg.to_string_lossy().into_owned());
    cfg.segmentation.pyannote.window_shift_ratio = WINDOW_SHIFT;
    cfg.segmentation.num_threads = threads;
    cfg.embedding.model = Some(emb.to_string_lossy().into_owned());
    cfg.embedding.num_threads = threads;
    cfg.clustering.num_clusters = -1;
    cfg.clustering.threshold = THRESHOLD;
    OfflineSpeakerDiarization::create(&cfg).ok_or_else(|| "Modèles de séparation des intervenants illisibles : supprimez-les et téléchargez-les à nouveau.".to_string())
}

fn make_embedder(dir: &Path) -> R<SpeakerEmbeddingExtractor> {
    let [_, (emb, _, _)] = files(dir);
    let mut cfg = SpeakerEmbeddingExtractorConfig::default();
    cfg.model = Some(emb.to_string_lossy().into_owned());
    cfg.num_threads = 1;
    SpeakerEmbeddingExtractor::create(&cfg).ok_or_else(|| "Modèle d'empreintes de voix illisible : supprimez-le et téléchargez-le à nouveau.".to_string())
}

fn embedder(dir: &Path) -> R<Arc<SpeakerEmbeddingExtractor>> {
    let mut g = EMB.lock().unwrap_or_else(|e| e.into_inner());
    if g.as_ref().map_or(true, |(p, _)| p != dir) {
        *g = Some((dir.to_path_buf(), Arc::new(make_embedder(dir)?)));
    }
    Ok(g.as_ref().unwrap().1.clone())
}

/// Charge les modèles en mémoire (appelé au démarrage, en arrière-plan) ; sans effet s'ils ne sont pas installés.
pub fn preload(dir: &Path) -> R<()> {
    if !ready(dir) {
        return Ok(());
    }
    {
        let mut g = DIAR.lock().unwrap_or_else(|e| e.into_inner());
        if g.as_ref().map_or(true, |(p, _)| p != dir) {
            *g = Some((dir.to_path_buf(), make_diarizer(dir)?));
        }
    }
    embedder(dir).map(|_| ())
}

/// Libère les modèles (suppression des fichiers, fermeture).
pub fn release() {
    *DIAR.lock().unwrap_or_else(|e| e.into_inner()) = None;
    *EMB.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// Fichiers des modèles et leur taille, pour la lecture préalable au démarrage.
pub fn model_files(dir: &Path) -> Vec<PathBuf> {
    files(dir).into_iter().map(|(p, _, _)| p).collect()
}

#[tauri::command]
pub fn diarize_remove(st: State<AppState>) -> R<()> {
    release(); // les modèles chargés verrouillent leurs fichiers
    for (p, _, _) in files(&st.mdir) {
        let _ = fs::remove_file(&p);
        let _ = fs::remove_file(p.with_extension("part"));
    }
    Ok(())
}

/// Plage de parole d'un intervenant (numéroté à partir de 0, dans l'ordre d'apparition).
#[derive(Clone, Debug)]
pub struct Turn {
    pub start: f64,
    pub end: f64,
    pub speaker: usize,
}

/// WAV 16 kHz mono 16 bits produit par `decode_to_wav16k` (en-tête de 44 octets).
pub fn load(p: &Path) -> R<Vec<f32>> {
    let b = fs::read(p).map_err(|e| e.to_string())?;
    if b.len() <= 44 {
        return Err("Audio vide.".into());
    }
    Ok(b[44..].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect())
}

/// Temps de calcul par seconde d'audio, mesuré à la dernière séparation (0 = pas encore mesuré). sherpa-onnx ne donne aucun
/// avancement pendant le calcul : le pourcentage affiché est une estimation à partir de cette vitesse.
static RATE: AtomicU64 = AtomicU64::new(0);
const DEFAULT_RATE: f64 = 0.06;

/// Qui parle quand ; le nombre d'intervenants est déterminé automatiquement.
/// `on_pct` reçoit l'avancement estimé (0 à 99) toutes les 400 ms environ pendant le calcul.
pub fn diarize(samples: &[f32], dir: &Path, on_pct: &dyn Fn(u8)) -> R<Vec<Turn>> {
    let secs = samples.len() as f64 / 16000.0;
    let rate = match RATE.load(Ordering::Relaxed) {
        0 => DEFAULT_RATE,
        b => f64::from_bits(b),
    };
    let expected = (secs * rate).max(1.0);
    let t0 = Instant::now();
    let r = std::thread::scope(|sc| {
        let job = sc.spawn(|| -> R<_> {
            // le verrou reste pris pendant le calcul : une seule séparation à la fois sur ces modèles partagés
            let mut g = DIAR.lock().unwrap_or_else(|e| e.into_inner());
            if g.as_ref().map_or(true, |(p, _)| p != dir) {
                *g = Some((dir.to_path_buf(), make_diarizer(dir)?));
            }
            g.as_ref().unwrap().1.process(samples).ok_or_else(|| "La séparation des intervenants a échoué.".to_string())
        });
        while !job.is_finished() {
            // 0 → 95 % sur la durée prévue, puis on s'approche de 99 % sans l'atteindre si le calcul dure plus longtemps
            let x = t0.elapsed().as_secs_f64() / expected;
            let p = if x < 1.0 { x * 95.0 } else { 95.0 + 4.0 * (1.0 - 1.0 / x) };
            on_pct(p.min(99.0) as u8);
            std::thread::sleep(Duration::from_millis(400));
        }
        job.join().unwrap_or_else(|_| Err("La séparation des intervenants a planté.".to_string()))
    })?;
    if secs > 30.0 {
        RATE.store((t0.elapsed().as_secs_f64() / secs).to_bits(), Ordering::Relaxed);
    }
    on_pct(100);
    let raw: Vec<(f64, f64, i32)> = r.sort_by_start_time().iter().map(|g| (g.start as f64, g.end as f64, g.speaker)).collect();
    Ok(tidy(raw, false))
}

/// Écart en secondes entre deux plages (0 si elles se chevauchent).
fn gap(a: (f64, f64), b: (f64, f64)) -> f64 {
    if b.0 >= a.1 { b.0 - a.1 } else if a.0 >= b.1 { a.0 - b.1 } else { 0.0 }
}

/// Fusionne les voix trop rares (mode automatique) avec la plage de parole la plus proche, puis renumérote
/// les intervenants dans l'ordre de leur première prise de parole.
fn tidy(raw: Vec<(f64, f64, i32)>, fixed: bool) -> Vec<Turn> {
    let mut total: BTreeMap<i32, f64> = BTreeMap::new();
    for t in &raw {
        *total.entry(t.2).or_default() += t.1 - t.0;
    }
    let all: f64 = total.values().sum();
    let min = MIN_SECS.max(all * MIN_SHARE);
    let keep = |s: i32| fixed || total.get(&s).copied().unwrap_or(0.0) >= min;
    let kept: Vec<&(f64, f64, i32)> = raw.iter().filter(|t| keep(t.2)).collect();
    let mut order: Vec<i32> = Vec::new();
    let mut out = Vec::with_capacity(raw.len());
    for t in &raw {
        let spk = if keep(t.2) {
            t.2
        } else {
            kept.iter().min_by(|a, b| gap((t.0, t.1), (a.0, a.1)).total_cmp(&gap((t.0, t.1), (b.0, b.1)))).map(|k| k.2).unwrap_or(t.2)
        };
        let n = match order.iter().position(|&x| x == spk) {
            Some(i) => i,
            None => {
                order.push(spk);
                order.len() - 1
            }
        };
        out.push(Turn { start: t.0, end: t.1, speaker: n });
    }
    out
}

/// Attribue à chaque segment l'intervenant qui parle le plus longtemps pendant sa durée
/// (à défaut, le plus proche dans le temps) : « Intervenant 1 », « Intervenant 2 »…
pub fn assign(segs: &mut [Segment], turns: &[Turn]) {
    if turns.is_empty() {
        return;
    }
    for g in segs.iter_mut() {
        let mut ov: Vec<(usize, f64)> = Vec::new();
        for t in turns {
            let o = (g.end.min(t.end) - g.start.max(t.start)).max(0.0);
            if o > 0.0 {
                match ov.iter_mut().find(|x| x.0 == t.speaker) {
                    Some(x) => x.1 += o,
                    None => ov.push((t.speaker, o)),
                }
            }
        }
        let spk = ov
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0)
            .or_else(|| turns.iter().min_by(|a, b| gap((g.start, g.end), (a.start, a.end)).total_cmp(&gap((g.start, g.end), (b.start, b.end)))).map(|t| t.speaker))
            .unwrap_or(0);
        g.speaker = format!("Intervenant {}", spk + 1);
    }
}

// ─── Présentations : « bonjour, je suis X » ─────────────────────────────

/// Un segment plus court ne donne pas d'empreinte de voix fiable.
const MIN_EMB_SECS: f64 = 1.5;
/// Durée de parole utilisée au plus pour une empreinte.
const MAX_EMB_SECS: f64 = 12.0;
/// La parole qui suit une présentation, dans cette fenêtre, sert aussi de référence pour la voix.
const FOLLOW_SECS: f64 = 30.0;
const FOLLOW_MAX: usize = 3;
/// Similarité (cosinus) minimale avec une voix de référence, et écart minimal avec la deuxième.
const SIM_MIN: f32 = 0.45;
const SIM_MARGIN: f32 = 0.04;

/// Mots qui suivent « je suis » sans être un prénom (Whisper met une majuscule en début de phrase).
const NOT_NAMES: &[&str] = &[
    "bonjour", "bonsoir", "salut", "merci", "et", "ok", "oui", "non", "alors", "donc", "enfin", "voilà", "je", "nous", "on", "ici", "là",
    "désolé", "désolée", "content", "contente", "heureux", "heureuse", "ravi", "ravie", "fatigué", "fatiguée", "prêt", "prête", "le", "la",
    "les", "un", "une", "avec", "pas", "très", "bien", "hello", "fine", "good", "sorry", "happy", "here", "tired", "ready", "the", "a", "and", "so",
];

fn is_stop(w: &str) -> bool {
    NOT_NAMES.contains(&w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase().as_str())
}

fn clean_name(raw: &str) -> Option<String> {
    let mut words: Vec<&str> = raw.split_whitespace().collect();
    // un mot de liaison collé au nom (« Marie Bonjour à tous ») n'en fait pas partie
    while words.len() > 1 && words.last().map(|w| is_stop(w)).unwrap_or(false) {
        words.pop();
    }
    if words.is_empty() || is_stop(words[0]) {
        return None;
    }
    let name = words.join(" ").trim_matches(|c| c == '\'' || c == '’' || c == '-').to_string();
    (name.chars().count() >= 2 && name.chars().count() <= 30).then_some(name)
}

fn intro_regexes() -> &'static [regex::Regex] {
    static R: std::sync::OnceLock<Vec<regex::Regex>> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        // le nom doit commencer par une majuscule : « je suis content » n'est pas une présentation
        let name = r"(?P<n>\p{Lu}[\p{L}'’-]*(?:\s+\p{Lu}[\p{L}'’-]*)?)";
        let lead = r"(?i:je m['’]appelle|moi c['’]est|mon (?:nom|prénom) est|je me présente,?\s+je suis|je suis|my name is|i['’]m|i am)";
        [
            format!(r"{lead}\s+{name}"),
            format!(r"(?i:bonjour|bonsoir|salut|hello|coucou)\W+(?i:c['’]est)\s+{name}"),
            format!(r"{name}\s+(?i:à l['’]appareil)"),
        ]
        .iter()
        .map(|p| regex::Regex::new(p).expect("motif de présentation"))
        .collect()
    })
}

/// Nom donné par une personne qui se présente (« Bonjour, je suis Marie Dupont »), si le segment en contient un.
pub fn intro_name(text: &str, secs: f64) -> Option<String> {
    if secs > 20.0 {
        return None;
    }
    intro_regexes().iter().find_map(|re| re.captures(text).and_then(|c| clean_name(&c["n"])))
}

fn unit(mut v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        v.iter_mut().for_each(|x| *x /= n);
    }
    v
}

fn mean_unit(vs: &[&Vec<f32>]) -> Vec<f32> {
    let mut m = vec![0.0f32; vs[0].len()];
    for v in vs {
        m.iter_mut().zip(v.iter()).for_each(|(a, b)| *a += b);
    }
    unit(m)
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Ce que `resolve` sait d'un segment.
struct SegInfo {
    /// intervenant numéroté de la séparation de base (« Intervenant 3 »)
    base: String,
    start: f64,
    end: f64,
    emb: Option<Vec<f32>>,
    intro: Option<String>,
}

/// Nom de la personne qui parle dans chaque segment, d'après les voix qui se sont présentées.
/// 1. chaque présentation fixe une voix de référence (le segment et la parole qui suit, de la même voix de base) ;
/// 2. un segment prend le nom de la référence la plus proche si elle est assez proche et nettement meilleure que la suivante ;
/// 3. les autres (trop courts, voix douteuse) prennent le nom qui domine leur intervenant de base, s'il y en a un.
fn resolve(info: &[SegInfo]) -> Vec<Option<String>> {
    let mut refs: Vec<(String, Vec<&Vec<f32>>)> = Vec::new();
    for (i, s) in info.iter().enumerate() {
        let Some(name) = &s.intro else { continue };
        let mut members: Vec<&Vec<f32>> = s.emb.iter().collect();
        let mut taken = 0;
        for t in &info[i + 1..] {
            if t.start - s.end > FOLLOW_SECS || taken >= FOLLOW_MAX {
                break;
            }
            let other_intro = t.intro.as_ref().map(|n| !n.eq_ignore_ascii_case(name)).unwrap_or(false);
            if t.base == s.base && !other_intro {
                if let Some(e) = &t.emb {
                    members.push(e);
                    taken += 1;
                }
            }
        }
        if members.is_empty() {
            continue;
        }
        match refs.iter_mut().find(|(n, _)| n.to_lowercase() == name.to_lowercase()) {
            Some((_, v)) => v.extend(members),
            None => refs.push((name.clone(), members)),
        }
    }
    let refvecs: Vec<(&String, Vec<f32>)> = refs.iter().map(|(n, v)| (n, mean_unit(v))).collect();
    if refvecs.is_empty() {
        return vec![None; info.len()];
    }
    // étape 2
    let direct: Vec<Option<&String>> = info
        .iter()
        .map(|s| {
            let e = s.emb.as_ref()?;
            if let Some(n) = &s.intro {
                if let Some((rn, _)) = refvecs.iter().find(|(r, _)| r.to_lowercase() == n.to_lowercase()) {
                    return Some(*rn);
                }
            }
            let mut sims: Vec<(f32, &String)> = refvecs.iter().map(|(n, v)| (cosine(e, v), *n)).collect();
            sims.sort_by(|a, b| b.0.total_cmp(&a.0));
            let second = sims.get(1).map(|x| x.0).unwrap_or(-1.0);
            (sims[0].0 >= SIM_MIN && sims[0].0 - second >= SIM_MARGIN).then_some(sims[0].1)
        })
        .collect();
    // étape 3 : nom dominant de chaque intervenant de base
    let mut total: std::collections::HashMap<&str, f64> = Default::default();
    let mut matched: std::collections::HashMap<&str, std::collections::HashMap<&String, f64>> = Default::default();
    for (s, d) in info.iter().zip(&direct) {
        *total.entry(&s.base).or_default() += s.end - s.start;
        if let Some(n) = *d {
            *matched.entry(&s.base).or_default().entry(n).or_default() += s.end - s.start;
        }
    }
    let mut by_base: std::collections::HashMap<&str, &String> = Default::default();
    for (base, m) in &matched {
        let sum: f64 = m.values().sum();
        if let Some((n, d)) = m.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
            if *d >= 0.6 * sum && sum >= 0.25 * total[*base] {
                by_base.insert(*base, *n);
            }
        }
    }
    info.iter().zip(direct).map(|(s, d)| d.or_else(|| by_base.get(s.base.as_str()).copied()).cloned()).collect()
}

/// Empreinte de la voix d'un segment (`None` s'il est trop court).
fn embed(ex: &SpeakerEmbeddingExtractor, samples: &[f32], g: &Segment) -> Option<Vec<f32>> {
    if g.end - g.start < MIN_EMB_SECS {
        return None;
    }
    let a = ((g.start * 16000.0) as usize).min(samples.len());
    let b = (((g.start + (g.end - g.start).min(MAX_EMB_SECS)) * 16000.0) as usize).min(samples.len());
    if b < a + 16000 {
        return None;
    }
    let s = ex.create_stream()?;
    s.accept_waveform(16000, &samples[a..b]);
    s.input_finished();
    if !ex.is_ready(&s) {
        return None;
    }
    ex.compute(&s).map(unit)
}

/// Donne leur nom aux voix qui se présentent (« Bonjour, je suis Marie ») : les segments de cette voix,
/// y compris ceux que la séparation de base avait attribués à un autre intervenant, deviennent « Marie ».
/// Les autres restent « Intervenant N ». Renvoie le nombre de noms attribués.
pub fn identify(samples: &[f32], dir: &Path, segs: &mut [Segment]) -> R<usize> {
    let intros: Vec<Option<String>> = segs.iter().map(|g| intro_name(&g.text, g.end - g.start)).collect();
    if intros.iter().all(Option::is_none) {
        return Ok(0);
    }
    let ex = embedder(dir)?;
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 8);
    let chunk = segs.len().div_ceil(threads).max(1);
    let mut embs: Vec<Option<Vec<f32>>> = vec![None; segs.len()];
    std::thread::scope(|sc| {
        for (cs, os) in segs.chunks(chunk).zip(embs.chunks_mut(chunk)) {
            let ex = &ex;
            sc.spawn(move || {
                for (g, o) in cs.iter().zip(os.iter_mut()) {
                    *o = embed(ex, samples, g);
                }
            });
        }
    });
    let info: Vec<SegInfo> = segs
        .iter()
        .zip(embs)
        .zip(intros)
        .map(|((g, emb), intro)| SegInfo { base: g.speaker.clone(), start: g.start, end: g.end, emb, intro })
        .collect();
    let mut found = std::collections::HashSet::new();
    for (g, n) in segs.iter_mut().zip(resolve(&info)) {
        if let Some(n) = n {
            found.insert(n.to_lowercase());
            g.speaker = n;
        }
    }
    Ok(found.len())
}

/// Relance la séparation des intervenants sur une réunion déjà transcrite (sans refaire la transcription).
#[tauri::command]
pub fn diarize_meeting(app: AppHandle, st: State<AppState>, id: String) -> R<()> {
    if !ready(&st.mdir) {
        return Err("Téléchargez d'abord les modèles de séparation des intervenants (Réglages › Transcription).".into());
    }
    let audio = {
        let s = st.store.lock().unwrap();
        let m = s.meetings.iter().find(|m| m.id == id).ok_or("Réunion introuvable")?;
        if m.status != "ready" {
            return Err("La réunion n'est pas prête.".into());
        }
        m.audio_path.clone().ok_or("Cette réunion n'a pas d'audio.")?
    };
    engine::set_transcribe_pct(&app, &id, 0);
    engine::set_status(&app, &id, "diarizing", None, None);
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        // un verrou « empoisonné » par un plantage précédent ne doit pas bloquer la file
        let _guard = st.job_lock.lock().unwrap_or_else(|e| e.into_inner());
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> R<()> {
            let tmp = st.dir.join(format!("{id}.16k.wav"));
            engine::decode_to_wav16k(&audio, &tmp)?;
            let samples = load(&tmp);
            let _ = fs::remove_file(&tmp);
            let samples = samples?;
            engine::set_transcribe_pct(&app, &id, 0);
            let turns = diarize(&samples, &st.mdir, &|p| engine::set_transcribe_pct(&app, &id, p))?;
            // calcul hors du verrou (plusieurs minutes) ; les noms donnés à la main sont écrasés : on repart des intervenants numérotés
            let mut segs = st.store.lock().unwrap().meetings.iter().find(|m| m.id == id).map(|m| m.transcript.clone()).ok_or("Réunion introuvable")?;
            assign(&mut segs, &turns);
            if let Err(e) = identify(&samples, &st.mdir, &mut segs) {
                eprintln!("Reconnaissance des présentations impossible : {e}");
            }
            let mut s = st.store.lock().unwrap();
            if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
                m.transcript = segs;
                let names: Vec<String> = m.transcript.iter().map(|g| g.speaker.clone()).collect();
                // les traductions ont les mêmes segments, dans le même ordre
                for tr in m.translations.values_mut().filter(|t| t.len() == names.len()) {
                    for (g, n) in tr.iter_mut().zip(&names) {
                        g.speaker = n.clone();
                    }
                }
            }
            st.save(&s)
        }))
        .unwrap_or_else(|_| Err("La séparation des intervenants a planté.".into()));
        engine::set_status(&app, &id, "ready", None, None);
        let _ = app.emit("diarize-done", json!({ "id": id, "ok": res.is_ok(), "error": res.err() }));
    });
    Ok(())
}

/// Renomme un intervenant dans la transcription et dans toutes les traductions.
#[tauri::command]
pub fn rename_speaker(st: State<AppState>, id: String, from: String, to: String) -> R<()> {
    let to = to.trim().to_string();
    if to.is_empty() {
        return Err("Le nom ne peut pas être vide.".into());
    }
    let mut s = st.store.lock().unwrap();
    let m = s.meetings.iter_mut().find(|m| m.id == id).ok_or("Réunion introuvable")?;
    for g in m.transcript.iter_mut().chain(m.translations.values_mut().flatten()) {
        if g.speaker == from {
            g.speaker = to.clone();
        }
    }
    st.save(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fusionne_les_voix_rares_et_renumerote() {
        // la voix 7 (1 s sur 100 s) est absorbée par la plage voisine ; les numéros suivent l'ordre d'apparition
        let raw = vec![(0.0, 40.0, 5), (40.0, 41.0, 7), (41.0, 70.0, 2), (70.0, 100.0, 5)];
        let t = tidy(raw, false);
        assert_eq!(t.iter().map(|x| x.speaker).collect::<Vec<_>>(), vec![0, 0, 1, 0]);
    }

    #[test]
    fn nombre_fixe_garde_toutes_les_voix() {
        let raw = vec![(0.0, 99.0, 1), (99.0, 100.0, 2)];
        assert_eq!(tidy(raw, true).iter().map(|x| x.speaker).collect::<Vec<_>>(), vec![0, 1]);
    }

    #[test]
    fn detecte_les_presentations() {
        let n = |t: &str| intro_name(t, 5.0);
        assert_eq!(n("Bonjour, je suis Marie Dupont, responsable du projet."), Some("Marie Dupont".into()));
        assert_eq!(n("Bonjour à tous, moi c'est Paul."), Some("Paul".into()));
        assert_eq!(n("Bonsoir, c'est Sophie."), Some("Sophie".into()));
        assert_eq!(n("Je m'appelle Jean-Pierre et je travaille à Lyon."), Some("Jean-Pierre".into()));
        assert_eq!(n("Hello, my name is Alice Martin."), Some("Alice Martin".into()));
        assert_eq!(n("Paul à l'appareil."), Some("Paul".into()));
        assert_eq!(n("Bonjour, je suis Marie Bonjour à tous"), Some("Marie".into()));
        // pas des présentations
        assert_eq!(n("Je suis content de vous voir."), None);
        assert_eq!(n("Je suis d'accord avec Paul."), None);
        assert_eq!(n("Je suis Désolé pour le retard."), None);
        assert_eq!(n("C'est Paul qui a le dossier."), None);
        // trop long pour une présentation
        assert_eq!(intro_name("Bonjour, je suis Marie.", 40.0), None);
    }

    fn info(base: &str, start: f64, end: f64, emb: Option<[f32; 3]>, intro: Option<&str>) -> SegInfo {
        SegInfo { base: base.into(), start, end, emb: emb.map(|e| unit(e.to_vec())), intro: intro.map(String::from) }
    }

    #[test]
    fn nomme_les_voix_qui_se_presentent() {
        let (a, b) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let v = vec![
            info("Intervenant 1", 0.0, 4.0, Some(a), Some("Marie")),
            info("Intervenant 2", 5.0, 9.0, Some(b), Some("Paul")),
            info("Intervenant 1", 10.0, 14.0, Some(a), None),
            // la séparation de base avait mis la voix de Paul dans l'intervenant 1 : l'empreinte corrige
            info("Intervenant 1", 15.0, 19.0, Some(b), None),
            // trop court pour une empreinte : prend le nom dominant de son intervenant de base
            info("Intervenant 2", 20.0, 21.0, None, None),
            // voix inconnue (aucune ressemblance) : reste numérotée
            info("Intervenant 3", 25.0, 29.0, Some([0.0, 0.0, 1.0]), None),
        ];
        let n = resolve(&v);
        assert_eq!(n, vec![Some("Marie".into()), Some("Paul".into()), Some("Marie".into()), Some("Paul".into()), Some("Paul".into()), None]);
    }

    #[test]
    fn sans_presentation_rien_ne_change() {
        let v = vec![info("Intervenant 1", 0.0, 4.0, Some([1.0, 0.0, 0.0]), None)];
        assert_eq!(resolve(&v), vec![None]);
    }

    #[test]
    fn attribue_le_plus_long_recouvrement() {
        let turns = vec![Turn { start: 0.0, end: 4.0, speaker: 0 }, Turn { start: 4.0, end: 10.0, speaker: 1 }];
        let mut segs = vec![Segment { speaker: String::new(), start: 3.0, end: 8.0, text: "x".into() }];
        assign(&mut segs, &turns);
        assert_eq!(segs[0].speaker, "Intervenant 2");
    }

    /// Essai réel (modèles installés requis) :
    /// `VERBATIM_TEST_IA=<dossier ia> VERBATIM_TEST_WAV=<wav 16 kHz> cargo test charge_une_fois -- --ignored --nocapture`
    /// Vérifie que le préchargement garde les modèles (la séparation suivante ne les recharge pas) et que le pourcentage est croissant.
    #[test]
    #[ignore]
    fn charge_une_fois_et_donne_un_pourcentage() {
        let dir = PathBuf::from(std::env::var("VERBATIM_TEST_IA").expect("VERBATIM_TEST_IA"));
        let wav = PathBuf::from(std::env::var("VERBATIM_TEST_WAV").expect("VERBATIM_TEST_WAV"));
        assert!(ready(&dir), "modèles de séparation absents de {}", dir.display());
        let mut s = load(&wav).unwrap();
        // durée analysée : 2 minutes par défaut, VERBATIM_TEST_SECS pour une autre (ex. 916 pour 15 min, 3600 pour 1 h)
        let secs: usize = std::env::var("VERBATIM_TEST_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
        s.truncate(secs * 16000);
        let t = Instant::now();
        preload(&dir).unwrap();
        println!("préchargement : {:.1} s", t.elapsed().as_secs_f64());
        for essai in 1..=2 {
            let pcts = Mutex::new(Vec::<u8>::new());
            let t = Instant::now();
            let turns = diarize(&s, &dir, &|p| pcts.lock().unwrap().push(p)).unwrap();
            let p = pcts.into_inner().unwrap();
            println!("séparation n°{essai} : {:.1} s, {} plages, {} intervenants, pourcentages {:?}", t.elapsed().as_secs_f64(), turns.len(), turns.iter().map(|x| x.speaker).max().map_or(0, |m| m + 1), p);
            assert!(p.windows(2).all(|w| w[0] <= w[1]), "le pourcentage ne doit jamais reculer : {p:?}");
            assert_eq!(p.last(), Some(&100));
        }
        release();
    }
}
