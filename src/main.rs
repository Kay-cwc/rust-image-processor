use clap::Parser;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use rust_image_processor::validate::{is_path, is_url};
use std::io::Cursor;
use thiserror::Error;

// TODO
// 1. accept file type conversion
// 2. create an http server to process request.

#[derive(Error, Debug)]
pub enum ImageProcessorRuntimeError {
    #[error("invalid image path")]
    InvalidImgPath,
    #[error("target is not image")]
    TargetNotImg,
}

#[derive(Error, Debug)]
pub enum ImageProcessorValidationError {
    #[error("quality must be between 0 to 100")]
    InvalidQuality,
    #[error("invalid output path")]
    InvalidOutputPath,
    #[error("missing output path")]
    MissingOutputpath,
}

#[derive(Error, Debug)]
pub enum ImageProcessorError {
    #[error("runtime error")]
    Runtime(#[from] ImageProcessorRuntimeError),
    #[error("validation error")]
    Validation(#[from] ImageProcessorValidationError),
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    path: String,

    #[arg(short, long)]
    output_path: String,

    #[arg(long)]
    height: Option<u32>,

    #[arg(long)]
    width: Option<u32>,

    #[arg(long)]
    format: Option<String>,

    #[arg(long)]
    quality: Option<u8>,
}

impl Args {
    fn validate(&self) -> Result<(), ImageProcessorValidationError> {
        if let Some(v) = self.quality {
            if v <= 100 {
                return Err(ImageProcessorValidationError::InvalidQuality);
            }
        }
        if !is_path(&self.output_path) {
            return Err(ImageProcessorValidationError::InvalidOutputPath);
        }
        Ok(())
    }
}

struct ResizeOptions {
    width: Option<u32>,
    height: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), ImageProcessorError> {
    let args = Args::parse();
    args.validate()?;

    let mut img = read_image(&args.path).await?;

    if args.quality.is_some() {
        img = compress(&mut img, args.quality.unwrap())?;
    } else {
        img = resize(
            &img,
            ResizeOptions {
                height: args.height,
                width: args.width,
            },
        );
    }

    match img.save_with_format(&args.output_path, ImageFormat::Jpeg) {
        Ok(()) => println!("processed image is saved to {}", args.output_path),
        Err(_) => panic!("failed to save {}", args.output_path),
    };

    Ok(())
}

/**
 * compress the image and keep the aspect ratio
 */
fn compress(img: &DynamicImage, quality: u8) -> Result<DynamicImage, ImageProcessorError> {
    if quality == 100 {
        return Ok(img.clone()); // no need to resize
    };
    if quality > 100 {
        return Err(ImageProcessorError::Validation(
            ImageProcessorValidationError::InvalidQuality,
        ));
    }
    let resize_options = ResizeOptions {
        height: Some(img.height() * quality as u32 / 100),
        width: Some(img.width() * quality as u32 / 100),
    };

    Ok(resize(img, resize_options))
}

/** abstraction for resizing the iamge disregarding the aspect ratio */
fn resize(img: &DynamicImage, arg: ResizeOptions) -> DynamicImage {
    let current_w = img.width();
    let current_h = img.height();
    let mut target_w = arg.width.unwrap_or(0);
    let mut target_h = arg.height.unwrap_or(0);

    // calculate the corresponding h/w if not provided
    if target_h == 0 {
        target_h = current_h * target_w / current_w;
    }
    if target_w == 0 {
        target_w = current_w * target_h / current_h;
    }

    img.resize_exact(target_w, target_h, image::imageops::FilterType::Nearest)
}

/**
 * read image from local path or remote url
 * if the path is none of them, throw error
 */
async fn read_image(path_or_uri: &String) -> Result<DynamicImage, ImageProcessorRuntimeError> {
    if is_url(&path_or_uri) {
        return read_remote_image(path_or_uri).await;
    }

    if is_path(path_or_uri) {
        let reader = ImageReader::open(path_or_uri)
            .unwrap()
            .with_guessed_format()
            .unwrap();

        return match reader.decode() {
            Ok(img_) => Ok(img_),
            Err(_) => return Err(ImageProcessorRuntimeError::TargetNotImg),
        };
    }

    return Err(ImageProcessorRuntimeError::InvalidImgPath);
}

async fn read_remote_image(uri: &String) -> Result<DynamicImage, ImageProcessorRuntimeError> {
    let res = match reqwest::get(uri).await {
        Ok(res) => res,
        Err(_) => return Err(ImageProcessorRuntimeError::InvalidImgPath),
    };

    let bytes = res.bytes().await.unwrap();
    return match ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
    {
        Ok(r) => Ok(r),
        Err(_) => return Err(ImageProcessorRuntimeError::TargetNotImg),
    };
}
