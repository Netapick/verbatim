//! Mise à jour : le téléchargement, la vérification de la signature (minisign) et l'installation sont faits par
//! `tauri-plugin-updater` depuis l'interface (`window.__TAURI__.updater`). Ce module fournit ce que le plugin ne sait pas :
//! la version affichée et la garde « un travail est en cours, on ne met pas à jour maintenant ».

use crate::{AppState, R};
use serde_json::{json, Value};
use tauri::State;

/// Version du projet et étiquette de build `v{version}-{horodatage}` (l'horodatage est fixé à la compilation).
/// L'étiquette est informative : la comparaison des versions se fait toujours sur le numéro seul.
#[tauri::command]
pub fn app_version() -> Value {
    let version = env!("CARGO_PKG_VERSION");
    json!({ "version": version, "label": format!("v{version}-{}", env!("VERBATIM_BUILD_TS")) })
}

/// Arrête les moteurs d'IA avant l'installation : le plugin quitte l'application sans passer par la fermeture normale
/// (`RunEvent::Exit`), qui les arrête d'habitude ; un moteur encore lancé bloquerait l'installateur.
#[tauri::command]
pub fn update_prepare() {
    crate::procs::kill_all();
}

/// Premier travail en cours qui interdit une mise à jour (l'application va se fermer et redémarrer).
fn busy_reason<'a>(recording: bool, meetings: impl Iterator<Item = (&'a str, &'a str, &'a str)>, download: bool) -> Option<&'static str> {
    if recording {
        return Some("Un enregistrement est en cours.");
    }
    for (status, report, translate) in meetings {
        if matches!(status, "queued" | "converting" | "transcribing" | "diarizing") {
            return Some("Une transcription est en cours.");
        }
        if report == "generating" {
            return Some("Un compte-rendu est en cours de rédaction.");
        }
        if translate == "translating" {
            return Some("Une traduction est en cours.");
        }
    }
    if download {
        return Some("Un téléchargement est en cours.");
    }
    None
}

/// Premier travail en cours (enregistrement, préparation de l'audio, transcription, séparation des intervenants, rédaction,
/// traduction, téléchargement), ou `None`. Sert à refuser une mise à jour et à prévenir avant de fermer l'application.
pub fn busy_now(st: &AppState) -> Option<&'static str> {
    let recording = st.rec.lock().unwrap_or_else(|e| e.into_inner()).is_some();
    let s = st.store.lock().unwrap_or_else(|e| e.into_inner());
    busy_reason(
        recording,
        s.meetings.iter().filter(|m| !m.deleted).map(|m| (m.status.as_str(), m.report_state.as_str(), m.translate_state.as_str())),
        crate::engine::dl_active(),
    )
}

/// `{ busy, reason }` : la mise à jour est refusée tant qu'un travail est en cours (voir `busy_now`).
#[tauri::command]
pub fn update_busy(st: State<AppState>) -> R<Value> {
    let reason = busy_now(&st);
    Ok(json!({ "busy": reason.is_some(), "reason": reason }))
}

/// Quitte l'application après la confirmation de l'utilisateur (les moteurs sont arrêtés par `RunEvent::Exit`).
#[tauri::command]
pub fn app_quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(status: &'static str, report: &'static str, translate: &'static str) -> (&'static str, &'static str, &'static str) {
        (status, report, translate)
    }

    #[test]
    fn rien_en_cours() {
        assert_eq!(busy_reason(false, [m("ready", "", ""), m("error", "error", "error")].into_iter(), false), None);
        assert_eq!(busy_reason(false, [m("needs_engine", "", "")].into_iter(), false), None);
    }

    #[test]
    fn travaux_en_cours() {
        assert_eq!(busy_reason(true, std::iter::empty(), false), Some("Un enregistrement est en cours."));
        for s in ["queued", "converting", "transcribing", "diarizing"] {
            assert_eq!(busy_reason(false, [m(s, "", "")].into_iter(), false), Some("Une transcription est en cours."));
        }
        assert_eq!(busy_reason(false, [m("ready", "generating", "")].into_iter(), false), Some("Un compte-rendu est en cours de rédaction."));
        assert_eq!(busy_reason(false, [m("ready", "", "translating")].into_iter(), false), Some("Une traduction est en cours."));
        assert_eq!(busy_reason(false, std::iter::empty(), true), Some("Un téléchargement est en cours."));
    }

    #[test]
    fn etiquette_de_build() {
        let v = app_version();
        let label = v["label"].as_str().unwrap();
        assert!(label.starts_with(&format!("v{}-", env!("CARGO_PKG_VERSION"))), "{label}");
        let stamp = label.rsplit('-').take(2).collect::<Vec<_>>();
        assert_eq!(stamp[0].len(), 4, "{label}");
        assert_eq!(stamp[1].len(), 8, "{label}");
    }
}
