//! Keeps one copy of the app running; a second launch shows the first one's settings.

use std::path::PathBuf;

use rune::AppHandle;

use crate::APP_ID;
use crate::settings::Main;

/// `~/Library/Application Support/<app id>`, where prefs and the instance lock live.
fn app_dir() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join("Library/Application Support").join(APP_ID)
}

/// Holds the single-instance lock for the process's lifetime. A second launch finds it taken,
/// pokes the running instance through the socket (which shows its settings) and exits.
pub(crate) fn single_instance() -> Option<std::fs::File> {
    let dir = app_dir();
    let _ = std::fs::create_dir_all(&dir);
    let (lock_path, sock) = (dir.join("instance.lock"), dir.join("instance.sock"));
    let lock = std::fs::File::create(&lock_path).ok()?; // can't lock: run anyway rather than refuse to start
    match lock.try_lock() {
        Ok(()) => {
            let _ = std::fs::remove_file(&sock); // left by a crashed instance
            Some(lock)
        }
        Err(std::fs::TryLockError::WouldBlock) => {
            let _ = std::os::unix::net::UnixStream::connect(&sock);
            std::process::exit(0);
        }
        Err(e) => { eprintln!("dscr: single-instance lock failed: {e}"); None }
    }
}

/// Opens the settings window whenever another launch connects.
pub(crate) fn listen_for_relaunch(handle: AppHandle<Main>) {
    let sock = app_dir().join("instance.sock");
    let Ok(listener) = std::os::unix::net::UnixListener::bind(&sock) else { return eprintln!("dscr: relaunch socket unavailable") };
    std::thread::spawn(move || for _ in listener.incoming() { handle.open() });
}
