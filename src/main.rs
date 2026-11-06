//! dscr: a Linux Mint style screenshot tool for macOS. Capture is done by the system's
//! `screencapture`; the app picks the mode, options and global keybinds, then shows a preview.

mod capture;
mod instance;
mod preview;
mod settings;
mod style;
mod tray;

use rune::{App, AppSpec, ImageCache, WindowOptions, WindowSpec};

use crate::preview::{Preview, view_preview};
use crate::settings::{MODES, init_main, rebind, view_main};

const APP_ID: &str = "dev.tbdsux.dscr";

fn main() {
    let _instance = instance::single_instance();
    let mut app = App::new(AppSpec { min_size: Some((400.0, 620.0)), ..AppSpec::new("DScr", (420.0, 640.0), init_main, view_main) })
        .window_options(WindowOptions { transparent: true, ..Default::default() })
        .windowless() // starts in the menu bar only; closing the window later keeps the app, hotkeys and menu-bar item running
        .accessory(); // no Dock icon: the menu-bar item is how you get back to it
    let preview_spec = AppSpec {
        setup: Some(|p: &mut Preview, waker| p.cache = Some(ImageCache::new(waker))),
        min_size: Some((420.0, 300.0)),
        ..AppSpec::new("DScr Preview", (760.0, 540.0), Preview::default, view_preview)
    };
    let preview_app = App::new(preview_spec).on_close(|p: &mut Preview, _| if let Some(h) = p.restore.take() { h.open() });
    let preview = app.window(WindowSpec::from_app("preview", preview_app).options(WindowOptions { visible: false, transparent: true, ..Default::default() }));
    let handle = app.handle();
    if _instance.is_some() { instance::listen_for_relaunch(handle.clone()); }
    let shared = handle.clone();
    handle.update(move |s, _| {
        // Built here, on the main thread once AppKit is running, as the status bar requires.
        s.tray = tray::make_tray(shared.clone()).map_err(|e| eprintln!("dscr: no menu-bar item: {e}")).ok();
        s.handle = Some(shared);
        s.preview = Some(preview);
        for m in 0..3 {
            let Some(sc) = s.recorders[m].shortcut().cloned() else { continue };
            if let Err(e) = rebind(s, m, Some(&sc)) {
                eprintln!("dscr: couldn't register {sc} for {}: {e}", MODES[m]);
                s.recorders[m].mark_unregistered();
            }
        }
    });
    app.run();
}
