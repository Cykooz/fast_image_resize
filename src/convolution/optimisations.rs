use crate::convolution::Coefficients;
// This code is based on C-implementation from Pillow-SIMD package for Python
// https://github.com/uploadcare/pillow-simd

const fn get_clip_table() -> [u8; 1280] {
    let mut table = [0u8; 1280];
    let mut i: usize = 640;
    while i < 640 + 255 {
        table[i] = (i - 640) as u8;
        i += 1;
    }
    while i < 1280 {
        table[i] = 255;
        i += 1;
    }
    table
}

// Handles values form -640 to 639.
static CLIP8_LOOKUPS: [u8; 1280] = get_clip_table();

// 8 bits for a result. Filter can have negative areas.
// In one case, the sum of the coefficients will be negative,
// in the other it will be more than 1.0. That is why we need
// two extra bits for overflow and i32 type.
const PRECISION_BITS: u8 = 32 - 8 - 2;
// We use i16 type to store coefficients.
const MAX_COEFFS_PRECISION: u8 = 16 - 1;

#[derive(Debug, Clone)]
pub(crate) struct CoefficientsI16Chunk {
    pub start: u32,
    values: Vec<i16>,
}

impl CoefficientsI16Chunk {
    #[inline(always)]
    pub fn values(&self) -> &[i16] {
        &self.values
    }
}

pub(crate) struct Normalizer16 {
    precision: u8,
    chunks: Vec<CoefficientsI16Chunk>,
}

impl Normalizer16 {
    #[inline]
    pub fn new(coefficients: Coefficients) -> Self {
        let max_weight = coefficients
            .values
            .iter()
            .max_by(|&x, &y| x.partial_cmp(y).unwrap())
            .unwrap_or(&0.0)
            .to_owned();

        let mut precision = 0u8;
        for cur_precision in 0..PRECISION_BITS {
            precision = cur_precision;
            let next_value: i32 = (max_weight * (1 << (precision + 1)) as f64).round() as i32;
            if next_value >= (1 << MAX_COEFFS_PRECISION) {
                // The next value will be outside the range, so stop
                break;
            }
        }
        debug_assert!(precision >= 4); // required for some SIMD optimisations

        let mut chunks = Vec::with_capacity(coefficients.bounds.len());
        if coefficients.window_size > 0 {
            let scale = (1 << precision) as f64;
            let coef_chunks = coefficients.values.chunks_exact(coefficients.window_size);
            for (chunk, bound) in coef_chunks.zip(&coefficients.bounds) {
                let chunk_i16: Vec<i16> = chunk
                    .iter()
                    .take(bound.size as usize)
                    .map(|&v| (v * scale).round() as i16)
                    .collect();
                chunks.push(CoefficientsI16Chunk {
                    start: bound.start,
                    values: chunk_i16,
                });
            }
        }

        Self { precision, chunks }
    }

    #[inline(always)]
    pub fn precision(&self) -> u8 {
        self.precision
    }

    #[inline(always)]
    pub fn chunks(&self) -> &[CoefficientsI16Chunk] {
        &self.chunks
    }

    pub fn chunks_len(&self) -> usize {
        self.chunks.len()
    }

    /// # Safety
    /// The function must be used with the `v`
    /// such that the expression `v >> self.precision`
    /// produces a result in the range `[-512, 511]`.    
    #[inline(always)]
    pub unsafe fn clip(&self, v: i32) -> u8 {
        let index = (640 + (v >> self.precision)) as usize;
        // index must be in range [(640-512)..(640+511)]
        debug_assert!((128..=1151).contains(&index));
        *CLIP8_LOOKUPS.get_unchecked(index)
    }
}

// 16 bits for a result. Filter can have negative areas.
// In one cases the sum of the coefficients will be negative,
// in the other it will be more than 1.0. That is why we need
// two extra bits for overflow and i64 type.
const PRECISION16_BITS: u8 = 64 - 16 - 2;
// We use i32 type to store coefficients.
const MAX_COEFFS_PRECISION16: u8 = 32 - 1;

#[derive(Debug, Clone)]
pub(crate) struct CoefficientsI32Chunk {
    pub start: u32,
    values: Vec<i32>,
}

