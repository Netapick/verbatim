//! Enregistrement en arrière-plan : pendant un enregistrement, la fenêtre de Verbatim est masquée (plus de bouton dans la
//! barre des tâches, seule reste l'icône de la zone de notification) et une petite pastille flottante (toujours au premier
//! plan) offre Pause / Terminer. L'icône de notification donne les mêmes actions. L'arrêt passe par la fenêtre principale (événement `overlay-stop`) : c'est elle qui connaît le titre saisi.

use crate::{AppState, R};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Wry,
};

const LABEL: &str = "overlay";
const SIZE: (f64, f64) = (340.0, 48.0); // la fenêtre épouse la pastille (découpée en forme de pilule, voir `round_corners`)

/// La pastille est ancrée dans la barre des tâches (par défaut) : elle ne se glisse pas tant qu'elle n'est pas détachée.
static DOCKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Début d'un glissement : position du curseur (écran) et de la fenêtre, en pixels logiques.
static DRAG: std::sync::Mutex<Option<(f64, f64, f64, f64)>> = std::sync::Mutex::new(None);

/// Déplace la pastille à la souris, sans le glissement natif de Windows : celui-ci redessine la barre de titre de la fenêtre
/// pendant le déplacement. `phase` : "start" (saisie), "move" (déplacement), autre valeur = fin.
#[tauri::command]
pub fn overlay_drag(app: AppHandle, phase: String, x: f64, y: f64) {
    let Some(w) = app.get_webview_window(LABEL) else { return };
    if DOCKED.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let mut d = DRAG.lock().unwrap_or_else(|e| e.into_inner());
    match phase.as_str() {
        "start" => {
            if let (Ok(p), Ok(sf)) = (w.outer_position(), w.scale_factor()) {
                *d = Some((x, y, p.x as f64 / sf, p.y as f64 / sf));
            }
        }
        "move" => {
            if let Some((sx, sy, wx, wy)) = *d {
                let (lx, ly) = keep_on_screen(&w, wx + x - sx, wy + y - sy);
                let _ = w.set_position(tauri::LogicalPosition::new(lx, ly));
            }
        }
        _ => *d = None,
    }
}

/// Ancre (`true`, par défaut) la pastille dans la barre des tâches, ou la détache (`false`) : elle se place alors en haut au
/// centre de l'écran principal et peut être glissée.
#[tauri::command]
pub fn overlay_dock(app: AppHandle, docked: bool) {
    DOCKED.store(docked, std::sync::atomic::Ordering::SeqCst);
    if let Some(w) = app.get_webview_window(LABEL) {
        place(&app, &w, docked);
    }
}

/// Bord gauche de la zone de notification de la barre des tâches (pixels physiques), si Windows la fournit.
#[cfg(windows)]
fn tray_left() -> Option<i32> {
    use windows::core::w;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, FindWindowW, GetWindowRect};
    unsafe {
        let bar = FindWindowW(w!("Shell_TrayWnd"), None).ok()?;
        let tray = FindWindowExW(Some(bar), None, w!("TrayNotifyWnd"), None).ok()?;
        let mut r = RECT::default();
        GetWindowRect(tray, &mut r).ok()?;
        Some(r.left)
    }
}
#[cfg(not(windows))]
fn tray_left() -> Option<i32> {
    None
}

