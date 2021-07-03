use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum ImageError {
    #[error("Buffer size don't corresponds to image dimensions")]
    InvalidBufferSize,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum CropBoxError {
    #[error("Position of the crop box is out of the image boundaries")]
    PositionIsOutOfImageBoundaries,
    #[error("Size of the crop box is out of the image boundaries")]
    SizeIsOutOfImageBoundaries,
}
