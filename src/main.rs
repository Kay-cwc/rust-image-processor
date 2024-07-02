use clap::Parser;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use rust_image_processor::validate::{is_path, is_url};
use std::io::Cursor;

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

struct ResizeOptions {
    width: Option<u32>,
    height: Option<u32>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if !is_path(&args.output_path) {
        panic!("invalid output path")
    }

    let mut img = read_image(&args.path).await;

    if args.quality.is_some() {
        img = compress(&mut img, args.quality.unwrap());
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
}

/**
 * compress the image and keep the aspect ratio
 */
fn compress(img: &DynamicImage, quality: u8) -> DynamicImage {
    if quality == 100 {
        return img.clone(); // no need to resize
    };
    if quality > 100 {
        panic!("quality must be between 0-100")
    }
    let resize_options = ResizeOptions {
        height: Some(img.height() * quality as u32 / 100),
        width: Some(img.width() * quality as u32 / 100),
    };

    resize(img, resize_options)
}

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
async fn read_image(path_or_uri: &String) -> DynamicImage {
    if is_url(&path_or_uri) {
        return read_remote_image(path_or_uri).await;
    }

    if is_path(path_or_uri) {
        let reader = ImageReader::open(path_or_uri)
            .unwrap()
            .with_guessed_format()
            .unwrap();

        return match reader.decode() {
            Ok(img_) => img_,
            Err(_) => panic!("unable to decode image"),
        };
    }

    panic!("path must be either a uri or a local path")
}

async fn read_remote_image(uri: &String) -> DynamicImage {
    let res = match reqwest::get(uri).await {
        Ok(res) => res,
        Err(_) => panic!("failed to fetch image from {}", uri),
    };

    let bytes = res.bytes().await.unwrap();
    return match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
        Ok(r) => r.decode().unwrap(),
        Err(_) => panic!("unrecognized image type"),
    };
}
