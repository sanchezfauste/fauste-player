//! DSF: a `DSD ` chunk, a `fmt ` chunk and a `data` chunk, little-endian.
//! Audio is stored in blocks of (usually) 4096 bytes per channel, one
//! channel after the other, least significant bit first.

use std::fs::File;

use super::{Layout, le_u32, le_u64, read_at};

/// `DSD ` (28 bytes) + `fmt ` (52) + the `data` chunk header (12).
const HEADER: usize = 92;
/// Largest block per channel accepted (the format uses 4096).
const MAX_BLOCK: u64 = 1 << 20;

pub(super) fn layout(file: &mut File) -> Result<Layout, String> {
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut h = [0u8; HEADER];
    read_at(file, 0, &mut h).map_err(|_| "DSF header is truncated".to_string())?;
    let bad = || "DSF header is malformed".to_string();
    if h.get(0..4) != Some(b"DSD ") || h.get(28..32) != Some(b"fmt ") {
        return Err(bad());
    }
    let format_id = le_u32(&h, 44).ok_or_else(bad)?;
    if format_id != 0 {
        return Err(format!("DSF format {format_id} is not raw DSD"));
    }
    let channel_type = le_u32(&h, 48).ok_or_else(bad)?;
    let channels = le_u32(&h, 52).ok_or_else(bad)?;
    let rate = le_u32(&h, 56).ok_or_else(bad)?;
    let bits = le_u32(&h, 60).ok_or_else(bad)?;
    let samples = le_u64(&h, 64).ok_or_else(bad)?;
    let block = u64::from(le_u32(&h, 72).ok_or_else(bad)?);
    let fmt_size = le_u64(&h, 32).ok_or_else(bad)?;
    let data_at = 28u64.checked_add(fmt_size).ok_or_else(bad)?;
    if data_at != 80 || h.get(80..84) != Some(b"data") {
        return Err(bad());
    }
    let data_size = le_u64(&h, 84).ok_or_else(bad)?;
    if block == 0 || block > MAX_BLOCK || !(bits == 1 || bits == 8) || channels == 0 {
        return Err(bad());
    }
    let channels_u64 = u64::from(channels);
    let data_start = HEADER as u64;
    // Only whole block groups the file really holds.
    let data_len = data_size
        .saturating_sub(12)
        .min(file_len.saturating_sub(data_start));
    let groups = data_len / (block * channels_u64);
    let bytes = (groups * block).min(samples.div_ceil(8));
    Ok(Layout {
        channels: usize::try_from(channels).map_err(|_| bad())?,
        rate,
        samples,
        bytes,
        data_start,
        block: Some(block),
        lsb_first: bits == 1,
        positions: positions(channel_type, usize::try_from(channels).map_err(|_| bad())?),
    })
}

/// Channel positions from the DSF channel type; the usual order for the
/// count if the type does not match it.
fn positions(channel_type: u32, channels: usize) -> Vec<Option<usize>> {
    use super::{C, L, LFE, LS, R, RS};
    let order: &[usize] = match channel_type {
        3 => &[L, R, C],
        4 => &[L, R, LS, RS],
        5 => &[L, R, C, LFE],
        6 => &[L, R, C, LS, RS],
        7 => &[L, R, C, LFE, LS, RS],
        _ => &[],
    };
    if order.len() == channels {
        order.iter().copied().map(Some).collect()
    } else {
        super::default_positions(channels)
    }
}
