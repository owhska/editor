//! System clipboard integration.

#[cfg(not(test))]
use std::io::Write;
#[cfg(not(test))]
use std::sync::{LazyLock, Mutex};

#[cfg(not(test))]
static CLIPBOARD: LazyLock<Mutex<Option<arboard::Clipboard>>> =
    LazyLock::new(|| Mutex::new(arboard::Clipboard::new().ok()));

#[cfg(test)]
pub fn set_clipboard(_text: &str) {}

#[cfg(test)]
pub fn get_clipboard() -> Option<String> {
    None
}

#[cfg(not(test))]
pub fn set_clipboard(text: &str) {
    if let Ok(mut guard) = CLIPBOARD.lock() {
        if let Some(clipboard) = guard.as_mut() {
            let _ = clipboard.set_text(text);
        } else if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
            *guard = Some(clipboard);
        }
    }

    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
        }
    }
    if std::env::var_os("DISPLAY").is_some() {
        if let Ok(mut child) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
        }
    }
}

#[cfg(not(test))]
pub fn get_clipboard() -> Option<String> {
    if let Ok(mut guard) = CLIPBOARD.lock() {
        if let Some(clipboard) = guard.as_mut() {
            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    return Some(text);
                }
            }
        } else if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                *guard = Some(clipboard);
                if !text.is_empty() {
                    return Some(text);
                }
            } else {
                *guard = Some(clipboard);
            }
        }
    }

    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(output) = std::process::Command::new("wl-paste")
            .arg("--no-newline")
            .output()
        {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }
    }
    if std::env::var_os("DISPLAY").is_some() {
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }
    }
    None
}
