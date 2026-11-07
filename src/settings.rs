//! The settings window: capture mode, options and shortcuts, and the prefs behind them.

use rune::prefs::Prefs;
use rune::tok::motion;
use rune::{AppHandle, Color, Cx, El, Shortcut, ShortcutError, ShortcutId, TextStyle, WindowHandle, col, el, icon, row, spacer, text};
use rune_kit::{Kit, ShortcutRecorder, ShortcutRecorderState, Switch};

use crate::APP_ID;
use crate::capture::capture;
use crate::preview::Preview;
use crate::style::{Glass, ICONS, caption, group};

pub(crate) const MODES: [&str; 3] = ["Screen", "Window", "Selection"];
pub(crate) const DEFAULT_KEYS: [&str; 3] = ["ctrl+alt+1", "ctrl+alt+2", "ctrl+alt+3"];

#[derive(Default)]
pub(crate) struct Main {
    pub(crate) mode: usize,
    pub(crate) pointer: bool,
    pub(crate) delay: u32,
    pub(crate) open_at_launch: bool,
    pub(crate) recorders: [ShortcutRecorderState; 3],
    pub(crate) ids: [Option<ShortcutId>; 3],
    pub(crate) handle: Option<AppHandle<Main>>,
    pub(crate) preview: Option<(AppHandle<Preview>, WindowHandle)>,
    /// The menu-bar item; dropping it removes it, so the state keeps it.
    pub(crate) tray: Option<tray_icon::TrayIcon>,
}

pub(crate) fn init_main() -> Main {
    let p = Prefs::open(APP_ID);
    let key = |i: usize| p.get::<String>(&format!("key{i}")).unwrap_or_else(|| DEFAULT_KEYS[i].into());
    Main {
        mode: p.get::<usize>("mode").unwrap_or(0).min(2),
        pointer: p.get("pointer").unwrap_or(false),
        delay: p.get("delay").unwrap_or(0),
        open_at_launch: p.get("open_at_launch").unwrap_or(false),
        recorders: [0, 1, 2].map(|i| ShortcutRecorderState::new(key(i).parse().ok())),
        ..Main::default()
    }
}

pub(crate) fn save<T: serde::Serialize>(key: &str, value: &T) {
    if let Err(e) = Prefs::open(APP_ID).set(key, value) { eprintln!("dscr: couldn't save {key}: {e}"); }
}

pub(crate) fn rebind(s: &mut Main, m: usize, next: Option<&Shortcut>) -> Result<(), ShortcutError> {
    let h = s.handle.clone().ok_or(ShortcutError::NotRunning)?;
    let new = next.map(|sc| h.register_shortcut(sc.clone(), move |s: &mut Main, tx| capture(s, m, tx))).transpose()?;
    if let Some(old) = s.ids[m].take() { let _ = h.unregister_shortcut(old); }
    s.ids[m] = new;
    save(&format!("key{m}"), &next.map(|sc| sc.to_string()).unwrap_or_default());
    Ok(())
}

