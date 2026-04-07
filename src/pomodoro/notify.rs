/// Send a desktop notification fire-and-forget. Never panics or returns Err.
/// Falls back gracefully if the OS notification tool is unavailable.
pub fn send_notification(title: &str, body: &str) {
    // Respect FOCUS_POMODORO_SOUND=false — currently audio is terminal bell only
    // (no sound file playback in scope); this env var is read without error.
    let _ = std::env::var("FOCUS_POMODORO_SOUND");

    let result = spawn_notification(title, body);
    if let Err(e) = result {
        eprintln!("notification error: {e}");
    }
}

fn spawn_notification(title: &str, body: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification \"{body}\" with title \"{title}\"",
            body = body.replace('"', "\\\""),
            title = title.replace('"', "\\\""),
        );
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("notify-send")
            .args([title, body])
            .spawn()?;
    }

    // Other platforms: no-op
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (title, body);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_notification_does_not_panic() {
        send_notification("Focus", "Work phase complete!");
    }

    #[test]
    fn send_notification_with_quotes_does_not_panic() {
        send_notification("Focus \"test\"", "Body with \"quotes\"");
    }

    #[test]
    fn focus_pomodoro_sound_env_read_without_error() {
        std::env::set_var("FOCUS_POMODORO_SOUND", "false");
        send_notification("test", "body");
        std::env::remove_var("FOCUS_POMODORO_SOUND");
    }
}
