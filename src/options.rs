use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "mandelzoom", about = "Mandelbrot zoom video creator.")]
pub struct Opt {
    /// Where to output to. Will also make a frames folder to contain frames
    #[structopt(parse(from_os_str))]
    pub output_path: PathBuf,

    /// Framerate of video output
    #[structopt(short = "f", long = "framerate", default_value = "60")]
    pub framerate: f64,

    /// Width of video
    #[structopt(short = "w", long = "width", default_value = "1280")]
    pub width: u32,

    /// Height of video
    #[structopt(short = "h", long = "height", default_value = "720")]
    pub height: u32,

    /// Real focus
    #[structopt(short = "r", long = "real-focus", default_value = "0")]
    pub real_focus: f64,

    /// Imaginary focus
    #[structopt(short = "i", long = "imaginary-focus", default_value = "0")]
    pub imaginary_focus: f64,
}