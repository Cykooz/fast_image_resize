#[inline(always)]
pub(crate) fn mul_div_255(a: u8, b: u8) -> u8 {
    let tmp = a as u32 * b as u32 + 128;
    (((tmp >> 8) + tmp) >> 8) as u8
}

#[inline(always)]
pub(crate) fn mul_div_65535(a: u16, b: u16) -> u16 {
    let tmp = a as u32 * b as u32 + 0x8000;
    (((tmp >> 16) + tmp) >> 16) as u16
}

const fn recip_alpha_array(precision: u32) -> [u32; 256] {
    let mut res = [0; 256];
    let scale = 1 << (precision + 1);
    let scaled_max = 255 * scale;
    let mut i: usize = 1;
    while i < 256 {
        res[i] = (((scaled_max / i as u32) + 1) >> 1) as u32;
        i += 1;
    }
    res
}

const fn recip_alpha16_array(precision: u64) -> [u64; 65536] {
    let mut res = [0; 65536];
    let scale = 1 << (precision + 1);
    let scaled_max = 0xffff * scale;
    let mut i: usize = 1;
    while i < 65536 {
        res[i] = (((scaled_max / i as u64) + 1) >> 1) as u64;
        i += 1;
    }
    res
}

const PRECISION: u32 = 8;
const PRECISION16: u64 = 33;

#[inline(always)]
pub(crate) fn div_and_clip(v: u8, recip_alpha: u32) -> u8 {
    ((v as u32 * recip_alpha) >> PRECISION).min(255) as u8
}

#[inline(always)]
pub(crate) fn div_and_clip16(v: u16, recip_alpha: u64) -> u16 {
    ((v as u64 * recip_alpha) >> PRECISION16).min(65535) as u16
}

pub(crate) const RECIP_ALPHA: [u32; 256] = recip_alpha_array(PRECISION);
pub(crate) static RECIP_ALPHA16: [u64; 65536] = recip_alpha16_array(PRECISION16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recip_alpha_array() {
        for alpha in 0..=255u8 {
            let expected = if alpha == 0 {
                0
            } else {
                let scale = (1 << PRECISION) as f64;
                (255.0 * scale / alpha as f64).round() as u32
            };

            let recip_alpha = RECIP_ALPHA[alpha as usize];
            assert_eq!(expected, recip_alpha, "alpha {}", alpha);
        }
    }

    #[test]
    fn test_div_and_clip() {
        let mut err_sum: i32 = 0;
        for alpha in 0..=255u8 {
            for color in 0..=255u8 {
                let multiplied_color = (color as f64 * alpha as f64 / 255.).round().min(255.) as u8;

                let expected_color = if alpha == 0 {
                    0
                } else {
                    let recip_alpha = 255. / alpha as f64;
                    let res = multiplied_color as f64 * recip_alpha;
                    res.min(255.) as u8
                };

                let recip_alpha = RECIP_ALPHA[alpha as usize];
                let result_color = div_and_clip(multiplied_color, recip_alpha);
                let delta = result_color as i32 - expected_color as i32;
                err_sum += delta.abs();
            }
        }
        assert_eq!(err_sum, 3468);
    }

    #[test]
    fn test_recip_alpha16_array() {
        for alpha in 0..=0xffffu16 {
            let expected = if alpha == 0 {
                0
            } else {
                let scale = (1u64 << PRECISION16) as f64;
                (65535.0 * scale / alpha as f64).round() as u64
            };

            let recip_alpha = RECIP_ALPHA16[alpha as usize];
            assert_eq!(expected, recip_alpha, "alpha {}", alpha);
        }
    }
}
