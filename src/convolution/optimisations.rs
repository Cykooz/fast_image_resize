use super::Bound;
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

// 8 bits for result. Filter can have negative areas.
// In one cases the sum of the coefficients will be negative,
// in the other it will be more than 1.0. That is why we need
// two extra bits for overflow and i32 type.
const PRECISION_BITS: u8 = 32 - 8 - 2;
// We use i16 type to store coefficients.
const MAX_COEFFS_PRECISION: u8 = 16 - 1;

/// Converts `Vec<f64>` into `Vec<i16>`.
pub(crate) struct Normalizer16 {
    values: Vec<i16>,
    precision: u8,
    window_size: usize,
    bounds: Vec<Bound>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CoefficientsI16Chunk<'a> {
    pub start: u32,
    pub values: &'a [i16],
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
            // The next value will be outside the range, so just stop
            if next_value >= (1 << MAX_COEFFS_PRECISION) {
                break;
            }
        }
        debug_assert!(precision >= 4); // required for some SIMD optimisations

        let mut values_i16 = Vec::with_capacity(coefficients.values.len());

        let scale = (1 << precision) as f64;
        for src in coefficients.values.iter().copied() {
            values_i16.push((src * scale).round() as i16);
        }
        Self {
            values: values_i16,
            precision,
            window_size: coefficients.window_size,
            bounds: coefficients.bounds,
        }
    }

    #[inline]
    pub fn normalized_chunks(&self) -> Vec<CoefficientsI16Chunk> {
        let mut cooefs = self.values.as_slice();
        let mut res = Vec::with_capacity(self.bounds.len());
        for bound in self.bounds.iter() {
            let (left, right) = cooefs.split_at(self.window_size);
            cooefs = right;
            let size = bound.size as usize;
            res.push(CoefficientsI16Chunk {
                start: bound.start,
                values: &left[0..size],
            });
        }
        res
    }

    #[inline]
    pub fn precision(&self) -> u8 {
        self.precision
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

// 16 bits for result. Filter can have negative areas.
// In one cases the sum of the coefficients will be negative,
// in the other it will be more than 1.0. That is why we need
// two extra bits for overflow and i64 type.
const PRECISION16_BITS: u8 = 64 - 16 - 2;
// We use i32 type to store coefficients.
const MAX_COEFFS_PRECISION16: u8 = 32 - 1;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CoefficientsI32Chunk<'a> {
    pub start: u32,
    pub values: &'a [i32],
}

/// Converts `Vec<f64>` into `Vec<i32>`.
pub(crate) struct Normalizer32 {
    values: Vec<i32>,
    precision: u8,
    window_size: usize,
    bounds: Vec<Bound>,
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

        let mut values_i32 = Vec::with_capacity(coefficients.values.len());

        let scale = (1i64 << precision) as f64;
        for src in coefficients.values.iter().copied() {
            values_i32.push((src * scale).round() as i32);
        }
        Self {
            values: values_i32,
            precision,
            window_size: coefficients.window_size,
            bounds: coefficients.bounds,
        }
    }

    #[inline]
    pub fn normalized_chunks(&self) -> Vec<CoefficientsI32Chunk> {
        let mut cooefs = self.values.as_slice();
        let mut res = Vec::with_capacity(self.bounds.len());
        for bound in self.bounds.iter() {
            let (left, right) = cooefs.split_at(self.window_size);
            cooefs = right;
            let size = bound.size as usize;
            res.push(CoefficientsI32Chunk {
                start: bound.start,
                values: &left[0..size],
            });
        }
        res
    }

    #[inline]
    pub fn precision(&self) -> u8 {
        self.precision
    }

    #[inline(always)]
    pub fn clip(&self, v: i64) -> u16 {
        (v >> self.precision).min(u16::MAX as i64).max(0) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_coefficients(value: f64) -> Coefficients {
        Coefficients {
            values: vec![value],
            window_size: 0,
            bounds: vec![],
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
