use thiserror::Error;

use crate::ImageError;

#[derive(Error, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum MulDivImagesError {
    #[error("Source or destination image is not supported")]
    ImageError(#[from] ImageError),
    #[error("Size of source image does not match to destination image")]
    SizeIsDifferent,
    #[error("Pixel type of source image does not match to destination image")]
    PixelTypesAreDifferent,
}
