use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum MulDivImagesError {
    #[error("Size of source image does not match to destination image")]
    SizeIsDifferent,
    #[error("Pixel type of source image does not match to destination image")]
    PixelTypeIsDifferent,
    #[error("Pixel type of image is not supported")]
    UnsupportedPixelType,
}

#[derive(Error, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum MulDivImageError {
    #[error("Pixel type of image is not supported")]
    UnsupportedPixelType,
}
