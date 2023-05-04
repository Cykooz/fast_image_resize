use std::arch::wasm32::*;

use crate::pixels::{U8x3, U8x4};

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn load_v128<T>(buf: &[T], index: usize) -> v128 {
    v128_load(buf.get_unchecked(index..).as_ptr() as *const v128)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn loadl_i64<T>(buf: &[T], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const i64;
    i64x2(p.read_unaligned(), 0)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn loadl_i32<T>(buf: &[T], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const i32;
    i32x4(p.read_unaligned(), 0, 0, 0)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn loadl_i16<T>(buf: &[T], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const i16;
    i16x8(p.read_unaligned(), 0, 0, 0, 0, 0, 0, 0)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn ptr_i16_to_set1_i64(buf: &[i16], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const i64;
    i64x2_splat(p.read_unaligned())
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn ptr_i16_to_set1_i32(buf: &[i16], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const i32;
    i32x4_splat(p.read_unaligned())
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn i32x4_extend_low_ptr_u8(buf: &[u8], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const v128;
    u32x4_extend_low_u16x8(i16x8_extend_low_u8x16(v128_load(p)))
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn i32x4_extend_low_ptr_u8x4(buf: &[U8x4], index: usize) -> v128 {
    let v: u32 = buf.get_unchecked(index).0;
    u32x4_extend_low_u16x8(i16x8_extend_low_u8x16(u32x4(v, 0, 0, 0)))
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn i32x4_extend_low_ptr_u8x3(buf: &[U8x3], index: usize) -> v128 {
    let pixel = buf.get_unchecked(index).0;
    i32x4(pixel[0] as i32, pixel[1] as i32, pixel[2] as i32, 0)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn i32x4_v128_from_u8(buf: &[u8], index: usize) -> v128 {
    let p = buf.get_unchecked(index..).as_ptr() as *const i32;
    i32x4(p.read_unaligned(), 0, 0, 0)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn u16x8_mul_shr16(a_u16x8: v128, b_u16x8: v128) -> v128 {
    let lo_u32x4 = u32x4_extmul_low_u16x8(a_u16x8, b_u16x8);
    let hi_u32x4 = u32x4_extmul_high_u16x8(a_u16x8, b_u16x8);
    i16x8_shuffle::<1, 3, 5, 7, 9, 11, 13, 15>(lo_u32x4, hi_u32x4)
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn i64x2_mul_lo(a: v128, b: v128) -> v128 {
    const SHUFFLE: v128 = i8x16(0, 1, 2, 3, 8, 9, 10, 11, -1, -1, -1, -1, -1, -1, -1, -1);
    i64x2_extmul_low_i32x4(i8x16_swizzle(a, SHUFFLE), i8x16_swizzle(b, SHUFFLE))
}
