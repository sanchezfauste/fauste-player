//! Playlist files (M3U, M3U8, PLS) and cart page files (Phase 2 spec P2.7).
//!
//! The parsers are tolerant: unknown lines are ignored, relative paths are
//! resolved against the file's folder, `file://` URLs are decoded, streams
//! are skipped and counted, and nothing panics on hostile input (fuzzed).
//! Paths are only resolved lexically: a file that does not exist still gets
//! an entry, shown as unavailable, so nothing is dropped silently.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use fp_model::{CartEdit, CartKind, CartPage, CartPageImport, Library, Limits};

/// One entry read from a playlist file.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedEntry {
    pub path: PathBuf,
    /// `Artist - Title` hint from `#EXTINF` or `TitleN`.
    pub title: Option<String>,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImportedPlaylist {
    pub entries: Vec<ImportedEntry>,
    /// Network streams (http, https, …), which are not supported.
    pub skipped_streams: usize,
}

#[derive(Debug, Error)]
pub enum PlaylistFileError {
    #[error("the playlist file is larger than {limit} bytes")]
    TooLarge { limit: u64 },
}

/// One entry to write to an M3U8 file.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportEntry {
    pub path: PathBuf,
    pub title: Option<String>,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    M3u,
    M3u8,
    Pls,
}

fn format_of(bytes: &[u8], source: &Path) -> Format {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("pls") => Format::Pls,
        Some("m3u8") => Format::M3u8,
        Some("m3u") => Format::M3u,
        _ => {
            let head: Vec<u8> = strip_bom(bytes)
                .iter()
                .skip_while(|b| b.is_ascii_whitespace())
                .take(10)
                .map(u8::to_ascii_lowercase)
                .collect();
            if head.starts_with(b"[playlist]") {
                Format::Pls
            } else {
                Format::M3u
            }
        }
    }
}

fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes)
}

/// Windows-1252 code points for bytes 0x80–0x9F; other bytes map to Latin-1.
const CP1252_HIGH: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

fn decode_cp1252(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x80..=0x9F => CP1252_HIGH
                .get(usize::from(b - 0x80))
                .copied()
                .unwrap_or('\u{FFFD}'),
            _ => char::from(b),
        })
        .collect()
}

/// UTF-8 when valid (always for M3U8), otherwise Windows-1252.
fn decode(bytes: &[u8], format: Format) -> String {
    let bytes = strip_bom(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) if format == Format::M3u8 => String::from_utf8_lossy(bytes).into_owned(),
        Err(_) => decode_cp1252(bytes),
    }
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        if b == b'%'
            && let (Some(h), Some(l)) = (bytes.get(i + 1), bytes.get(i + 2))
            && let (Some(h), Some(l)) = ((*h as char).to_digit(16), (*l as char).to_digit(16))
        {
            out.push(u8::try_from(h * 16 + l).unwrap_or(b'?'));
            i += 3;
            continue;
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `C:\…`, `C:/…` or `\\server\…`.
fn is_windows_absolute(text: &str) -> bool {
    let b = text.as_bytes();
    let drive = matches!((b.first(), b.get(1), b.get(2)), (Some(l), Some(b':'), Some(b'\\' | b'/')) if l.is_ascii_alphabetic());
    drive || text.starts_with("\\\\")
}

/// Removes `.` and resolves `..` without touching the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// What one playlist line points at.
enum Target {
    File(PathBuf),
    Stream,
}

fn resolve(entry: &str, base: &Path) -> Option<Target> {
    let entry = entry.trim();
    if entry.is_empty() {
        return None;
    }
    if is_windows_absolute(entry) {
        return Some(Target::File(PathBuf::from(entry)));
    }
    let lower = entry.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("file:") {
        // Keep the original case: slice the same length off the original.
        let rest = entry.get(entry.len() - rest.len()..).unwrap_or(rest);
        let rest = match rest.strip_prefix("//") {
            // `file:///path` and `file://localhost/path` are local.
            Some(after) if after.starts_with('/') => after,
            Some(after) => match after.strip_prefix("localhost") {
                Some(local) => local,
                // `file://server/share/…`: a network path.
                None => rest,
            },
            // `file:/path`
            None => rest,
        };
        let decoded = percent_decode(rest);
        // `file:///C:/x` → `C:/x` on Windows.
        let decoded = match decoded.strip_prefix('/') {
            Some(win) if is_windows_absolute(win) => win.to_owned(),
            _ => decoded,
        };
        return Some(Target::File(PathBuf::from(decoded)));
    }
    if let Some(scheme_end) = entry.find("://")
        && entry.get(..scheme_end).is_some_and(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        })
    {
        return Some(Target::Stream);
    }
    if is_windows_absolute(entry) || Path::new(entry).is_absolute() {
        return Some(Target::File(PathBuf::from(entry)));
    }
    let relative = if cfg!(windows) {
        entry.to_owned()
    } else {
        entry.replace('\\', "/")
    };
    Some(Target::File(normalize(&base.join(relative))))
}