/// Remet la pastille au-dessus de tout, barre des tâches comprise : Windows remonte la barre au-dessus des fenêtres « au premier
/// plan » dès qu'on la sollicite, et `set_always_on_top` ne refait rien si l'état n'a pas changé, d'où l'appel direct.
fn raise(w: &tauri::WebviewWindow) {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
        if let Ok(h) = w.hwnd() {
            unsafe {
                let _ = SetWindowPos(HWND(h.0), Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }
    #[cfg(not(windows))]
    let _ = w.set_always_on_top(true);
}

/// Place la pastille : ancrée, juste au-dessus de la barre des tâches, à droite (sous la barre si elle est en haut) ;
/// détachée, en haut au centre de l'écran principal.
fn place(app: &AppHandle, w: &tauri::WebviewWindow, docked: bool) {
    let Ok(sz) = w.outer_size() else { return };
    if let Some((x, y)) = target(app, sz.width as i32, sz.height as i32, docked) {
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    }
}

/// Position (pixels physiques) de la pastille de taille `ww` × `wh`, calculable avant la création de la fenêtre.
fn target(app: &AppHandle, ww: i32, wh: i32, docked: bool) -> Option<(i32, i32)> {
    let m = app.primary_monitor().ok()??;
    let (mp, ms) = (m.position(), m.size());
    let sf = m.scale_factor();
    let (x, y);
    if !docked {
        x = mp.x + (ms.width as i32 - ww) / 2;
        y = mp.y + (8.0 * sf) as i32;
    } else {
        // barre des tâches en bas ou en haut : la pastille s'y loge, centrée dans la bande, à gauche de la zone de notification.
        // Sans bande visible (barre masquée ou sur un côté) : en bas à droite de la zone utile.
        let wa = m.work_area();
        let (mon_bottom, wa_bottom) = (mp.y + ms.height as i32, wa.position.y + wa.size.height as i32);
        let (band_bottom, band_top) = (mon_bottom - wa_bottom, wa.position.y - mp.y);
        let margin = (8.0 * sf) as i32;
        if band_bottom > 0 || band_top > 0 {
            let (top, band) = if band_bottom > 0 { (wa_bottom, band_bottom) } else { (mp.y, band_top) };
            y = if band >= wh {
                top + (band - wh) / 2
            } else if band_bottom > 0 {
                top + band - wh // bande plus basse que la pastille : alignée sur le bas de l'écran
            } else {
                top
            };
            x = tray_left().map(|l| l - ww - margin).unwrap_or(mp.x + ms.width as i32 - ww - (300.0 * sf) as i32);
        } else {
            y = wa_bottom - wh - margin;
            x = wa.position.x + wa.size.width as i32 - ww - margin;
        }
    }
    Some((x.clamp(mp.x, (mp.x + ms.width as i32 - ww).max(mp.x)), y))
}

/// Garde la pastille entièrement visible : dans l'écran qui la contient, ou, avec plusieurs écrans, dans l'ensemble des
/// écrans (elle peut passer de l'un à l'autre, jamais sortir du bureau ni se loger dans un vide entre deux écrans).
/// Position et résultat en pixels logiques.
fn keep_on_screen(w: &tauri::WebviewWindow, lx: f64, ly: f64) -> (f64, f64) {
    let (Ok(mons), Ok(sf), Ok(sz)) = (w.available_monitors(), w.scale_factor(), w.outer_size()) else { return (lx, ly) };
    if mons.is_empty() {
        return (lx, ly);
    }
    let (ww, wh) = (sz.width as f64, sz.height as f64);
    // rectangles des écrans en pixels physiques : (gauche, haut, droite, bas)
    let rects: Vec<(f64, f64, f64, f64)> = mons
        .iter()
        .map(|m| {
            let (p, s) = (m.position(), m.size());
            (p.x as f64, p.y as f64, p.x as f64 + s.width as f64, p.y as f64 + s.height as f64)
        })
        .collect();
    let (px, py) = (lx * sf, ly * sf);
    let clamp = |r: (f64, f64, f64, f64), x: f64, y: f64| (x.clamp(r.0, (r.2 - ww).max(r.0)), y.clamp(r.1, (r.3 - wh).max(r.1)));
    // 1. dans l'enveloppe de tous les écrans
    let hull = rects.iter().fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN), |a, r| (a.0.min(r.0), a.1.min(r.1), a.2.max(r.2), a.3.max(r.3)));
    let (mut x, mut y) = clamp(hull, px, py);
    // 2. si le centre tombe hors de tout écran (vide entre deux écrans de tailles différentes), retour dans l'écran le plus proche
    let (cx, cy) = (x + ww / 2.0, y + wh / 2.0);
    if !rects.iter().any(|r| cx >= r.0 && cx < r.2 && cy >= r.1 && cy < r.3) {
        let dist = |r: &(f64, f64, f64, f64)| {
            let (dx, dy) = ((r.0 - cx).max(0.0).max(cx - r.2), (r.1 - cy).max(0.0).max(cy - r.3));
            dx * dx + dy * dy
        };
        if let Some(near) = rects.iter().min_by(|a, b| dist(a).total_cmp(&dist(b))) {
            (x, y) = clamp(*near, x, y);
        }
    }
    (x / sf, y / sf)
}

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

/// Ramène la fenêtre principale au premier plan. La pastille se masque alors : la fenêtre affiche déjà l'enregistrement
/// (Pause / Terminer), et la pastille, toujours au premier plan, se redessinait de travers par-dessus (barre de titre grise).
pub fn show_main(app: &AppHandle) {
    if let Some(o) = app.get_webview_window(LABEL) {
        let _ = o.hide();
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// La fenêtre principale a été réduite pendant un enregistrement : la pastille réapparaît et la fenêtre se masque tout à fait
/// (plus de bouton dans la barre des tâches, comme au démarrage de l'enregistrement) ; la pastille et l'icône de notification
/// la rouvrent.
pub fn main_minimized(app: &AppHandle) {
    if let Some(o) = app.get_webview_window(LABEL) {
        let _ = o.show();
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
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
        .always_on_top(true)
        .skip_taskbar(true)
        // un clic sur la pastille ne prend pas le focus à l'application en cours (visio, présentation…)
        .focusable(false)
        .focused(false);
    // position de départ (ancrée) calculée d'avance : créer la fenêtre invisible puis l'afficher laissait la pastille à moitié dessinée
    if let Ok(Some(m)) = app.primary_monitor() {
        let sf = m.scale_factor();
        if let Some((x, y)) = target(app, (SIZE.0 * sf).round() as i32, (SIZE.1 * sf).round() as i32, true) {
            b = b.position(x as f64 / sf, y as f64 / sf);
        }
    }
    DOCKED.store(true, std::sync::atomic::Ordering::SeqCst); // à chaque enregistrement, la pastille repart ancrée
    let w = b.build()?;
    #[cfg(windows)]
    round_corners(&w);
    raise(&w);
    // ancrée dans la barre, la pastille doit rester par-dessus : la barre se remonte quand on la sollicite
    let app2 = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let Some(w) = app2.get_webview_window(LABEL) else { break };
        if DOCKED.load(std::sync::atomic::Ordering::SeqCst) {
            raise(&w);
        }
    });
    Ok(())
}

/// La fenêtre est un rectangle : on la découpe en pilule pour que seule la pastille soit visible (coins transparents).
/// Pas de flou de l'écran derrière : ceux de Windows (Blur, Acrylic, fond système) ignorent cette découpe et laissent un rectangle.
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
