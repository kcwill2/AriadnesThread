//! Keeps the live `\r` progress line within one terminal row.
//!
//! `\r` only returns to the start of the CURRENT screen row: a frame wider
//! than the terminal wraps, and every redraw then lands on a fresh row
//! instead of overwriting. Frames are therefore fitted to the terminal
//! width by eliding the MIDDLE of the message, which keeps the leading
//! count (`34/327 TUs`) and the trailing file name readable.

/// Marker inserted where characters were removed.
pub const ELISION_MARKER: &str = "...";

/// Shortens `text` to at most `max_chars` characters by replacing its middle
/// with [`ELISION_MARKER`]. Text that already fits is returned unchanged.
pub fn elide_middle(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    let marker_len = ELISION_MARKER.chars().count();
    if max_chars <= marker_len {
        return chars[..max_chars].iter().collect();
    }
    let kept = max_chars - marker_len;
    let head_len = kept / 2;
    let tail_len = kept - head_len;
    let head: String = chars[..head_len].iter().collect();
    let tail: String = chars[chars.len() - tail_len..].iter().collect();
    format!("{head}{ELISION_MARKER}{tail}")
}

/// Keeps the END of `text` (the file name of a path) within `max_chars`.
fn elide_left(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    let tail_len = max_chars.saturating_sub(ELISION_MARKER.chars().count());
    let tail: String = chars[chars.len() - tail_len..].iter().collect();
    format!("{ELISION_MARKER}{tail}")
}

/// Separator between a frame message's count and its detail (`3/9 TUs  a.cpp`).
const DETAIL_SEPARATOR: &str = "  ";

/// Width-fitted message for a step frame whose fixed prefix (stage label,
/// phase title, bar and percent) takes `prefix_chars` columns. One column is
/// left free so the cursor never sits in the last cell and forces a wrap.
///
/// Priority: the leading count stays whole; the detail after it (a path)
/// loses characters from the LEFT so the file name survives; when only the
/// count fits, the detail is dropped.
pub fn fit_message(message: &str, prefix_chars: usize, terminal_width: usize) -> String {
    let budget = terminal_width.saturating_sub(prefix_chars + 1);
    if message.chars().count() <= budget {
        return message.to_string();
    }
    let Some((count, detail)) = message.split_once(DETAIL_SEPARATOR) else {
        return elide_middle(message, budget);
    };
    let count_len = count.chars().count();
    let detail_budget = budget.saturating_sub(count_len + DETAIL_SEPARATOR.len());
    if detail_budget > ELISION_MARKER.len() {
        format!(
            "{count}{DETAIL_SEPARATOR}{}",
            elide_left(detail, detail_budget)
        )
    } else if count_len <= budget {
        count.to_string()
    } else {
        elide_middle(count, budget)
    }
}

/// Column count of the terminal stdout is attached to, or `None` when stdout
/// is not a terminal (piped/redirected output keeps full frames).
/// Order: `COLUMNS`, then the platform console query, then 80.
pub fn detect_terminal_width() -> Option<usize> {
    use std::io::IsTerminal;
    if !std::io::stdout().is_terminal() {
        return None;
    }
    let from_env = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|&columns| columns > 0);
    Some(from_env.or_else(console_width).unwrap_or(80))
}

#[cfg(windows)]
fn console_width() -> Option<usize> {
    #[repr(C)]
    #[derive(Default)]
    struct Coord {
        x: i16,
        y: i16,
    }
    #[repr(C)]
    #[derive(Default)]
    struct SmallRect {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }
    #[repr(C)]
    #[derive(Default)]
    struct ScreenBufferInfo {
        size: Coord,
        cursor_position: Coord,
        attributes: u16,
        window: SmallRect,
        maximum_window_size: Coord,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(std_handle: u32) -> *mut std::ffi::c_void;
        fn GetConsoleScreenBufferInfo(
            console: *mut std::ffi::c_void,
            info: *mut ScreenBufferInfo,
        ) -> i32;
    }
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    let mut info = ScreenBufferInfo::default();
    // SAFETY: both calls only read the process's own stdout handle and
    // write into the stack-owned, correctly laid out `info`.
    let ok = unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) };
    let columns = i32::from(info.window.right) - i32::from(info.window.left) + 1;
    (ok != 0 && columns > 0).then_some(columns as usize)
}

#[cfg(not(windows))]
fn console_width() -> Option<usize> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_unchanged() {
        assert_eq!(
            elide_middle("12/24 TUs  src/a.cpp", 40),
            "12/24 TUs  src/a.cpp"
        );
    }

    #[test]
    fn long_text_keeps_head_and_tail_within_budget() {
        let text =
            "34/327 TUs  C:/Users/admin/Documents/Code/boost_1_92_0/libs/cobalt/src/channel.cpp";
        let fitted = elide_middle(text, 40);
        assert_eq!(fitted.chars().count(), 40);
        assert!(
            fitted.starts_with("34/327 TUs"),
            "count stays readable: {fitted}"
        );
        assert!(
            fitted.ends_with("src/channel.cpp"),
            "file name stays readable: {fitted}"
        );
        assert!(fitted.contains(ELISION_MARKER));
    }

    #[test]
    fn budget_smaller_than_marker_truncates_plainly() {
        assert_eq!(elide_middle("abcdefgh", 2), "ab");
        assert_eq!(elide_middle("abcdefgh", 0), "");
    }

    #[test]
    fn elision_counts_characters_not_bytes() {
        let text = "\u{2588}".repeat(30);
        assert_eq!(elide_middle(&text, 10).chars().count(), 10);
    }

    #[test]
    fn fit_message_leaves_one_free_column() {
        let message = "x".repeat(100);
        let fitted = fit_message(&message, 60, 80);
        assert_eq!(fitted.chars().count(), 80 - 60 - 1);
    }

    #[test]
    fn fit_message_with_no_room_yields_empty_message() {
        assert_eq!(fit_message("12/24 TUs", 90, 80), "");
    }

    #[test]
    fn fit_message_keeps_count_whole_and_path_tail() {
        let message = "34/327 TUs  C:/Users/admin/Code/boost_1_92_0/libs/cobalt/src/channel.cpp";
        let fitted = fit_message(message, 0, 41);
        assert_eq!(fitted.chars().count(), 40);
        assert!(fitted.starts_with("34/327 TUs  ..."), "{fitted}");
        assert!(fitted.ends_with("/src/channel.cpp"), "{fitted}");
    }

    #[test]
    fn fit_message_drops_detail_when_only_the_count_fits() {
        let message = "34/327 TUs  C:/a/very/long/path/channel.cpp";
        assert_eq!(fit_message(message, 0, 15), "34/327 TUs");
    }
}
