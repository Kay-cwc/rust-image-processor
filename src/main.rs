use clap::Parser;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    path: String,

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

const OUT_DIR: &str = "./output";

fn main() {
    let args = Args::parse();
    let path = validate_path(&args.path);

    let mut img = read_image(path);

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

    let output_path = get_output_path(&args.path);

    match img.save_with_format(&output_path, ImageFormat::Jpeg) {
        Ok(()) => println!("{}", output_path),
        Err(_) => panic!("failed to save {}", output_path)
    };
}

fn get_output_path(org_path: &String) -> String {
    let filename = org_path.split("/").last().unwrap();
    let fragments = filename.split('.').collect::<Vec<_>>();
    let base_filename = fragments[0..fragments.len()-1].concat();

    let output_path = format!(
        "{base}/{filename}_formatted.{ext}",
        base=OUT_DIR,
        filename=base_filename,
        ext=fragments.last().unwrap()
    );

    output_path
}

fn compress(img: &DynamicImage, quality: u8) -> DynamicImage {
    if quality == 100 {
        return img.clone() // no need to resize
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

    print!("{}", target_w);
    print!("{}", target_h);
    // calculate the corresponding h/w if not provided
    if target_h == 0 {
        target_h = current_h * target_w / current_w;
    }
    if target_w == 0 {
        target_w = current_w * target_h / current_h;
    }

    img.resize_exact(target_w, target_h, image::imageops::FilterType::Nearest)
}

fn read_image(path: &std::path::Path) -> DynamicImage {
    let reader = ImageReader::open(path)
        .unwrap()
        .with_guessed_format()
        .unwrap();

    let img = match reader.decode() {
        Ok(img_) => img_,
        Err(_) => panic!("unable to decode image"),
    };

    img
}

fn validate_path(path_: &String) -> &std::path::Path {
    let path = std::path::Path::new(path_);

    if !path.is_file() {
        panic!("{} is not a valid file", path_);
    };

    path
}
