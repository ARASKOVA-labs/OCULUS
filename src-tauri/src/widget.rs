//! The drop widget: what opening OCULUS shows on macOS. A small floating orb, above other windows
//! and movable; files dragged onto it open the ring of what they can become, and letting go on a
//! part converts them beside the originals. Clicking the orb offers the full app and Quit.
//!
//! It only knows about files dragged onto it: nothing watches the mouse or keyboard elsewhere.

use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "widget";
/// The orb at rest, and the ring when files are held over it (points).
const REST: f64 = 96.0;
const OPEN: f64 = 340.0;

/// Where the widget's middle is, so it opens and closes around the same point.
static CENTRE: Mutex<Option<(f64, f64)>> = Mutex::new(None);

/// Creates the widget near the bottom-right corner of the main screen, above the Dock.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let w = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("/".into()))
        .title("OCULUS")
        .inner_size(REST, REST)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;
    let (sw, sh) = match w.primary_monitor()? {
        Some(m) => {
            let s = m.size().to_logical::<f64>(m.scale_factor());
            (s.width, s.height)
        }
        None => (1440.0, 900.0),
    };
    place(&w, (sw - 120.0, sh - 200.0), REST);
    Ok(())
}

/// The page has drawn the orb (until then the window stays hidden, so it never flashes the
/// loading screen).
#[tauri::command]
pub fn widget_ready(app: AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
    }
}

fn place(w: &tauri::WebviewWindow, centre: (f64, f64), side: f64) {
    *CENTRE.lock().unwrap() = Some(centre);
    let _ = w.set_size(LogicalSize::new(side, side));
    let _ = w.set_position(LogicalPosition::new(centre.0 - side / 2.0, centre.1 - side / 2.0));
}

/// Opens the widget into the ring (files are held over it) or closes it back to the orb, around
/// the same middle. The middle is read afresh at rest, since the orb may have been moved.
#[tauri::command]
pub fn widget_open(app: AppHandle, open: bool) {
    let Some(w) = app.get_webview_window(LABEL) else { return };
    let side = if open { OPEN } else { REST };
    let current = || -> Option<(f64, f64)> {
        let scale = w.scale_factor().ok()?;
        let p = w.outer_position().ok()?.to_logical::<f64>(scale);
        let s = w.outer_size().ok()?.to_logical::<f64>(scale);
        Some((p.x + s.width / 2.0, p.y + s.height / 2.0))
    };
    let centre = if open {
        current()
    } else {
        *CENTRE.lock().unwrap()
    };
    if let Some(c) = centre.or_else(current) {
        place(&w, c, side);
    }
}

/// The orb's menu: the full app, or Quit.
#[tauri::command]
pub fn widget_menu(app: AppHandle) -> Result<(), String> {
    let w = app.get_webview_window(LABEL).ok_or("No widget")?;
    let open = MenuItem::with_id(&app, "widget-open", "Open OCULUS", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(&app, "widget-quit", "Quit OCULUS", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(&app, &[&open, &sep, &quit]).map_err(|e| e.to_string())?;
    w.popup_menu(&menu).map_err(|e| e.to_string())
}

/// What the orb's menu does.
pub fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "widget-open" => show_main(app),
        "widget-quit" => app.exit(0),
        _ => {}
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
