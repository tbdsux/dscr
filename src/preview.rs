//! The preview window shown after a capture: save, copy or cancel.

use std::path::PathBuf;
use std::process::Command;

use rune::{AppHandle, Color, Cx, El, FileDialogOptions, Icon, Image, ImageCache, SurfaceFit, col, el, icon, row, spacer, text};
use rune_kit::Kit;

use crate::settings::Main;
use crate::style::Glass;

#[derive(Default)]
pub(crate) struct Preview {
    pub(crate) cache: Option<ImageCache>,
    pub(crate) image: Option<Image>,
    pub(crate) path: PathBuf,
    pub(crate) status: String,
    /// The settings window to show again once this one closes, if it was open before the capture.
    pub(crate) restore: Option<AppHandle<Main>>,
}

fn copy_to_clipboard(path: &std::path::Path) -> Result<(), String> {
    // Rune's clipboard doesn't write images yet, so AppleScript puts the PNG there.
    let script = format!("set the clipboard to (read (POSIX file {:?}) as «class PNGf»)", path.display().to_string());
    let out = Command::new("/usr/bin/osascript").arg("-e").arg(script).output().map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_owned()) }
}

fn default_name() -> String {
    let stamp = Command::new("date").arg("+%Y-%m-%d at %H.%M.%S").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()).unwrap_or_default();
    format!("Screenshot {stamp}.png")
}

/// A capsule toolbar button; `primary` takes the accent.
fn pill(k: &Kit, g: &Glass, key: &'static str, title: &'static str, glyph: Option<&'static Icon>, primary: bool) -> El {
    let ink = if primary { Color::WHITE } else { k.c.text_base };
    let mut b = row().key(key).role("button").label(title).focusable(true).h(32.0).px(16.0).gap(6.0).radius(16.0).center();
    if let Some(gl) = glyph { b = b.child(icon(gl, 14.0).color(ink)); }
    b = b.child(text(title).font(k.ty.ui_medium).color(ink));
    if primary { k.interactive_with(g.accent_fill(b), Color::WHITE.alpha(0.08), Color::BLACK.alpha(0.12)) } else { k.interactive(b) }
}

pub(crate) fn view_preview(s: &Preview, cx: &mut Cx<Preview>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let g = Glass::new(&k, cx.accent());
    let size = s.image.as_ref().and_then(Image::size);
    let picture = match (&s.image, size) {
        (Some(img), Some([w, h])) if h > 0 => el().aspect_fit(w as f32 / h as f32).radius(10.0).continuous_corners()
            .border(1.0, g.edge).shadow(10.0, 32.0, Color::BLACK.alpha(0.35))
            .pixels_fit(img.pixels(), SurfaceFit::Fill),
        _ => el(),
    };
    let stage = col().w_full().grow(1.0).center().px(24.0).pt(8.0).child(picture);

    let cancel = pill(&k, &g, "cancel", "Cancel", None, false).on_press_tx(|_: &mut Preview, tx| tx.close_window());
    let copy = pill(&k, &g, "copy", "Copy", Some(&rune_kit::glyphs::COPY), false).on_press(|s: &mut Preview| {
        s.status = match copy_to_clipboard(&s.path) { Ok(()) => "Copied to clipboard".into(), Err(e) => format!("Copy failed: {e}") };
    });
    let save = pill(&k, &g, "save", "Save…", None, true).on_press_tx(|_: &mut Preview, tx| {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let options = FileDialogOptions { title: "Save Screenshot".into(), directory: Some(home.join("Pictures")), file_name: Some(default_name()), ..Default::default() }
            .filter("PNG image", &["png"]);
        tx.save_file(options, |s: &mut Preview, result, tx| match result {
            Ok(Some(dst)) => match std::fs::copy(&s.path, &dst) {
                Ok(_) => tx.close_window(),
                Err(e) => s.status = format!("Save failed: {e}"),
            },
            Ok(None) => {}
            Err(e) => s.status = format!("Save failed: {e}"),
        });
    });
    let toolbar = row().gap(4.0).padding(4.0).radius(20.0).continuous_corners().fill(g.card).border(1.0, g.edge)
        .shadow(4.0, 16.0, Color::BLACK.alpha(if g.dark { 0.3 } else { 0.08 }))
        .child(cancel).child(copy).child(save);
    let info = col().gap(2.0)
        .child(text(size.map(|[w, h]| format!("{w} × {h}")).unwrap_or_default()).font(k.ty.ui_medium).color(k.c.text_strong))
        .child(text(if s.status.is_empty() { "PNG".to_owned() } else { s.status.clone() }).font(k.ty.small).color(k.c.text_muted));

    g.root(&k)
        .child(el().w_full().h(30.0).window_drag())
        .child(stage)
        .child(row().w_full().px(20.0).py(16.0).items_center().child(info).child(spacer()).child(toolbar))
}