/// `#EXTINF:<secs>[ attributes],<title>`
fn parse_extinf(rest: &str) -> (Option<f64>, Option<String>) {
    let (head, title) = rest.split_once(',').unwrap_or((rest, ""));
    let secs = head
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s >= 0.0);
    let title = Some(title.trim().to_owned()).filter(|t| !t.is_empty());
    (secs, title)
}

fn push(
    list: &mut ImportedPlaylist,
    target: Option<Target>,
    title: Option<String>,
    duration_secs: Option<f64>,
) {
    match target {
        Some(Target::File(path)) => list.entries.push(ImportedEntry {
            path,
            title,
            duration_secs,
        }),
        Some(Target::Stream) => list.skipped_streams += 1,
        None => {}
    }
}

fn parse_m3u(text: &str, base: &Path) -> ImportedPlaylist {
    let mut list = ImportedPlaylist::default();
    let mut pending: (Option<f64>, Option<String>) = (None, None);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            pending = parse_extinf(rest);
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let (secs, title) = std::mem::take(&mut pending);
        push(&mut list, resolve(line, base), title, secs);
    }
    list
}

#[derive(Default)]
struct PlsEntry {
    file: Option<String>,
    title: Option<String>,
    length: Option<f64>,
}

fn parse_pls(text: &str, base: &Path) -> ImportedPlaylist {
    let mut entries: BTreeMap<u64, PlsEntry> = BTreeMap::new();
    for line in text.lines() {
        let Some((key, value)) = line.trim().split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        let (field, number) = ["file", "title", "length"]
            .iter()
            .find_map(|f| key.strip_prefix(f).map(|n| (*f, n)))
            .unwrap_or(("", ""));
        let Ok(n) = number.parse::<u64>() else {
            continue;
        };
        let entry = entries.entry(n).or_default();
        match field {
            "file" => entry.file = Some(value.to_owned()),
            "title" => entry.title = Some(value.to_owned()).filter(|t| !t.is_empty()),
            _ => {
                entry.length = value
                    .parse::<f64>()
                    .ok()
                    .filter(|s| s.is_finite() && *s >= 0.0)
            }
        }
    }
    let mut list = ImportedPlaylist::default();
    for entry in entries.into_values() {
        if let Some(file) = entry.file {
            push(&mut list, resolve(&file, base), entry.title, entry.length);
        }
    }
    list
}

/// Reads an M3U, M3U8 or PLS file. `source` is where the bytes came from
/// (for the format and for relative paths).
pub fn parse_playlist(
    bytes: &[u8],
    source: &Path,
    limits: &Limits,
) -> Result<ImportedPlaylist, PlaylistFileError> {
    if bytes.len() as u64 > limits.max_playlist_file_bytes {
        return Err(PlaylistFileError::TooLarge {
            limit: limits.max_playlist_file_bytes,
        });
    }
    let base = source.parent().unwrap_or(Path::new(""));
    let format = format_of(bytes, source);
    let text = decode(bytes, format);
    Ok(match format {
        Format::Pls => parse_pls(&text, base),
        Format::M3u | Format::M3u8 => parse_m3u(&text, base),
    })
}

/// An M3U8 file with `#EXTINF` lines and absolute paths.
pub fn write_m3u8(entries: &[ExportEntry]) -> String {
    let mut out = String::from("#EXTM3U\n");
    for e in entries {
        let secs = e
            .duration_secs
            .filter(|s| s.is_finite() && *s >= 0.0)
            .map_or(-1, |s| s.round() as i64);
        let title = e.title.as_deref().unwrap_or("").replace(['\n', '\r'], " ");
        out.push_str(&format!("#EXTINF:{secs},{title}\n"));
        out.push_str(&e.path.to_string_lossy().replace(['\n', '\r'], " "));
        out.push('\n');
    }
    out
}