impl CoefficientsI32Chunk {
    #[inline(always)]
    pub fn values(&self) -> &[i32] {
        &self.values
    }
}

/// Converts `Vec<f64>` into `Vec<i32>`.
pub(crate) struct Normalizer32 {
    precision: u8,
    chunks: Vec<CoefficientsI32Chunk>,
}

impl Normalizer32 {
    #[inline]
    pub fn new(coefficients: Coefficients) -> Self {
        let max_weight = coefficients
            .values
            .iter()
            .max_by(|&x, &y| x.partial_cmp(y).unwrap())
            .unwrap_or(&0.0)
            .to_owned();

        let mut precision = 0u8;
        for cur_precision in 0..PRECISION16_BITS {
            precision = cur_precision;
            let next_value: i64 = (max_weight * (1i64 << (precision + 1)) as f64).round() as i64;
            // The next value will be outside the range, so just stop
            if next_value >= (1i64 << MAX_COEFFS_PRECISION16) {
                break;
            }
        }
        debug_assert!(precision >= 4); // required for some SIMD optimisations

        let mut chunks = Vec::with_capacity(coefficients.bounds.len());
        if coefficients.window_size > 0 {
            let scale = (1i64 << precision) as f64;
            let coef_chunks = coefficients.values.chunks_exact(coefficients.window_size);
            for (chunk, bound) in coef_chunks.zip(&coefficients.bounds) {
                let chunk_i32: Vec<i32> = chunk
                    .iter()
                    .take(bound.size as usize)
                    .map(|&v| (v * scale).round() as i32)
                    .collect();
                chunks.push(CoefficientsI32Chunk {
                    start: bound.start,
                    values: chunk_i32,
                });
            }
        }

        Self { precision, chunks }
    }

    #[inline]
    pub fn precision(&self) -> u8 {
        self.precision
    }

    #[inline(always)]
    pub fn chunks(&self) -> &[CoefficientsI32Chunk] {
        &self.chunks
    }

    #[inline(always)]
    pub fn chunks_len(&self) -> usize {
        self.chunks.len()
    }

    #[inline(always)]
    pub fn clip(&self, v: i64) -> u16 {
        (v >> self.precision).min(u16::MAX as i64).max(0) as u16
    }
}

macro_rules! try_process_in_threads_h {
    {$op: ident($src_view: ident, $dst_view: ident, $offset: ident, $($arg: ident),+$(,)?);}  => {
        #[allow(unused_labels)]
        'block: {
            #[cfg(feature = "rayon")]
            {
                use crate::threading::split_h_two_images_for_threading;
                use rayon::prelude::*;

                if let Some(iter) = split_h_two_images_for_threading($src_view, $dst_view, $offset) {
                    iter.for_each(|(src, mut dst)| {
                        $op(&src, &mut dst, 0, $($arg),+);
                    });
                    break 'block;
                }
            }
            $op($src_view, $dst_view, $offset, $($arg),+);
        }
    };
}

macro_rules! try_process_in_threads_v {
    {$op: ident($src_view: ident, $dst_view: ident, $offset: ident, $($arg: ident),+$(,)?);}  => {
        #[allow(unused_labels)]
        'block: {
            #[cfg(feature = "rayon")]
            {
                use crate::threading::split_v_two_images_for_threading;
                use rayon::prelude::*;

                if let Some(iter) = split_v_two_images_for_threading($src_view, $dst_view, $offset) {
                    iter.for_each(|(src, mut dst)| {
                        $op(&src, &mut dst, 0, $($arg),+);
                    });
                    break 'block;
                }
            }
            $op($src_view, $dst_view, $offset, $($arg),+);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convolution::Bound;
    fn get_coefficients(value: f64) -> Coefficients {
        Coefficients {
            values: vec![value],
            window_size: 1,
            bounds: vec![Bound { start: 0, size: 1 }],
        }
    }

    #[test]
    fn test_minimal_precision() {
        // required for some SIMD optimisations
        assert!(Normalizer16::new(get_coefficients(0.0)).precision() >= 4);
        assert!(Normalizer16::new(get_coefficients(2.0)).precision() >= 4);
        assert!(Normalizer32::new(get_coefficients(0.0)).precision() >= 4);
        assert!(Normalizer32::new(get_coefficients(2.0)).precision() >= 4);
    }
}
