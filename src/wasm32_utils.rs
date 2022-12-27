use std::arch::wasm32::*;

#[inline(always)]
pub unsafe fn load_v128<T>(buf: &[T], index: usize) -> v128 {
    v128_load(buf.get_unchecked(index..).as_ptr() as *const v128)
}

#[inline(always)]
pub unsafe fn loadl_i64<T>(buf: &[T], index: usize) -> v128 {
    i64x2(buf.get_unchecked(index..).as_ptr() as i64, 0)
}

#[inline(always)]
pub unsafe fn loadl_i32<T>(buf: &[T], index: usize) -> v128 {
    i32x4(buf.get_unchecked(index..).as_ptr() as i32, 0, 0, 0)
}

#[inline(always)]
pub unsafe fn loadl_i16<T>(buf: &[T], index: usize) -> v128 {
    i16x8(
        buf.get_unchecked(index..).as_ptr() as i16,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

#[inline(always)]
pub fn i8x16_shuffle(a: v128, b: v128) -> v128 {
    u8x16_swizzle(a, v128_and(b, i8x16_splat(-113)))
}

#[inline(always)]
pub unsafe fn ptr_i16_to_set1_i64(buf: &[i16], index: usize) -> v128 {
    i64x2(*(buf.get_unchecked(index..).as_ptr() as *const i64), 0)
}

#[inline(always)]
pub unsafe fn ptr_i16_to_set1_i32(buf: &[i16], index: usize) -> v128 {
    i32x4(
        *(buf.get_unchecked(index..).as_ptr() as *const i32),
        0,
        0,
        0,
    )
}
