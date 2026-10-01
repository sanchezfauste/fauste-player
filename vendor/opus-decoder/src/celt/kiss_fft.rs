#![allow(dead_code)]

//! Minimal CELT FFT primitives for decoder path.
//!
//! This module intentionally implements only the inverse transform behavior
//! needed by the decoder. It follows the libopus scaling convention where the
//! inverse FFT is unscaled and the forward path applies `1/N`.

use core::f32::consts::PI;

/// Complex number used by CELT FFT/MDCT primitives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Complex32 {
    /// Real component.
    pub re: f32,
    /// Imaginary component.
    pub im: f32,
}

impl Complex32 {
    /// Create a complex value from real and imaginary parts.
    ///
    /// Params: `re` is the real component, `im` is the imaginary component.
    /// Returns: a new `Complex32`.
    pub const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
}

/// FFT plan for CELT decoder usage: a mixed-radix decimation-in-time FFT
/// (the KISS FFT algorithm) in safe Rust.
///
/// Fauste Player's vendored copy: upstream 0.1.1 computed every transform
/// as a direct O(n²) DFT, about 5.5 ms per 20 ms packet.
#[derive(Debug, Clone)]
pub(crate) struct KissFft {
    nfft: usize,
    /// `exp(+2πik/n)`; the forward transform uses their conjugates.
    inv_twiddles: Vec<Complex32>,
    /// Radix and remaining length of each stage, outermost first.
    factors: Vec<(usize, usize)>,
}

/// Largest radix butterflied without a heap scratch buffer.
const STACK_RADIX: usize = 16;

/// Splits `n` into radices: 4s first, then 2, then odd numbers (3, 5, …),
/// with the remainder taken whole once no factor up to its square root is
/// left, as KISS FFT does.
fn factorize(n: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut n = n;
    let mut p = 4usize;
    while n > 1 {
        while n % p != 0 {
            p = match p {
                4 => 2,
                2 => 3,
                _ => p + 2,
            };
            if p * p > n {
                p = n;
            }
        }
        n /= p;
        out.push((p, n));
    }
    out
}

impl KissFft {
    /// Build an FFT plan with runtime twiddle generation.
    ///
    /// Params: `nfft` is the FFT size.
    /// Returns: a plan containing precomputed twiddles and radices.
    pub fn new(nfft: usize) -> Self {
        let mut inv_twiddles = Vec::with_capacity(nfft);
        for k in 0..nfft {
            let phase = 2.0 * core::f64::consts::PI * (k as f64) / (nfft as f64);
            inv_twiddles.push(Complex32::new(phase.cos() as f32, phase.sin() as f32));
        }
        Self {
            nfft,
            inv_twiddles,
            factors: factorize(nfft),
        }
    }

    /// Return FFT size configured in this plan.
    ///
    /// Params: none.
    /// Returns: number of points in the transform.
    pub fn len(&self) -> usize {
        self.nfft
    }

