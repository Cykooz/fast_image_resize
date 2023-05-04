use std::arch::aarch64::*;

use crate::pixels::PixelExt;

#[inline(always)]
pub unsafe fn load_u8x1<T>(buf: &[T], index: usize) -> uint8x8_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u8;
    vcreate_u8(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_u8x2<T>(buf: &[T], index: usize) -> uint8x8_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u16;
    vcreate_u8(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_u8x4<T>(buf: &[T], index: usize) -> uint8x8_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u32;
    vcreate_u8(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_u8x8<T>(buf: &[T], index: usize) -> uint8x8_t {
    vld1_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_u8x8x3<T>(buf: &[T], index: usize) -> uint8x8x3_t {
    vld1_u8_x3(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_deintrel_u8x8x2<T>(buf: &[T], index: usize) -> uint8x8x2_t {
    vld2_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_deintrel_u8x8x4<T>(buf: &[T], index: usize) -> uint8x8x4_t {
    vld4_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_u8x16<T>(buf: &[T], index: usize) -> uint8x16_t {
    vld1q_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_u8x16x2<T>(buf: &[T], index: usize) -> uint8x16x2_t {
    vld1q_u8_x2(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_u8x16x4<T>(buf: &[T], index: usize) -> uint8x16x4_t {
    vld1q_u8_x4(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_deintrel_u8x16x2<T>(buf: &[T], index: usize) -> uint8x16x2_t {
    vld2q_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_deintrel_u8x16x3<T>(buf: &[T], index: usize) -> uint8x16x3_t {
    vld3q_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_deintrel_u8x16x4<T>(buf: &[T], index: usize) -> uint8x16x4_t {
    vld4q_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
}

#[inline(always)]
pub unsafe fn load_u16x1<T>(buf: &[T], index: usize) -> uint16x4_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u16;
    vcreate_u16(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_u16x2<T>(buf: &[T], index: usize) -> uint16x4_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u32;
    vcreate_u16(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_u16x4<T>(buf: &[T], index: usize) -> uint16x4_t {
    vld1_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_u16x8<T>(buf: &[T], index: usize) -> uint16x8_t {
    vld1q_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_u16x8x2<T>(buf: &[T], index: usize) -> uint16x8x2_t {
    vld1q_u16_x2(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_u16x8x4<T>(buf: &[T], index: usize) -> uint16x8x4_t {
    vld1q_u16_x4(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x1x3<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x4x3_t {
    let mut arr = [0u16; 12];
    let src_ptr = buf.get_unchecked(index..).as_ptr() as *const u16;
    let dst_ptr = arr.as_mut_ptr();
    std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, 3);
    vld3_u16(arr.as_ptr())
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x2x3<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x4x3_t {
    let mut arr = [0u16; 12];
    let src_ptr = buf.get_unchecked(index..).as_ptr() as *const u16;
    let dst_ptr = arr.as_mut_ptr();
    std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, 6);
    vld3_u16(arr.as_ptr())
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x4x3<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x4x3_t {
    vld3_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x4x4<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x4x4_t {
    vld4_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x4x2<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x4x2_t {
    vld2_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x8x2<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x8x2_t {
    vld2q_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x8x3<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x8x3_t {
    vld3q_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_deintrel_u16x8x4<T: PixelExt<Component = u16>>(
    buf: &[T],
    index: usize,
) -> uint16x8x4_t {
    vld4q_u16(buf.get_unchecked(index..).as_ptr() as *const u16)
}

#[inline(always)]
pub unsafe fn load_i32x1<T>(buf: &[T], index: usize) -> int32x2_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u32;
    vcreate_s32(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_i32x2<T>(buf: &[T], index: usize) -> int32x2_t {
    vld1_s32(buf.get_unchecked(index..).as_ptr() as *const i32)
}

#[inline(always)]
pub unsafe fn load_i32x4<T>(buf: &[T], index: usize) -> int32x4_t {
    vld1q_s32(buf.get_unchecked(index..).as_ptr() as *const i32)
}

#[inline(always)]
pub unsafe fn store_i32x4<T>(buf: &mut [T], index: usize, v: int32x4_t) {
    vst1q_s32(buf.get_unchecked_mut(index..).as_mut_ptr() as *mut i32, v);
}

#[inline(always)]
pub unsafe fn load_i32x4x2<T>(buf: &[T], index: usize) -> int32x4x2_t {
    vld1q_s32_x2(buf.get_unchecked(index..).as_ptr() as *const i32)
}

#[inline(always)]
pub unsafe fn store_i32x4x2<T>(buf: &mut [T], index: usize, v: int32x4x2_t) {
    vst1q_s32_x2(buf.get_unchecked_mut(index..).as_mut_ptr() as *mut i32, v);
}

#[inline(always)]
pub unsafe fn load_i32x4x4<T>(buf: &[T], index: usize) -> int32x4x4_t {
    vld1q_s32_x4(buf.get_unchecked(index..).as_ptr() as *const i32)
}

#[inline(always)]
pub unsafe fn store_i32x4x4<T>(buf: &mut [T], index: usize, v: int32x4x4_t) {
    vst1q_s32_x4(buf.get_unchecked_mut(index..).as_mut_ptr() as *mut i32, v);
}

#[inline(always)]
pub unsafe fn load_i64x2<T>(buf: &[T], index: usize) -> int64x2_t {
    vld1q_s64(buf.get_unchecked(index..).as_ptr() as *const i64)
}

#[inline(always)]
pub unsafe fn load_i64x2x2<T>(buf: &[T], index: usize) -> int64x2x2_t {
    vld1q_s64_x2(buf.get_unchecked(index..).as_ptr() as *const i64)
}

#[inline(always)]
pub unsafe fn load_i64x2x4<T>(buf: &[T], index: usize) -> int64x2x4_t {
    vld1q_s64_x4(buf.get_unchecked(index..).as_ptr() as *const i64)
}

#[inline(always)]
pub unsafe fn store_i64x2x2<T>(buf: &mut [T], index: usize, v: int64x2x2_t) {
    vst1q_s64_x2(buf.get_unchecked_mut(index..).as_mut_ptr() as *mut i64, v);
}

#[inline(always)]
pub unsafe fn store_i64x2x4<T>(buf: &mut [T], index: usize, v: int64x2x4_t) {
    vst1q_s64_x4(buf.get_unchecked_mut(index..).as_mut_ptr() as *mut i64, v);
}

#[inline(always)]
pub unsafe fn load_i16x1<T>(buf: &[T], index: usize) -> int16x4_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u16;
    vcreate_s16(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_i16x2<T>(buf: &[T], index: usize) -> int16x4_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u32;
    vcreate_s16(ptr.read_unaligned() as u64)
}

#[inline(always)]
pub unsafe fn load_i16x4<T>(buf: &[T], index: usize) -> int16x4_t {
    vld1_s16(buf.get_unchecked(index..).as_ptr() as *const i16)
}

#[inline(always)]
pub unsafe fn load_i16x8<T>(buf: &[T], index: usize) -> int16x8_t {
    vld1q_s16(buf.get_unchecked(index..).as_ptr() as *const i16)
}

#[inline(always)]
pub unsafe fn load_i16x8x2<T>(buf: &[T], index: usize) -> int16x8x2_t {
    vld1q_s16_x2(buf.get_unchecked(index..).as_ptr() as *const i16)
}

/// Moves 32-bit integer from `buf` to the least significant 32 bits of an uint8x16_t object,
/// zero extending the upper bits.
/// ```plain
///   r0 := a
///   r1 := 0x0
///   r2 := 0x0
///   r3 := 0x0
/// ```
#[inline(always)]
pub unsafe fn create_u8x16_from_one_u32<T>(buf: &[T], index: usize) -> uint8x16_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u32;
    vreinterpretq_u8_u32(vsetq_lane_u32::<0>(ptr.read_unaligned(), vdupq_n_u32(0u32)))
}

/// Multiply the packed unsigned 16-bit integers in a and b, producing
/// intermediate 32-bit integers, and store the high 16 bits of the intermediate
/// integers in dst.
#[inline(always)]
pub unsafe fn mulhi_u16x8(a: uint16x8_t, b: uint16x8_t) -> uint16x8_t {
    let a3210 = vget_low_u16(a);
    let b3210 = vget_low_u16(b);
    let ab3210 = vmull_u16(a3210, b3210);
    let ab7654 = vmull_high_u16(a, b);
    vuzp2q_u16(vreinterpretq_u16_u32(ab3210), vreinterpretq_u16_u32(ab7654))
}

/// Multiply the packed unsigned 32-bit integers in a and b, producing
/// intermediate 64-bit integers, and store the high 32 bits of the intermediate
/// integers in dst.
#[inline(always)]
pub unsafe fn mulhi_u32x4(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    let a3210 = vget_low_u32(a);
    let b3210 = vget_low_u32(b);
    let ab3210 = vmull_u32(a3210, b3210);
    let ab7654 = vmull_high_u32(a, b);
    vuzp2q_u32(vreinterpretq_u32_u64(ab3210), vreinterpretq_u32_u64(ab7654))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn mul_color_to_alpha_u8x16(
    color: uint8x16_t,
    alpha_u16: uint16x8x2_t,
    zero: uint8x16_t,
) -> uint8x16_t {
    let color_u16_lo = vreinterpretq_u16_u8(vzip1q_u8(color, zero));
    let mut tmp_res = vmulq_u16(color_u16_lo, alpha_u16.0);
    tmp_res = vaddq_u16(tmp_res, vrshrq_n_u16::<8>(tmp_res));
    let res_u16_lo = vrshrq_n_u16::<8>(tmp_res);

    let color_u16_hi = vreinterpretq_u16_u8(vzip2q_u8(color, zero));
    let mut tmp_res = vmulq_u16(color_u16_hi, alpha_u16.1);
    tmp_res = vaddq_u16(tmp_res, vrshrq_n_u16::<8>(tmp_res));
    let res_u16_hi = vrshrq_n_u16::<8>(tmp_res);

    vcombine_u8(vqmovn_u16(res_u16_lo), vqmovn_u16(res_u16_hi))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn mul_color_to_alpha_u8x8(
    color: uint8x8_t,
    alpha_u16: uint16x8_t,
    zero: uint8x8_t,
) -> uint8x8_t {
    let color_u16_lo = vreinterpret_u16_u8(vzip1_u8(color, zero));
    let color_u16_hi = vreinterpret_u16_u8(vzip2_u8(color, zero));
    let color_u16 = vcombine_u16(color_u16_lo, color_u16_hi);
    let mut tmp_res = vmulq_u16(color_u16, alpha_u16);
    tmp_res = vaddq_u16(tmp_res, vrshrq_n_u16::<8>(tmp_res));
    let res_u16 = vrshrq_n_u16::<8>(tmp_res);
    vqmovn_u16(res_u16)
}

#[inline(always)]
pub unsafe fn multiply_color_to_alpha_u16x8(color: uint16x8_t, alpha: uint16x8_t) -> uint16x8_t {
    let rounder = vdupq_n_u32(0x8000);
    let color_lo_u32 = vmlal_u16(rounder, vget_low_u16(color), vget_low_u16(alpha));
    let color_hi_u32 = vmlal_high_u16(rounder, color, alpha);
    let color_lo_u16 = vaddhn_u32(color_lo_u32, vshrq_n_u32::<16>(color_lo_u32));
    let color_hi_u16 = vaddhn_u32(color_hi_u32, vshrq_n_u32::<16>(color_hi_u32));
    vcombine_u16(color_lo_u16, color_hi_u16)
}

#[inline(always)]
pub unsafe fn multiply_color_to_alpha_u16x4(color: uint16x4_t, alpha: uint16x4_t) -> uint16x4_t {
    let rounder = vdupq_n_u32(0x8000);
    let color_u32 = vmlal_u16(rounder, color, alpha);
    vaddhn_u32(color_u32, vshrq_n_u32::<16>(color_u32))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn mul_color_recip_alpha_u8x16(
    color: uint8x16_t,
    recip_alpha: uint16x8x2_t,
    zero: uint8x16_t,
) -> uint8x16_t {
    let color_u16_lo = vreinterpretq_u16_u8(vzip1q_u8(zero, color));
    let color_u16_hi = vreinterpretq_u16_u8(vzip2q_u8(zero, color));

    let res_u16_lo = mulhi_u16x8(color_u16_lo, recip_alpha.0);
    let res_u16_hi = mulhi_u16x8(color_u16_hi, recip_alpha.1);
    vcombine_u8(vmovn_u16(res_u16_lo), vmovn_u16(res_u16_hi))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn mul_color_recip_alpha_u8x8(
    color: uint8x8_t,
    recip_alpha: uint16x8_t,
    zero: uint8x8_t,
) -> uint8x8_t {
    let color_u16_lo = vreinterpret_u16_u8(vzip1_u8(zero, color));
    let color_u16_hi = vreinterpret_u16_u8(vzip2_u8(zero, color));
    let color_u16 = vcombine_u16(color_u16_lo, color_u16_hi);
    let res_u16 = mulhi_u16x8(color_u16, recip_alpha);
    vmovn_u16(res_u16)
}
