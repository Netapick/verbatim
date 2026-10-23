//! Traduction de la transcription par l'IA locale : les segments sont traduits par lots, les horodatages
//! d'origine sont conservés (le lecteur et les exports restent alignés). Utilisée aussi pour le direct.

use crate::llm::{self, Backend};
use crate::{AppState, Segment, Settings, R};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, State};

/// Demande d'arrêt de la traduction en cours (vérifiée entre deux lots).
static STOP: AtomicBool = AtomicBool::new(false);
const STOPPED: &str = "__arrêtée__";

/// Nom anglais de la langue (invite de TranslateGemma et des autres modèles).
pub fn en_name(code: &str) -> &'static str {
    match code {
        "fr" => "French", "en" => "English", "es" => "Spanish", "de" => "German", "it" => "Italian",
        "pt" => "Portuguese", "nl" => "Dutch", "pl" => "Polish", "ru" => "Russian", "ja" => "Japanese",
        "zh" => "Chinese", "ar" => "Arabic", "tr" => "Turkish", "ko" => "Korean",
        _ => "",
    }
}

/// Modèle de traduction choisi : celui des Réglages, sinon celui du compte-rendu.
pub fn model_for(cfg: &Settings) -> String {
    if cfg.translate_model.is_empty() { cfg.llm_local.clone() } else { cfg.translate_model.clone() }
}

fn gemma_prompt(src: &str, dst: &str, text: &str) -> String {
    let d = en_name(dst);
    let head = match en_name(src) {
        "" => format!(
            "You are a professional translator. Your goal is to accurately convey the meaning and nuances of the original text \
             while adhering to {d} grammar, vocabulary, and cultural sensitivities.\n\
             Produce only the {d} translation, without any additional explanations or commentary. Please translate the following text into {d}:"),
        s => format!(
            "You are a professional {s} ({src}) to {d} ({dst}) translator. Your goal is to accurately convey the meaning and nuances of the original {s} text \
             while adhering to {d} grammar, vocabulary, and cultural sensitivities.\n\
             Produce only the {d} translation, without any additional explanations or commentary. Please translate the following {s} text into {d}:"),
    };
    format!("<start_of_turn>user\n{head}\n\n\n{text}<end_of_turn>\n<start_of_turn>model\n")
}

fn chat_system(src: &str, dst: &str, n: usize) -> String {
    let from = match en_name(src) { "" => String::new(), s => format!(" from {s}") };
    format!(
        "You are a professional translator. Translate each line of the user's text{from} into {}. \
         Output exactly {n} line(s), in the same order, one translation per input line, with no numbering, no quotes and no commentary. \
         Keep names, numbers and technical terms unchanged.",
        en_name(dst)
    )
}

/// Traduit un ou plusieurs textes d'un coup ; renvoie la réponse brute du modèle.
fn call(be: &Backend, gemma: bool, src: &str, dst: &str, lines: &[&str]) -> R<String> {
    let text = lines.join("\n");
    let max = ((text.chars().count() as u32) * 2 + 200).clamp(200, 3000);
    if gemma {
        llm::raw_complete(be, &gemma_prompt(src, dst, &text), max, &["<end_of_turn>"])
    } else {
        llm::chat_text(be, &chat_system(src, dst, lines.len()), &text, max)
    }
}

/// Une traduction par ligne d'entrée. Si le modèle ne rend pas autant de lignes, on retraduit une par une.
pub fn translate_lines(be: &Backend, gemma: bool, src: &str, dst: &str, lines: &[String]) -> R<Vec<String>> {
    let todo: Vec<(usize, &str)> = lines.iter().enumerate().map(|(i, l)| (i, l.trim())).filter(|(_, l)| !l.is_empty()).collect();
    let mut out = vec![String::new(); lines.len()];
    if todo.is_empty() {
        return Ok(out);
    }
    if todo.len() > 1 {
        let raw = call(be, gemma, src, dst, &todo.iter().map(|x| x.1).collect::<Vec<_>>())?;
        let got: Vec<&str> = raw.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if got.len() == todo.len() {
            for ((i, _), g) in todo.iter().zip(got) {
                out[*i] = g.to_string();
            }
            return Ok(out);
        }
    }
    for (i, l) in todo {
        let raw = call(be, gemma, src, dst, &[l])?;
        out[i] = raw.lines().map(str::trim).filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" ");
    }
    Ok(out)
}

