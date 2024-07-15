use actix_web::{get, http::header::ContentType, App, HttpResponse, HttpServer, Responder};
use actix_web_validator;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use serde::Deserialize;
use std::io::Cursor;
use thiserror::Error;
use validator::Validate;
use rust_image_processor::validate::is_url;

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

#[derive(Deserialize, Validate, Debug)]
struct Args {
    url: String,
    #[validate(range(min=1, max=100))]
    quality: Option<u8>,
}

struct ResizeOptions {
    width: Option<u32>,
    height: Option<u32>,
}

#[get("/")]
async fn resize_handler(query: actix_web_validator::Query<Args>) -> impl Responder {
    // oad image from remote and parse as dynamic image
    if !is_url(&query.url) {
        return HttpResponse::BadRequest().body("invalid url");
    }
    let mut img = match read_remote_image(&query.url).await {
        Ok(img_) => img_,
        Err(err) => return HttpResponse::BadRequest().body(err.to_string())
    };
    // compress image
    img = compress(&mut img, query.quality.unwrap_or(100)).expect("failed to compress image");
    // process the image as bytes response
    let format = detect_image_format(&img);
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), format).expect("failed to convert to buffer");
    HttpResponse::Ok()
        .append_header(("accept-ranges", "bytes"))
        .content_type(ContentType::png())
        .body(bytes)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(resize_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn detect_image_format(img: &DynamicImage) -> ImageFormat {
    match img.color() {
        image::ColorType::L8 => ImageFormat::Png,
        image::ColorType::La8 => ImageFormat::Png,
        image::ColorType::Rgb8 => ImageFormat::Png,
        image::ColorType::Rgba8 => ImageFormat::Png,
        image::ColorType::L16 => ImageFormat::Png,
        image::ColorType::La16 => ImageFormat::Png,
        image::ColorType::Rgb16 => ImageFormat::Png,
        image::ColorType::Rgba16 => ImageFormat::Png,
        image::ColorType::Rgb32F => ImageFormat::OpenExr,
        image::ColorType::Rgba32F => ImageFormat::OpenExr,
        _ => ImageFormat::Png, // Default to PNG for unknown formats
    }
}

/**
 * compress the image and keep the aspect ratio
 */
fn compress(img: &DynamicImage, quality: u8) -> Result<DynamicImage, ImageProcessorError> {
    if quality == 100 {
        return Ok(img.clone()); // no need to resize. // FIXME: is this the only way?
    };
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
