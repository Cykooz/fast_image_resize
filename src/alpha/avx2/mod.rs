pub(crate) use div::{divide_alpha_avx2, divide_alpha_inplace_avx2};
pub(crate) use mul::{multiply_alpha_avx2, multiply_alpha_inplace_avx2};

mod div;
mod mul;