/// Traduit toute la transcription, par lots de quelques segments.
pub fn translate_segments(be: &Backend, gemma: bool, src: &str, dst: &str, segs: &[Segment], on_pct: &dyn Fn(f32)) -> R<Vec<Segment>> {
    let mut out: Vec<Segment> = Vec::with_capacity(segs.len());
    let mut i = 0;
    while i < segs.len() {
        if STOP.load(Ordering::Relaxed) {
            return Err(STOPPED.into());
        }
        let mut j = i;
        let mut chars = 0;
        while j < segs.len() && j - i < 8 && (j == i || chars + segs[j].text.len() < 1200) {
            chars += segs[j].text.len();
            j += 1;
        }
        let lines: Vec<String> = segs[i..j].iter().map(|s| s.text.clone()).collect();
        let tr = translate_lines(be, gemma, src, dst, &lines)?;
        for (s, t) in segs[i..j].iter().zip(tr) {
            out.push(Segment { speaker: s.speaker.clone(), start: s.start, end: s.end, text: if t.is_empty() { s.text.clone() } else { t } });
        }
        i = j;
        on_pct(100.0 * i as f32 / segs.len() as f32);
    }
    Ok(out)
}

fn set_state(app: &AppHandle, id: &str, f: impl FnOnce(&mut crate::Meeting)) {
    let st = app.state::<AppState>();
    let mut s = st.store.lock().unwrap();
    if let Some(m) = s.meetings.iter_mut().find(|m| m.id == id) {
        f(m);
    }
    let _ = st.save(&s);
}

#[tauri::command]
pub fn translate_meeting(app: AppHandle, st: State<AppState>, id: String, lang: String) -> R<()> {
    if en_name(&lang).is_empty() {
        return Err("Langue de traduction inconnue.".into());
    }
    let (cfg, segs) = {
        let mut s = st.store.lock().unwrap();
        let cfg = s.settings.clone();
        let m = s.meetings.iter_mut().find(|m| m.id == id).ok_or("Réunion introuvable")?;
        if m.status != "ready" {
            return Err("La transcription n'est pas encore terminée.".into());
        }
        if m.translate_state == "translating" {
            return Err("Une traduction est déjà en cours.".into());
        }
        if m.transcript.is_empty() {
            return Err("La transcription est vide.".into());
        }
        STOP.store(false, Ordering::Relaxed);
        m.translate_state = "translating".into();
        m.translate_error = None;
        m.translate_lang = lang.clone();
        m.translate_pct = 0;
        let segs = m.transcript.clone();
        st.save(&s)?;
        (cfg, segs)
    };
    std::thread::spawn(move || {
        let src = if cfg.whisper_lang == "auto" { "" } else { cfg.whisper_lang.as_str() };
        let sel = model_for(&cfg);
        let gemma = llm::is_translator(&sel);
        let a = app.clone();
        let i = id.clone();
        let used = std::sync::Mutex::new(String::new());
        let res = if src == lang {
            Err("La transcription est déjà dans cette langue.".to_string())
        } else {
            llm::with_backend(&app, &llm::translate_cfg(&cfg), &sel, &|_| {}, &|be| {
                let (hw, label) = (be.hw.clone(), be.label.clone());
                *used.lock().unwrap() = format!("{label} · {hw}");
                set_state(&a, &i, |m| { m.translate_hw = hw; m.translate_label = label; });
                translate_segments(be, gemma, src, &lang, &segs, &|p| {
                    set_state(&a, &i, |m| m.translate_pct = (p as u8).min(99));
                })
            })
        };
        set_state(&app, &id, |m| {
            m.translate_pct = 0;
            m.translate_hw.clear();
            m.translate_label.clear();
            match res {
                Ok(v) => {
                    m.translations.insert(lang.clone(), v);
                    m.translation_models.insert(lang.clone(), used.lock().unwrap().clone());
                    m.translate_state = String::new();
                }
                Err(e) if e == STOPPED || STOP.load(Ordering::Relaxed) => {
                    m.translate_state = String::new(); // arrêtée par l'utilisateur : rien à signaler
                }
                Err(e) => {
                    m.translate_state = "error".into();
                    m.translate_error = Some(e);
                }
            }
        });
    });
    Ok(())
}

/// Arrête la traduction en cours (à la fin du lot en cours de traitement).
#[tauri::command]
pub fn translate_stop() {
    STOP.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn translation_remove(st: State<AppState>, id: String, lang: String) -> R<()> {
    let mut s = st.store.lock().unwrap();
    let m = s.meetings.iter_mut().find(|m| m.id == id).ok_or("Réunion introuvable")?;
    m.translations.remove(&lang);
    m.translation_models.remove(&lang);
    st.save(&s)
}
