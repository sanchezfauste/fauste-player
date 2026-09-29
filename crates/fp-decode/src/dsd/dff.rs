//! DSDIFF: an `FRM8` form of type `DSD `, big-endian, with a `PROP` chunk
//! (rate, channels, compression) and a `DSD ` chunk of bytes interleaved by
//! channel, most significant bit first. DST-compressed audio is refused.

use std::fs::File;

use super::{Layout, be_u16, be_u32, be_u64, read_at};

/// Largest `PROP` chunk read (it holds a few small sub-chunks).
const MAX_PROP: u64 = 64 * 1024;

fn chunk_header(file: &mut File, at: u64) -> Result<([u8; 4], u64), String> {
    let mut h = [0u8; 12];
    read_at(file, at, &mut h).map_err(|_| "DSDIFF is truncated".to_string())?;
    let mut id = [0u8; 4];
    id.copy_from_slice(h.get(0..4).unwrap_or(&[0; 4]));
    let size = be_u64(&h, 4).ok_or("DSDIFF is truncated")?;
    Ok((id, size))
}

pub(super) fn layout(file: &mut File) -> Result<Layout, String> {
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    let bad = || "DSDIFF header is malformed".to_string();
    let (id, form_size) = chunk_header(file, 0)?;
    let mut form_type = [0u8; 4];
    read_at(file, 12, &mut form_type).map_err(|_| bad())?;
    if &id != b"FRM8" || &form_type != b"DSD " {
        return Err(bad());
    }
    let form_end = form_size.saturating_add(12).min(file_len);
    let (mut rate, mut channels, mut positions) = (None, None, None);
    let mut at = 16u64;
    while at.saturating_add(12) <= form_end {
        let (id, size) = chunk_header(file, at)?;
        let body = at + 12;
        match &id {
            b"PROP" => {
                if size > MAX_PROP || body.saturating_add(size) > file_len {
                    return Err(bad());
                }
                let mut prop = vec![0u8; usize::try_from(size).map_err(|_| bad())?];
                read_at(file, body, &mut prop)?;
                if prop.get(0..4) != Some(b"SND ") {
                    return Err(bad());
                }
                let mut p = 4usize;
                while let (Some(sub), Some(len)) = (prop.get(p..p + 4), be_u64(&prop, p + 4)) {
                    let data = p + 12;
                    match sub {
                        b"FS  " => rate = be_u32(&prop, data),
                        b"CHNL" => {
                            channels = be_u16(&prop, data);
                            let ids = prop.get(data + 2..).unwrap_or_default();
                            positions = channels.map(|n| {
                                ids.as_chunks::<4>()
                                    .0
                                    .iter()
                                    .take(usize::from(n))
                                    .map(position)
                                    .collect::<Vec<_>>()
                            });
                        }
                        b"CMPR" if prop.get(data..data + 4) != Some(b"DSD ") => {
                            return Err("DST-compressed DSDIFF is not supported".into());
                        }
                        _ => {}
                    }
                    // A sub-chunk must end inside PROP, which keeps `p` small
                    // enough that the offsets above cannot overflow.
                    let end = usize::try_from(len)
                        .ok()
                        .and_then(|len| data.checked_add(len)?.checked_add(len % 2))
                        .filter(|end| *end <= prop.len() + 1)
                        .ok_or_else(bad)?;
                    p = end;
                }
            }
            b"DST " => return Err("DST-compressed DSDIFF is not supported".into()),
            b"DSD " => {
                let rate = rate.ok_or_else(bad)?;
                let channels = usize::from(channels.ok_or_else(bad)?);
                let data_len = size.min(file_len.saturating_sub(body));
                let bytes = data_len / u64::try_from(channels.max(1)).map_err(|_| bad())?;
                return Ok(Layout {
                    channels,
                    rate,
                    samples: bytes.saturating_mul(8),
                    bytes,
                    data_start: body,
                    block: None,
                    lsb_first: false,
                    positions: positions
                        .filter(|p: &Vec<Option<usize>>| p.len() == channels)
                        .unwrap_or_else(|| super::default_positions(channels)),
                });
            }
            _ => {}
        }
        at = body.saturating_add(size).saturating_add(size % 2);
    }
    Err("DSDIFF has no DSD audio".into())
}

/// The frame position of a DSDIFF channel ID.
fn position(id: &[u8; 4]) -> Option<usize> {
    use super::{C, L, LFE, LS, R, RS};
    match id {
        b"SLFT" | b"MLFT" => Some(L),
        b"SRGT" | b"MRGT" => Some(R),
        b"C   " => Some(C),
        b"LFE " => Some(LFE),
        b"LS  " => Some(LS),
        b"RS  " => Some(RS),
        _ => None,
    }
}
