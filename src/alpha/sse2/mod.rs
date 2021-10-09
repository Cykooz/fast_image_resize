pub(crate) use div::{divide_alpha_inplace_sse2, divide_alpha_sse2};
pub(crate) use mul::multiply_alpha_sse2;

mod div;
mod mul;
