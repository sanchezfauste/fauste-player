//! Time and number formatting for the UI.

fn split(secs: f64) -> (u64, u64, u64) {
    let total = if secs.is_finite() && secs > 0.0 {
        secs.floor() as u64
    } else {
        0
    };
    (total / 3600, total % 3600 / 60, total % 60)
}

/// `mm:ss`, or `h:mm:ss` from one hour on.
pub fn clock(secs: f64) -> String {
    let (h, m, s) = split(secs);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// The big countdown: (`-mm:ss`, `.t`) with tenths shown smaller.
pub fn countdown(remaining_secs: f64) -> (String, String) {
    let r = if remaining_secs.is_finite() {
        remaining_secs.max(0.0)
    } else {
        0.0
    };
    let tenths = (r * 10.0).floor() as u64;
    (
        format!("-{}", clock((tenths / 10) as f64)),
        format!(".{}", tenths % 10),
    )
}

/// Digits for the `#` column: at least two, more for long playlists.
pub fn number_width(len: usize) -> usize {
    len.max(1).to_string().len().max(2)
}