    /// Compute unscaled inverse FFT.
    ///
    /// Params: `input` is frequency-domain complex spectrum, `output` is
    /// destination time-domain buffer and must have length `self.len()`.
    /// Returns: `Ok(())` on success, otherwise a static validation error.
    pub fn ifft(&self, input: &[Complex32], output: &mut [Complex32]) -> Result<(), &'static str> {
        self.transform(input, output, true)
    }

    /// Compute unscaled forward FFT.
    ///
    /// Params: `input` is time-domain complex vector, `output` is destination
    /// frequency-domain buffer and must have length `self.len()`.
    /// Returns: `Ok(())` on success, otherwise a static validation error.
    pub fn fft(&self, input: &[Complex32], output: &mut [Complex32]) -> Result<(), &'static str> {
        self.transform(input, output, false)
    }

    /// Unscaled forward FFT on flat interleaved buffers `[re0, im0, …]` of
    /// `self.len()` complex samples.
    pub fn fft_flat(&self, input: &[f32], output: &mut [f32]) -> Result<(), &'static str> {
        let n = self.nfft;
        if input.len() < 2 * n || output.len() < 2 * n {
            return Err("kiss_fft length mismatch");
        }
        let x: Vec<Complex32> = input
            .chunks_exact(2)
            .take(n)
            .map(|c| Complex32::new(c[0], c[1]))
            .collect();
        let mut y = vec![Complex32::new(0.0, 0.0); n];
        self.transform(&x, &mut y, false)?;
        for (o, c) in output.chunks_exact_mut(2).zip(&y) {
            o[0] = c.re;
            o[1] = c.im;
        }
        Ok(())
    }

    fn transform(
        &self,
        input: &[Complex32],
        output: &mut [Complex32],
        inverse: bool,
    ) -> Result<(), &'static str> {
        if input.len() != self.nfft || output.len() != self.nfft {
            return Err("kiss_fft length mismatch");
        }
        if self.nfft <= 1 {
            output.copy_from_slice(input);
            return Ok(());
        }
        self.work(output, input, 0, 1, &self.factors, inverse);
        Ok(())
    }

    /// One decimation stage: the `p` sub-transforms of length `m`, then the
    /// radix-`p` butterflies that combine them.
    fn work(
        &self,
        out: &mut [Complex32],
        input: &[Complex32],
        offset: usize,
        stride: usize,
        factors: &[(usize, usize)],
        inverse: bool,
    ) {
        let Some((&(p, m), rest)) = factors.split_first() else {
            return;
        };
        if m == 1 {
            for (q, o) in out.iter_mut().take(p).enumerate() {
                *o = input[offset + q * stride];
            }
        } else {
            for (q, chunk) in out.chunks_exact_mut(m).take(p).enumerate() {
                self.work(chunk, input, offset + q * stride, stride * p, rest, inverse);
            }
        }
        if p <= STACK_RADIX {
            let mut scratch = [Complex32::new(0.0, 0.0); STACK_RADIX];
            self.butterfly(out, stride, p, m, inverse, &mut scratch[..p]);
        } else {
            let mut scratch = vec![Complex32::new(0.0, 0.0); p];
            self.butterfly(out, stride, p, m, inverse, &mut scratch);
        }
    }

    /// The generic radix-`p` butterfly of KISS FFT: the twiddle index
    /// `stride·k·q (mod n)` folds the stage twiddle into the DFT kernel.
    fn butterfly(
        &self,
        out: &mut [Complex32],
        stride: usize,
        p: usize,
        m: usize,
        inverse: bool,
        scratch: &mut [Complex32],
    ) {
        let n = self.nfft;
        for u in 0..m {
            for (q, s) in scratch.iter_mut().enumerate() {
                *s = out[u + q * m];
            }
            for q1 in 0..p {
                let k = u + q1 * m;
                let mut acc = scratch[0];
                let mut tw_index = 0usize;
                for s in scratch.iter().skip(1) {
                    tw_index += stride * k;
                    while tw_index >= n {
                        tw_index -= n;
                    }
                    let tw = self.inv_twiddles[tw_index];
                    let tw_im = if inverse { tw.im } else { -tw.im };
                    acc.re += s.re * tw.re - s.im * tw_im;
                    acc.im += s.re * tw_im + s.im * tw.re;
                }
                out[k] = acc;
            }
        }
    }
}

