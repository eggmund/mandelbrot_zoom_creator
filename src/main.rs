mod mandelbrot;
mod options;
mod threadpool;

use mandelbrot::Mandelbrot;
use options::Opt;

use sfml::system::Clock;

use structopt::StructOpt;

use rug::{Complex, Float};

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

pub static PRECISION: u32 = 64; // Precision of complex and float numbers (bits)
pub const INITIAL_MAX_ITER: usize = 100;

fn create_or_clear_output_location(path: &Path) {
    if let Err(_) = fs::create_dir(path) {
        // If dir already exists, clear it
        fs::remove_dir_all(path).unwrap();
        fs::create_dir(path).unwrap();
    }

    fs::create_dir(path.join("frames")).unwrap()
}

#[inline]
fn max_iterations_increase_formula(max_iter: usize, zoom_speed: f64) -> usize {
    // Returns new max iter value
    (max_iter as f64 * ((zoom_speed - 1.0) / 20.0 + 1.0)).floor() as usize
}

fn main() {
    let opt = Arc::new(Opt::from_args());

    if opt.video_quality > 51 {
        eprintln!(
            "mandelzoom: Error, video quality chosen not in range 0 - 51: {}",
            opt.video_quality
        );
        std::process::exit(1);
    }

    create_or_clear_output_location(&opt.output_path);

    let dims = (opt.width, opt.height);

    let thread_num = if opt.thread_num != 0 {
        opt.thread_num
    } else {
        num_cpus::get()
    };
    let mut mandel: Mandelbrot = Mandelbrot::new(
        INITIAL_MAX_ITER,
        Complex::with_val(
            PRECISION,
            (
                Float::parse_radix(&opt.real_focus, 10).unwrap(),
                Float::parse_radix(&opt.imaginary_focus, 10).unwrap(),
            ),
        ),
        Float::with_val(PRECISION, 1.0),
        thread_num,
    );

    // Where to store generated frames
    let frames_location = opt.output_path.join("frames");

    let clock = Clock::start();
    let mut last_time = clock.elapsed_time();

    for i in 0..opt.frame_count {
        let image = mandel.generate_image(dims);
        image.save_to_file(
            frames_location
                .join(format!("frame_{}.png", i))
                .to_str()
                .unwrap(),
        );

        // Increase mandelbrot values
        mandel.change_zoom_by(opt.zoom_speed);
        mandel.max_iter = max_iterations_increase_formula(mandel.max_iter, opt.zoom_speed);

        // Calculate framerate and other output
        let current_time = clock.elapsed_time();
        let frame_rate = 1.0 / (current_time - last_time).as_seconds();
        last_time = current_time;

        let time_left = (opt.frame_count - (i + 1)) as f32 / frame_rate;

        println!(
            "Frame number: {} of {}, Frames per second: {:.6}, Time left (seconds): {:.2}, Next max iterations: {}",
            i + 1,
            opt.frame_count,
            frame_rate,
            time_left,
            mandel.max_iter
        );
    }

    println!("Done!");

    if !opt.no_video {
        // Turn images into final video
        let output = Command::new("sh")
            .args(&[
                "-c",
                &format!(
                    "ffmpeg -y -loglevel 24 -s {width}x{height} -i {frames_loc}/frame_%d.png -crf {quality} -vf 'fps={framerate},format=yuv420p' {output_video_loc}",
                    framerate = opt.framerate,
                    width = opt.width,
                    height = opt.height,
                    frames_loc = frames_location.to_str().unwrap(),
                    output_video_loc = opt.output_path.join("output.mp4").to_str().unwrap(),
                    quality = opt.video_quality,
                )
            ])
            .output()
            .expect("Failed to turn into video :(");

        println!(
            "ffmpeg warnings/errors: {}",
            String::from_utf8(output.stderr).unwrap()
        );
    }
}
