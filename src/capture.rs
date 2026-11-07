//! Taking the screenshot with macOS's `screencapture` and handing it to the preview window.

use std::process::Command;
use std::time::Duration;

use rune::Tx;

use crate::settings::Main;

/// `screencapture` flags for a mode. `-C` (pointer) only applies to non-interactive captures.
pub(crate) fn flags(mode: usize, pointer: bool, delay: u32) -> Vec<String> {
    let mut a: Vec<String> = match mode { 1 => vec!["-i".into(), "-w".into()], 2 => vec!["-i".into(), "-s".into()], _ => vec![] };
    if pointer { a.push("-C".into()); }
    if delay > 0 { a.push("-T".into()); a.push(delay.to_string()); }
    a
}

pub(crate) fn capture(s: &mut Main, mode: usize, _: &mut Tx) {
    let (Some(main), Some((preview, window))) = (s.handle.clone(), s.preview.clone()) else { return };
    let was_visible = main.is_visible();
    main.close(); // keep the settings window out of the shot
    let args = flags(mode, s.pointer, s.delay);
    std::thread::spawn(move || {
        // shortcut: fixed wait for the window's hide animation, upgrade to a visibility callback if shots catch it
        std::thread::sleep(Duration::from_millis(250));
        let path = std::env::temp_dir().join("dscr-capture.png");
        let _ = std::fs::remove_file(&path);
        let ok = Command::new("/usr/sbin/screencapture").args(&args).arg(&path).status().is_ok_and(|st| st.success());
        match std::fs::read(&path) {
            Ok(bytes) if ok && !bytes.is_empty() => {
                let restore = was_visible.then(|| main.clone());
                let (sent, decoded) = std::sync::mpsc::channel();
                preview.update(move |p, _| {
                    p.image = p.cache.as_ref().map(|c| c.load(bytes));
                    let _ = sent.send(p.image.clone());
                    p.path = path;
                    p.status.clear();
                    p.restore = restore;
                });
                // A full-screen shot takes long enough to decode that the view would lay out an
                // empty box and never resize it: wait for the size, then re-run the view.
                if let Ok(Some(img)) = decoded.recv() { let _ = img.wait(); }
                preview.update(|_, _| {});
                window.open();
            }
            // Escape in an interactive capture writes no file: just go back.
            _ => if was_visible { main.open() },
        }
    });
}

#[cfg(test)]
mod tests {
    use super::flags;

    #[test]
    fn flags_per_mode() {
        assert!(flags(0, false, 0).is_empty());
        assert_eq!(flags(0, true, 3), ["-C", "-T", "3"]);
        assert_eq!(flags(1, false, 0), ["-i", "-w"]);
        assert_eq!(flags(2, false, 2), ["-i", "-s", "-T", "2"]);
    }
}
