//! The menu-bar item and its menu.

use rune::AppHandle;

use crate::capture::capture;
use crate::settings::Main;

/// A 36×36 (18 pt @2x) template image: selection corners around a dot. macOS tints it.
fn tray_image() -> Vec<u8> {
    const N: i32 = 36;
    let (m, stroke, arm) = (5, 3, 10);
    let mut rgba = vec![0u8; (N * N * 4) as usize];
    for y in 0..N {
        for x in 0..N {
            let inside = (m..N - m).contains(&x) && (m..N - m).contains(&y);
            let (dx, dy) = ((x - m).min(N - 1 - m - x), (y - m).min(N - 1 - m - y));
            let corner = inside && ((dx < stroke && dy < arm) || (dy < stroke && dx < arm));
            let dot = (x * 2 - N + 1).pow(2) + (y * 2 - N + 1).pow(2) <= 4 * 16;
            if corner || dot { rgba[((y * N + x) * 4 + 3) as usize] = 255; }
        }
    }
    rgba
}

pub(crate) fn make_tray(handle: AppHandle<Main>) -> Result<tray_icon::TrayIcon, String> {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    let item = |id: &str, title: &str| MenuItem::with_id(id, title, true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &item("settings", "Open DScr"),
        &PredefinedMenuItem::separator(),
        &item("0", "Capture Screen"), &item("1", "Capture Window"), &item("2", "Capture Selection"),
        &PredefinedMenuItem::separator(),
        &item("quit", "Quit DScr"),
    ]).map_err(|e| e.to_string())?;
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        let id = e.id.0;
        handle.update(move |s: &mut Main, tx| match id.as_str() {
            "settings" => if let Some(h) = &s.handle { h.open() },
            "quit" => tx.quit(),
            m => if let Ok(m) = m.parse() { capture(s, m, tx) },
        });
    }));
    let icon = tray_icon::Icon::from_rgba(tray_image(), 36, 36).map_err(|e| e.to_string())?;
    tray_icon::TrayIconBuilder::new().with_menu(Box::new(menu)).with_icon_templated(icon)
        .with_tooltip("DScr").build().map_err(|e| e.to_string())
}