/// The format identifier of cart page files.
pub const CART_PAGE_FORMAT: &str = "fauste-cart-page";
pub const CART_PAGE_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum CartPageFileError {
    #[error("the cart page file is larger than {limit} bytes")]
    TooLarge { limit: u64 },
    #[error("not a cart page file")]
    NotACartPage,
    #[error("the cart page file was written by a newer version (format version {0})")]
    TooNew(u32),
    #[error("invalid cart page file: {0}")]
    Invalid(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct CartPageFile {
    format: String,
    version: u32,
    #[serde(default)]
    name: String,
    #[serde(default = "default_rows")]
    rows: u32,
    #[serde(default = "default_cols")]
    cols: u32,
    #[serde(default)]
    carts: Vec<CartFile>,
}

fn default_rows() -> u32 {
    2
}

fn default_cols() -> u32 {
    8
}

#[derive(Debug, Serialize, Deserialize)]
struct CartFile {
    /// 1-based, row-major.
    position: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(default, rename = "loop")]
    looped: bool,
    #[serde(default)]
    exclusive: bool,
}

fn kind_name(kind: CartKind) -> &'static str {
    match kind {
        CartKind::Jingle => "jingle",
        CartKind::Effect => "effect",
        CartKind::Spot => "spot",
    }
}

fn kind_from(name: &str) -> CartKind {
    match name.to_ascii_lowercase().as_str() {
        "effect" => CartKind::Effect,
        "spot" => CartKind::Spot,
        _ => CartKind::Jingle,
    }
}

/// Reads a cart page file. The grid is clamped to the limits; relative
/// files resolve against the file's folder; carts outside the grid are
/// kept in the result and dropped by the model.
pub fn parse_cart_page(
    bytes: &[u8],
    source: &Path,
    limits: &Limits,
) -> Result<CartPageImport, CartPageFileError> {
    if bytes.len() as u64 > limits.max_playlist_file_bytes {
        return Err(CartPageFileError::TooLarge {
            limit: limits.max_playlist_file_bytes,
        });
    }
    let value: serde_json::Value = serde_json::from_slice(strip_bom(bytes))
        .map_err(|e| CartPageFileError::Invalid(e.to_string()))?;
    if value.get("format").and_then(|f| f.as_str()) != Some(CART_PAGE_FORMAT) {
        return Err(CartPageFileError::NotACartPage);
    }
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    if version > u64::from(CART_PAGE_VERSION) {
        return Err(CartPageFileError::TooNew(
            u32::try_from(version).unwrap_or(u32::MAX),
        ));
    }
    let file: CartPageFile =
        serde_json::from_value(value).map_err(|e| CartPageFileError::Invalid(e.to_string()))?;
    let base = source.parent().unwrap_or(Path::new(""));
    let clamp = |v: u32, max: u16| u16::try_from(v).unwrap_or(u16::MAX).clamp(1, max.max(1));
    let carts = file
        .carts
        .into_iter()
        .filter(|c| c.position >= 1)
        .map(|c| {
            let path = c.file.and_then(|f| match resolve(&f, base) {
                Some(Target::File(p)) => Some(p),
                _ => None,
            });
            let position = usize::try_from(c.position - 1).unwrap_or(usize::MAX);
            let edit = CartEdit {
                name: c.name,
                kind: kind_from(&c.kind),
                looped: c.looped,
                exclusive: c.exclusive,
            };
            (position, edit, path)
        })
        .collect();
    Ok(CartPageImport {
        name: file.name,
        rows: clamp(file.rows, limits.max_cart_rows),
        cols: clamp(file.cols, limits.max_cart_cols),
        carts,
    })
}

/// Writes a cart page file; files are absolute paths. Empty carts without a
/// name are left out.
pub fn write_cart_page(page: &CartPage, library: &Library) -> String {
    let carts = page
        .carts
        .iter()
        .enumerate()
        .filter(|(_, c)| c.track.is_some() || !c.name.is_empty() || c.looped || c.exclusive)
        .map(|(i, c)| CartFile {
            position: i as u64 + 1,
            name: c.name.clone(),
            kind: kind_name(c.kind).to_owned(),
            file: c
                .track
                .and_then(|t| library.get(t))
                .map(|t| t.path.to_string_lossy().into_owned()),
            looped: c.looped,
            exclusive: c.exclusive,
        })
        .collect();
    let file = CartPageFile {
        format: CART_PAGE_FORMAT.to_owned(),
        version: CART_PAGE_VERSION,
        name: page.name.clone(),
        rows: u32::from(page.rows),
        cols: u32::from(page.cols),
        carts,
    };
    serde_json::to_string_pretty(&file).unwrap_or_default()
}
