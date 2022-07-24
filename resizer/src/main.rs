use std::ffi::OsStr;
use std::fmt::Debug;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use image::io::Reader as ImageReader;
use image::ColorType;
use log::debug;

use fast_image_resize as fr;

mod structs;

#[derive(Parser)]
#[clap(author = "Kirill K.")]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Path to source image file
    #[clap(value_parser)]
    source_path: PathBuf,

    /// Path to result image file
    #[clap(value_parser)]
    destination_path: Option<PathBuf>,

    /// Width of result image
    #[clap(short, long, value_parser)]
    width: Option<NonZeroU32>,

    /// Height of result image
    #[clap(short, long, value_parser)]
    height: Option<NonZeroU32>,

    /// Overwrite destination file
    #[clap(short, long, action)]
    overwrite: bool,

    /// Algorithm used to resize image
    #[clap(short, long, value_enum, default_value_t = structs::Algorithm::Convolution)]
    algorithm: structs::Algorithm,

    /// Type of filter used with the "convolution" resizing algorithm
    #[clap(short, long, value_enum, default_value_t = structs::FilterType::Lanczos3)]
    filter: structs::FilterType,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();
    resize(&cli)
}

fn resize(cli: &Cli) -> Result<()> {
    let (mut src_image, color_type) = open_source_image(&cli.source_path)?;
    let mut dst_image = create_destination_image(cli, &src_image);

    let mul_div = fr::MulDiv::default();
    let algorithm = get_resizing_algorithm(cli);
    let mut resizer = fr::Resizer::new(algorithm);

    if color_type.has_alpha() {
        debug!("Multiply color channels of the source image by alpha channel");
        mul_div
            .multiply_alpha_inplace(&mut src_image.view_mut())
            .with_context(|| "Failed to multiply color channels by alpha")?;
    }

    debug!(
        "Resize the source image into {}x{}",
        dst_image.width(),
        dst_image.height()
    );
    resizer
        .resize(&src_image.view(), &mut dst_image.view_mut())
        .with_context(|| "Failed to resize image")?;

    if color_type.has_alpha() {
        debug!("Divide color channels of the result image by alpha channel");
        mul_div
            .divide_alpha_inplace(&mut dst_image.view_mut())
            .with_context(|| "Failed to divide color channels by alpha")?;
    }

    save_result(cli, &dst_image, color_type)
}

fn open_source_image<P>(source_path: P) -> Result<(fr::Image<'static>, ColorType)>
where
    P: AsRef<Path> + Debug,
{
    debug!("Opening the source image {:?}", source_path);
    let image = ImageReader::open(&source_path)
        .with_context(|| format!("Failed to read source file from {:?}", source_path))?
        .decode()
        .with_context(|| "Failed to decode source image")?;

    let src_width = NonZeroU32::new(image.width())
        .with_context(|| "Failed to get width of the source image")?;
    let src_height = NonZeroU32::new(image.height())
        .with_context(|| "Failed to get height of the source image")?;

    let color_type = image.color();
    let (src_buffer, pixel_type) = match color_type {
        ColorType::L8 => (image.to_luma8().into_raw(), fr::PixelType::U8),
        ColorType::La8 => (image.to_luma_alpha8().into_raw(), fr::PixelType::U8x2),
        ColorType::Rgb8 => (image.to_rgb8().into_raw(), fr::PixelType::U8x3),
        ColorType::Rgba8 => (image.to_rgba8().into_raw(), fr::PixelType::U8x4),
        ColorType::L16 => (
            image
                .to_luma16()
                .as_raw()
                .iter()
                .flat_map(|&c| c.to_le_bytes())
                .collect(),
            fr::PixelType::U16,
        ),
        ColorType::La16 => (
            image
                .to_luma_alpha16()
                .as_raw()
                .iter()
                .flat_map(|&c| c.to_le_bytes())
                .collect(),
            fr::PixelType::U16x2,
        ),
        ColorType::Rgb16 => (
            image
                .to_rgb16()
                .as_raw()
                .iter()
                .flat_map(|&c| c.to_le_bytes())
                .collect(),
            fr::PixelType::U16x3,
        ),
        ColorType::Rgba16 => (
            image
                .to_rgba16()
                .as_raw()
                .iter()
                .flat_map(|&c| c.to_le_bytes())
                .collect(),
            fr::PixelType::U16x4,
        ),
        _ => {
            return Err(anyhow!(
                "Unsupported pixel's format of source image: {:?}",
                color_type
            ))
        }
    };

    Ok((
        fr::Image::from_vec_u8(src_width, src_height, src_buffer, pixel_type)
            .with_context(|| "Failed to create source image pixels container")?,
        color_type,
    ))
}

fn create_destination_image(cli: &Cli, src_image: &fr::Image) -> fr::Image<'static> {
    let aspect_ratio = src_image.width().get() as f32 / src_image.height().get() as f32;

    let (dst_width, dst_height) = match (cli.width, cli.height) {
        (None, None) => (src_image.width(), src_image.height()),
        (Some(width), None) => (
            width,
            get_non_zero_u32((width.get() as f32 / aspect_ratio).round() as u32),
        ),
        (None, Some(height)) => (
            get_non_zero_u32((height.get() as f32 * aspect_ratio).round() as u32),
            height,
        ),
        (Some(width), Some(height)) => (width, height),
    };

    fr::Image::new(dst_width, dst_height, src_image.pixel_type())
}

fn get_non_zero_u32(v: u32) -> NonZeroU32 {
    NonZeroU32::new(v).unwrap_or(NonZeroU32::new(1).unwrap())
}

fn get_resizing_algorithm(cli: &Cli) -> fr::ResizeAlg {
    let filter_type: fr::FilterType = cli.filter.into();
    match cli.algorithm {
        structs::Algorithm::Nearest => fr::ResizeAlg::Nearest,
        structs::Algorithm::Convolution => fr::ResizeAlg::Convolution(filter_type),
        structs::Algorithm::SuperSampling => fr::ResizeAlg::SuperSampling(filter_type, 2),
    }
}

fn save_result(cli: &Cli, image: &fr::Image, color_type: ColorType) -> Result<()> {
    let result_path = if let Some(path) = cli.destination_path.clone() {
        path
    } else {
        let mut path = PathBuf::from("./");
        let ext = cli
            .source_path
            .extension()
            .unwrap_or_else(|| OsStr::new("png"));
        path.push("result");
        path.set_extension(ext);
        path
    };
    if result_path.exists() && !cli.overwrite {
        return Err(anyhow!(
            "Destination path {:?} already exists.",
            result_path
        ));
    };
    debug!("Save the result image into the file {:?}", result_path);
    image::save_buffer(
        result_path,
        image.buffer(),
        image.width().get(),
        image.height().get(),
        color_type,
    )
    .with_context(|| "Failed to save the result image")?;
    Ok(())
}
