//! The player header's fit: which of the left items have room, so that no
//! two header items ever overlap. Pure, so it is tested without a frame.

/// One item of the header's left group, measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    /// Its width in points.
    pub width: f32,
    /// How long it survives: 0 is never collapsed, a higher rank collapses
    /// before a lower one.
    pub rank: u8,
}

/// Which `items` are shown (same order) in `available` points, with
/// `spacing` between two shown items. The item of the highest rank
/// collapses first (the rightmost of two equal ranks); an item of rank 0 is
/// always kept, even when it alone does not fit.
pub fn fit(items: &[Item], available: f32, spacing: f32) -> Vec<bool> {
    let mut shown = vec![true; items.len()];
    loop {
        let widths: Vec<f32> = items
            .iter()
            .zip(&shown)
            .filter(|(_, s)| **s)
            .map(|(i, _)| i.width)
            .collect();
        let gaps = widths.len().saturating_sub(1) as f32;
        if widths.iter().sum::<f32>() + gaps * spacing <= available {
            return shown;
        }
        // The shown item of the highest rank, the rightmost on a tie.
        let victim = items
            .iter()
            .enumerate()
            .filter(|(i, item)| item.rank > 0 && shown.get(*i).copied().unwrap_or(false))
            .max_by_key(|(i, item)| (item.rank, *i))
            .map(|(i, _)| i);
        match victim.and_then(|i| shown.get_mut(i)) {
            Some(slot) => *slot = false,
            None => return shown,
        }
    }
}
