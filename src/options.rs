use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "mandelzoom", about = "Mandelbrot zoom video creator.")]
pub struct Opt {
    /// Where to output to. Will also make a frames folder to contain frames.
    #[structopt(parse(from_os_str), default_value = "./output")]
    pub output_path: PathBuf,

    /// Framerate of video output.
    #[structopt(short = "f", long = "framerate", default_value = "60")]
    pub framerate: f64,

    /// Number of frames to render.
    #[structopt(short = "c", long = "frame-count", default_value = "120")]
    pub frame_count: usize,

    /// Zoom multiplier by frame.
    #[structopt(short = "z", long = "zoom-speed", default_value = "1.05")]
    pub zoom_speed: f64,

    /// Width of video.
    #[structopt(short = "w", long = "width", default_value = "1280")]
    pub width: u32,

    /// Height of video.
    #[structopt(short = "h", long = "height", default_value = "720")]
    pub height: u32,

    /// Real focus.
    #[structopt(
        short = "r",
        long = "real-focus",
        default_value = "-1.769383179195515018213847286085473782905747263654751437465528216527888191264756458836163446389529667304485825781820303157487491238421719403128246195113747521255072180379778727429039155281457214"
    )]
    pub real_focus: String,

    /// Imaginary focus.
    #[structopt(
        short = "i",
        long = "imaginary-focus",
        default_value = "0.004236847918736772214926507171367997076682670917403757279459435650112344000805545157302430995023636506313532683359652571823004948055387363061275248149392923559310270429656787009247955481321870"
    )]
    pub imaginary_focus: String,

    /// ffmpeg quality input. Range is 0 - 51. Lower = better quality (0 = lossless), but at cost of file size.
    #[structopt(short = "q", long = "video-quality", default_value = "16")]
    pub video_quality: u8,

    /// Number of threads to use to render the frames. Defaults to number of CPU cores in machine.
    #[structopt(short = "n", long = "thread-num", default_value = "0")]
    pub thread_num: usize,

    /// Do not convert frames into video at the end.
    #[structopt(long = "no-video")]
    pub no_video: bool,

    /// Initial maximum iterations to use. Higher = more defined boundaries, at cost of performance.
    #[structopt(short = "m", long = "max-iters", default_value = "100")]
    pub initial_max_iters: usize,
}
