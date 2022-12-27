use std::arch::wasm32::*;
#[test]
fn convolution_one_row() {
    let mut ll_sum = i64x2(0, 0);
    let k: [i64; 8] = [3, 1, 4, 1, 5, 9, 2, 6];
    let source = u16x8(129, 380, 867, 408, 235, 809, 7824, 4752);

    let l01_shuffle = i8x16(-1, -1, -1, -1, -1, -1, 0, 1, -1, -1, -1, -1, -1, -1, 0, 1);
    let l23_shuffle = i8x16(-1, -1, -1, -1, -1, -1, 4, 5, -1, -1, -1, -1, -1, -1, 6, 7);
    let l45_shuffle = i8x16(-1, -1, -1, -1, -1, -1, 8, 9, -1, -1, -1, -1, -1, -1, 10, 11);
    let l67_shuffle = i8x16(
        -1, -1, -1, -1, -1, -1, 12, 13, -1, -1, -1, -1, -1, -1, 14, 15,
    );

    let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
    let coeff23_i64x2 = i64x2(k[2] as i64, k[3] as i64);
    let coeff45_i64x2 = i64x2(k[4] as i64, k[5] as i64);
    let coeff67_i64x2 = i64x2(k[6] as i64, k[7] as i64);
    let l_i64x2 = i8x16_swizzle(source, l01_shuffle);
    ll_sum = i64x2_add(ll_sum, i64x2_mul(l_i64x2, coeff01_i64x2));

    let l_i64x2 = i8x16_swizzle(source, l23_shuffle);
    ll_sum = i64x2_add(ll_sum, i64x2_mul(l_i64x2, coeff23_i64x2));

    let l_i64x2 = i8x16_swizzle(source, l45_shuffle);
    ll_sum = i64x2_add(ll_sum, i64x2_mul(l_i64x2, coeff45_i64x2));

    let l_i64x2 = i8x16_swizzle(source, l67_shuffle);
    ll_sum = i64x2_add(ll_sum, i64x2_mul(l_i64x2, coeff67_i64x2));
    let simd_answer = i64x2_extract_lane::<0>(ll_sum) + i64x2_extract_lane::<1>(ll_sum);
    // Non simd calculation
    let source: [u16; 8] = [129, 380, 867, 408, 235, 809, 7824, 4752];
    let native_ans = source
        .into_iter()
        .map(|x| k.into_iter().map(|ki| x as i64 * ki as i64).sum::<i64>())
        .sum::<i64>();
    assert_eq!(native_ans, simd_answer);
}