pub(crate) fn view_main(s: &Main, cx: &mut Cx<Main>) -> El {
    let mut k = Kit::neutral(cx.system_dark());
    let g = Glass::new(&k, cx.accent());
    k.c.bg_accent = g.accent; // switches follow the system accent like the tiles
    if !k.dark { k.c.layer_04 = Color::BLACK.alpha(0.14); } // an off switch's track, visible on light glass
    let label = |t: &str| text(t).font(k.ty.ui).color(k.c.text_base);

    let badge = g.accent_fill(el().size(40.0, 40.0).radius(11.0).continuous_corners().center())
        .child(icon(&ICONS[2], 22.0).color(Color::WHITE));
    let header = row().w_full().gap(12.0).items_center().px(20.0).pb(6.0).window_drag()
        .child(badge)
        .child(col().gap(2.0)
            .child(text("DScr").font(TextStyle::new(17.0, 650.0)).color(k.c.text_strong))
            .child(text("Capture the screen, a window or an area").font(k.ty.small).color(k.c.text_muted)));

    let mut tiles = row().w_full().gap(10.0).role("radiogroup").label("Capture area").focus_group();
    for (m, name) in MODES.iter().enumerate() {
        let on = s.mode == m;
        let ink = if on { Color::WHITE } else { k.c.text_base };
        let tile = col().key(format!("mode{m}")).role("radio").label(*name).checked(on).focusable(true)
            .grow(1.0).h(92.0).radius(14.0).continuous_corners().gap(8.0).center().motion(motion::SNAPPY)
            .on_press(move |s: &mut Main| { s.mode = m; save("mode", &m) })
            .child(icon(&ICONS[m], 26.0).color(if on { Color::WHITE } else { k.c.icon_muted }))
            .child(text(*name).font(k.ty.ui_medium).color(ink));
        let tile = if on { g.accent_fill(tile) } else { tile.fill(g.card).border(1.0, g.edge) };
        tiles = tiles.child(k.interactive(tile));
    }

    let step = |key: &'static str, name: &'static str, glyph: &'static str, f: fn(&mut Main)| {
        k.interactive(el().key(key).role("button").label(name).focusable(true).size(26.0, 26.0).radius(13.0).center()
            .on_press(move |s: &mut Main| { f(s); save("delay", &s.delay) })
            .child(text(glyph).font(k.ty.ui_medium).color(k.c.text_base)))
    };
    let stepper = row().gap(2.0).padding(2.0).items_center().radius(15.0).fill(g.well)
        .child(step("delay-", "Shorter delay", "−", |s| s.delay = s.delay.saturating_sub(1)))
        .child(text(if s.delay == 0 { "None".into() } else { format!("{} s", s.delay) })
            .font(k.ty.ui_medium).color(k.c.text_strong).w(44.0).center())
        .child(step("delay+", "Longer delay", "+", |s| s.delay = (s.delay + 1).min(60)));
    let options = group(&g, vec![
        row().child(label("Show pointer")).child(spacer())
            .child(Switch::new(s.pointer).label("Show pointer").hide_label()
                .on_toggle(|s: &mut Main| { s.pointer = !s.pointer; save("pointer", &s.pointer) }).build(&k)),
        row().child(label("Delay")).child(spacer()).child(stepper),
        row().child(label("Open window at launch")).child(spacer())
            .child(Switch::new(s.open_at_launch).label("Open window at launch").hide_label()
                .on_toggle(|s: &mut Main| { s.open_at_launch = !s.open_at_launch; save("open_at_launch", &s.open_at_launch) }).build(&k)),
    ]);

    let keys = group(&g, MODES.iter().enumerate().map(|(m, name)| {
        let rec = ShortcutRecorder::new(format!("key{m}"), &s.recorders[m], move |s: &mut Main| &mut s.recorders[m])
            .label(format!("Capture {name}")).width(150.0)
            .on_change(move |s, next, _| rebind(s, m, next))
            .build(&k);
        row().gap(10.0).child(icon(&ICONS[m], 16.0).color(k.c.icon_muted)).child(label(name)).child(spacer()).child(rec)
    }).collect());

    let mut shoot_label = row().gap(8.0).items_center()
        .child(icon(&ICONS[s.mode], 17.0).color(Color::WHITE))
        .child(text(format!("Capture {}", MODES[s.mode])).font(TextStyle::new(14.0, 600.0)).color(Color::WHITE));
    let caps = s.recorders[s.mode].keycaps();
    if !caps.is_empty() {
        shoot_label = shoot_label.child(text(caps.join("")).font(k.ty.small_medium).color(Color::WHITE.alpha(0.85))
            .px(7.0).py(2.0).radius(6.0).fill(Color::WHITE.alpha(0.18)));
    }
    let shoot = g.accent_fill(el().key("shoot").role("button").label("Take Screenshot").focusable(true)
        .w_full().h(46.0).radius(14.0).continuous_corners().center())
        .on_press_tx(|s: &mut Main, tx| { let m = s.mode; capture(s, m, tx) })
        .child(shoot_label);

    g.root(&k)
        .child(el().w_full().h(30.0).window_drag()) // clears the traffic lights
        .child(header)
        .child(col().w_full().grow(1.0).px(20.0).pt(12.0).gap(8.0)
            .child(tiles)
            .child(el().h(8.0))
            .child(caption(&k, "Options"))
            .child(options)
            .child(el().h(8.0))
            .child(caption(&k, "Shortcuts"))
            .child(keys)
            .child(spacer()))
        .child(col().w_full().px(20.0).pb(16.0).gap(10.0).items_center()
            .child(k.interactive_with(shoot, Color::WHITE.alpha(0.08), Color::BLACK.alpha(0.12)))
            .child(text("Lives in the menu bar · shortcuts work from any app").font(k.ty.small).color(k.c.text_faint)))
}
