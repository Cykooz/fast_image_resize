pub trait FloatExt {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn powf(self, exp: Self) -> Self;
}

impl FloatExt for f32 {
    fn sin(self) -> Self {
        libm::sinf(self)
    }
    fn cos(self) -> Self {
        libm::cosf(self)
    }
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }
    fn exp(self) -> Self {
        libm::expf(self)
    }
    fn powf(self, exp: f32) -> Self {
        libm::powf(self, exp)
    }
}

impl FloatExt for f64 {
    fn sin(self) -> Self {
        libm::sin(self)
    }
    fn cos(self) -> Self {
        libm::cos(self)
    }
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }
    fn exp(self) -> Self {
        libm::exp(self)
    }
    fn powf(self, exp: f64) -> Self {
        libm::pow(self, exp)
    }
}
