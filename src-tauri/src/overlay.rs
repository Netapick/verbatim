//! Enregistrement en arrière-plan : pendant un enregistrement, la fenêtre de Verbatim est masquée (plus de bouton dans la
//! barre des tâches, seule reste l'icône de la zone de notification) et une petite pastille flottante (toujours au premier
//! plan) offre Pause / Terminer. L'icône de notification donne les mêmes actions. L'arrêt passe par la fenêtre principale (événement `overlay-stop`) : c'est elle qui connaît le titre saisi.

use crate::{AppState, R};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    window::{Color, Effect, EffectsBuilder},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Wry,
};

const LABEL: &str = "overlay";
const SIZE: (f64, f64) = (340.0, 48.0); // la fenêtre épouse la pastille (découpée en forme de pilule, voir `round_corners`)

/// Éléments de l'icône de notification, conservés pour les activer/désactiver et changer les libellés.
pub struct TrayState {
    tray: TrayIcon,
    show: MenuItem<Wry>,
    pause: MenuItem<Wry>,
    stop: MenuItem<Wry>,
}

/// Crée l'icône de la zone de notification (au démarrage de l'application).
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Afficher Verbatim", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause / Reprendre", false, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Terminer l'enregistrement", false, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &PredefinedMenuItem::separator(app)?, &pause, &stop])?;
    let mut b = TrayIconBuilder::with_id("main")
        .tooltip("Verbatim")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| match ev.id.as_ref() {
            "show" => show_main(app),
            "pause" => {
                let st = app.state::<AppState>();
                if let Err(e) = crate::recorder::toggle_pause(&st) {
                    eprintln!("pause : {e}");
                }
            }
            "stop" => {
                let _ = app.emit_to("main", "overlay-stop", ());
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    let tray = b.build(app)?;
    app.manage(TrayState { tray, show, pause, stop });
    Ok(())
}

/// Ramène la fenêtre principale au premier plan.
pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Un enregistrement démarre : active les actions de l'icône, affiche la pastille et masque la fenêtre principale.
/// (Les fenêtres se créent hors du fil de la commande : sous Windows, c'est ce qui évite un blocage.)
pub fn recording_started(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        if let Some(t) = app.try_state::<TrayState>() {
            let _ = t.pause.set_enabled(true);
            let _ = t.stop.set_enabled(true);
            let _ = t.tray.set_tooltip(Some("Verbatim — enregistrement en cours"));
        }
        match open_overlay(&app) {
            // masquée, la fenêtre n'occupe plus la barre des tâches ; la pastille et l'icône de notification la rouvrent
            Ok(()) => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            Err(e) => eprintln!("pastille d'enregistrement : {e}"),
        }
    });
}

/// L'enregistrement est terminé : ferme la pastille et rend la main à la fenêtre principale.
pub fn recording_stopped(app: &AppHandle) {
    if let Some(t) = app.try_state::<TrayState>() {
        let _ = t.pause.set_enabled(false);
        let _ = t.stop.set_enabled(false);
        let _ = t.tray.set_tooltip(Some("Verbatim"));
    }
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.destroy();
    }
    show_main(app);
}

fn open_overlay(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(LABEL).is_some() {
        return Ok(());
    }
    let mut b = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("overlay.html".into()))
        .title("Verbatim")
        .inner_size(SIZE.0, SIZE.1)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        // flou de l'écran derrière la pastille (le CSS ne le peut pas : il ne voit pas le bureau) + teinte sombre
        .effects(EffectsBuilder::new().effect(Effect::Blur).color(Color(22, 26, 33, 150)).build())
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false);
    // en haut et au centre de l'écran principal
    if let Ok(Some(m)) = app.primary_monitor() {
        let sf = m.scale_factor();
        let (pos, size) = (m.position().to_logical::<f64>(sf), m.size().to_logical::<f64>(sf));
        b = b.position(pos.x + (size.width - SIZE.0) / 2.0, pos.y + 8.0);
    }
    let w = b.build()?;
    #[cfg(windows)]
    round_corners(&w);
    let _ = &w;
    Ok(())
}

/// Le flou s'applique à tout le rectangle de la fenêtre : on la découpe en pilule pour qu'il suive la pastille.
#[cfg(windows)]
fn round_corners(w: &tauri::WebviewWindow) {
    use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn};
    let (Ok(hwnd), Ok(size)) = (w.hwnd(), w.inner_size()) else { return };
    let (pw, ph) = (size.width as i32, size.height as i32);
    unsafe {
        let rgn = CreateRoundRectRgn(0, 0, pw + 1, ph + 1, ph, ph);
        SetWindowRgn(hwnd, Some(rgn), true);
    }
}

/// Libellés de l'icône de notification, dans la langue de l'interface (le texte vient de l'interface).
#[tauri::command]
pub fn tray_labels(st: tauri::State<TrayState>, show: String, pause: String, stop: String) -> R<()> {
    st.show.set_text(show).map_err(|e| e.to_string())?;
    st.pause.set_text(pause).map_err(|e| e.to_string())?;
    st.stop.set_text(stop).map_err(|e| e.to_string())
}

/// Actions de la pastille : "show" ramène la fenêtre principale, "stop" demande l'arrêt à l'interface.
#[tauri::command]
pub fn overlay_action(app: AppHandle, action: String) {
    match action.as_str() {
        "show" => show_main(&app),
        "stop" => {
            let _ = app.emit_to("main", "overlay-stop", ());
        }
        _ => {}
    }
}