/// Compute forward DFT on flat interleaved complex buffers.
///
/// Params: `input`/`output` are `[re0, im0, re1, im1, ...]` and `n` is the
/// number of complex samples.
/// Returns: nothing; writes the forward transform to `output`. The direct
/// O(n²) definition, kept as the reference the FFT is tested against; the
/// decoder uses [`KissFft::fft_flat`].
pub(crate) fn flat_fft_forward(input: &[f32], output: &mut [f32], n: usize) {
    assert!(input.len() >= 2 * n && output.len() >= 2 * n);
    for k in 0..n {
        let mut sum_re = 0.0f64;
        let mut sum_im = 0.0f64;
        for j in 0..n {
            let angle = -2.0 * PI as f64 * (k as f64) * (j as f64) / (n as f64);
            let (sin_a, cos_a) = angle.sin_cos();
            let re = input[2 * j] as f64;
            let im = input[2 * j + 1] as f64;
            sum_re += re * cos_a - im * sin_a;
            sum_im += re * sin_a + im * cos_a;
        }
        output[2 * k] = sum_re as f32;
        output[2 * k + 1] = sum_im as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::{Complex32, KissFft};
    use core::f32::consts::PI;

    /// Compute libopus-style forward DFT with `1/N` scaling for tests.
    ///
    /// Params: `input` is time-domain complex vector.
    /// Returns: scaled frequency-domain vector.
    fn forward_scaled(input: &[Complex32]) -> Vec<Complex32> {
        let n = input.len();
        let mut out = vec![Complex32::new(0.0, 0.0); n];
        for (k, yk) in out.iter_mut().enumerate() {
            let mut acc_re = 0.0f32;
            let mut acc_im = 0.0f32;
            for (n_idx, xn) in input.iter().enumerate() {
                let phase = -2.0 * PI * (k as f32) * (n_idx as f32) / (n as f32);
                let c = phase.cos();
                let s = phase.sin();
                acc_re += xn.re * c - xn.im * s;
                acc_im += xn.re * s + xn.im * c;
            }
            *yk = Complex32::new(acc_re / (n as f32), acc_im / (n as f32));
        }
        out
    }

    #[test]
    fn the_fft_matches_the_direct_dft_at_every_celt_size() {
        for n in [1usize, 7, 15, 60, 120, 240, 480] {
            let fft = KissFft::new(n);
            let flat: Vec<f32> = (0..2 * n)
                .map(|i| ((i * 7919 % 1000) as f32 / 500.0) - 1.0)
                .collect();
            let mut direct = vec![0.0f32; 2 * n];
            super::flat_fft_forward(&flat, &mut direct, n);
            let mut fast = vec![0.0f32; 2 * n];
            fft.fft_flat(&flat, &mut fast).expect("sizes match");
            // Outputs reach n (inputs within ±1): f32 rounding grows with n.
            for (a, b) in direct.iter().zip(&fast) {
                assert!((a - b).abs() < 2e-5 * n as f32, "n={n}: {a} vs {b}");
            }
            // The inverse undoes the forward transform (scaled by n).
            let x: Vec<Complex32> = flat.chunks_exact(2).map(|c| Complex32::new(c[0], c[1])).collect();
            let mut y = vec![Complex32::new(0.0, 0.0); n];
            fft.fft(&x, &mut y).expect("sizes match");
            let mut z = vec![Complex32::new(0.0, 0.0); n];
            fft.ifft(&y, &mut z).expect("sizes match");
            for (a, b) in x.iter().zip(&z) {
                assert!((a.re - b.re / n as f32).abs() < 1e-5, "n={n}");
                assert!((a.im - b.im / n as f32).abs() < 1e-5, "n={n}");
            }
        }
    }

    #[test]
    fn ifft_roundtrip_matches_input() {
        let n = 60usize;
        let fft = KissFft::new(n);
        let mut input = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f32 / n as f32;
            input.push(Complex32::new(
                (2.0 * PI * 3.0 * t).sin(),
                (2.0 * PI * 5.0 * t).cos(),
            ));
        }
        let freq = forward_scaled(&input);
        let mut recon = vec![Complex32::new(0.0, 0.0); n];
        fft.ifft(&freq, &mut recon).expect("ifft must succeed");
        for (a, b) in input.iter().zip(recon.iter()) {
            assert!(
                (a.re - b.re).abs() < 2e-5,
                "re mismatch: {} vs {}",
                a.re,
                b.re
            );
            assert!(
                (a.im - b.im).abs() < 2e-5,
                "im mismatch: {} vs {}",
                a.im,
                b.im
            );
        }
    }
}
