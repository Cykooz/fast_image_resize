use std::arch::aarch64::*;

#[inline(always)]
pub unsafe fn load_u8x4<T>(buf: &[T], index: usize) -> uint8x8_t {
    let ptr = buf.get_unchecked(index..).as_ptr() as *const u32;
    vcreate_u8(*ptr as u64)
}

#[inline(always)]
pub unsafe fn load_u8x8<T>(buf: &[T], index: usize) -> uint8x8_t {
    vld1_u8(buf.get_unchecked(index..).as_ptr() as *const u8)
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
pub unsafe fn load_i16x4<T>(buf: &[T], index: usize) -> int16x4_t {
    vld1_s16(buf.get_unchecked(index..).as_ptr() as *const i16)
}

#[inline(always)]
pub unsafe fn load_i16x8<T>(buf: &[T], index: usize) -> int16x8_t {
    vld1q_s16(buf.get_unchecked(index..).as_ptr() as *const i16)
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
    vreinterpretq_u8_u32(vsetq_lane_u32::<0>(*ptr, vdupq_n_u32(0u32)))
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
