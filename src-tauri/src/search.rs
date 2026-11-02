//! Recherche dans toutes les réunions (titre, transcription, compte-rendu), sans tenir compte des accents ni de la casse.

use crate::{AppState, Meeting, R};
use serde_json::{json, Value};
use tauri::State;

/// Minuscules sans accents, caractère pour caractère (les positions restent alignées sur le texte d'origine).
fn fold(t: &str) -> Vec<char> {
    t.chars()
        .map(|c| {
            let c = c.to_lowercase().next().unwrap_or(c);
            match c {
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'æ' | 'ā' => 'a',
                'ç' | 'ć' | 'č' => 'c',
                'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
                'ì' | 'í' | 'î' | 'ï' | 'ī' => 'i',
                'ñ' | 'ń' => 'n',
                'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'œ' | 'ō' => 'o',
                'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' => 'u',
                'ý' | 'ÿ' => 'y',
                'ß' | 'ś' | 'š' => 's',
                'ž' | 'ź' | 'ż' => 'z',
                '’' => '\'',
                c => c,
            }
        })
        .collect()
}

/// Premier emplacement (en caractères) de `q` dans `hay`.
fn find(hay: &[char], q: &[char]) -> Option<usize> {
    if q.is_empty() || hay.len() < q.len() {
        return None;
    }
    (0..=hay.len() - q.len()).find(|&i| hay[i..i + q.len()] == *q)
}

/// Extrait autour de la première occurrence : (avant, trouvé, après), une soixantaine de caractères de contexte.
fn snippet(text: &str, q: &[char]) -> Option<(String, String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let i = find(&fold(text), q)?;
    let a = i.saturating_sub(60);
    let b = (i + q.len() + 80).min(chars.len());
    let s = |x: &[char]| x.iter().collect::<String>();
    let mut before = s(&chars[a..i]);
    let mut after = s(&chars[i + q.len()..b]);
    if a > 0 {
        before = format!("…{}", before.trim_start());
    }
    if b < chars.len() {
        after = format!("{}…", after.trim_end());
    }
    Some((before, s(&chars[i..i + q.len()]), after))
}

fn hit(kind: &str, t: Option<f64>, who: &str, sn: (String, String, String)) -> Value {
    json!({ "kind": kind, "t": t, "who": who, "before": sn.0, "match": sn.1, "after": sn.2 })
}

/// Textes du compte-rendu : (section, texte).
fn report_texts(m: &Meeting) -> Vec<String> {
    let Some(r) = &m.report else { return vec![] };
    let mut v = Vec::new();
    if let Some(s) = r.get("summary").and_then(|x| x.as_str()) {
        v.push(s.to_string());
    }
    for k in ["topics", "decisions", "actions", "next_steps"] {
        if let Some(a) = r.get(k).and_then(|x| x.as_array()) {
            v.extend(a.iter().filter_map(|x| x.as_str().map(String::from)));
        }
    }
    v
}

/// Réunions (hors corbeille) contenant `query` : jusqu'à 4 extraits par réunion, le total des occurrences en plus.
#[tauri::command]
pub fn search_meetings(st: State<AppState>, query: String) -> R<Vec<Value>> {
    let q = fold(query.trim());
    if q.len() < 2 {
        return Ok(vec![]);
    }
    let s = st.store.lock().unwrap();
    let mut ms: Vec<&Meeting> = s.meetings.iter().filter(|m| !m.deleted).collect();
    ms.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let mut out = Vec::new();
    for m in ms {
        let mut hits = Vec::new();
        let mut total = 0usize;
        if let Some(sn) = snippet(&m.title, &q) {
            total += 1;
            hits.push(hit("title", None, "", sn));
        }
        for t in report_texts(m) {
            if let Some(sn) = snippet(&t, &q) {
                total += 1;
                if hits.len() < 4 {
                    hits.push(hit("report", None, "", sn));
                }
            }
        }
        for seg in &m.transcript {
            if let Some(sn) = snippet(&seg.text, &q) {
                total += 1;
                if hits.len() < 4 {
                    hits.push(hit("transcript", Some(seg.start), &seg.speaker, sn));
                }
            }
        }
        if total > 0 {
            out.push(json!({ "id": m.id, "title": m.title, "created_at": m.created_at, "total": total, "hits": hits }));
        }
    }
    Ok(out)
}
