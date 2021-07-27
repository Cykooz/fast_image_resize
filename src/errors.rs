use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum ImageRowsError {
    #[error("Count of rows don't match to image height")]
    InvalidRowsCount,
    #[error("Size of row don't match to image width")]
    InvalidRowSize,
}

#[derive(Error, Debug, Clone, Copy)]
#[error("Size of buffer don't match to image dimensions")]
pub struct InvalidBufferSizeError;

#[derive(Error, Debug, Clone, Copy)]
pub enum ImageBufferError {
    #[error("Size of buffer don't match to image dimensions")]
    InvalidBufferSize,
    #[error("Alignment of buffer don't match to alignment of u32")]
    InvalidBufferAlignment,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum CropBoxError {
    #[error("Position of the crop box is out of the image boundaries")]
    PositionIsOutOfImageBoundaries,
    #[error("Size of the crop box is out of the image boundaries")]
    SizeIsOutOfImageBoundaries,
}
