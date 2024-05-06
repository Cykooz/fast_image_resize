use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImageError {
    #[error("Pixel type of image is not supported")]
    UnsupportedPixelType,
}

#[derive(Error, Debug, Clone, Copy)]
#[error("Size of container with pixels is smaller than required")]
pub struct InvalidPixelsSize;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageBufferError {
    #[error("Size of buffer is smaller than required")]
    InvalidBufferSize,
    #[error("Alignment of buffer don't match to alignment of required pixel type")]
    InvalidBufferAlignment,
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropBoxError {
    #[error("Position of the crop box is out of the image boundaries")]
    PositionIsOutOfImageBoundaries,
    #[error("Size of the crop box is out of the image boundaries")]
    SizeIsOutOfImageBoundaries,
    #[error("Width or height of the crop box is less than zero")]
    WidthOrHeightLessThanZero,
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResizeError {
    #[error("Source or destination image is not supported")]
    ImageError(#[from] ImageError),
    #[error("Pixel type of source image does not match to destination image")]
    PixelTypesAreDifferent,
    #[error("Source cropping option is invalid: {0}")]
    SrcCroppingError(#[from] CropBoxError),
}

#[derive(Error, Debug, Clone, Copy)]
#[error(
    "The dimensions of the source image are not equal to the dimensions of the destination image"
)]
pub struct DifferentDimensionsError;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingError {
    #[error("Source or destination image is not supported")]
    ImageError(#[from] ImageError),
    #[error("The dimensions of the source image are not equal to the dimensions of the destination image")]
    DifferentDimensions,
    #[error("Unsupported combination of pixels of source and/or destination images")]
    UnsupportedCombinationOfImageTypes,
}

impl From<DifferentDimensionsError> for MappingError {
    fn from(_: DifferentDimensionsError) -> Self {
        MappingError::DifferentDimensions
    }
}
