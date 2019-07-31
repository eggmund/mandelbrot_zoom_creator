mod mandelbrot;
mod options;

use mandelbrot::Mandelbrot;
use options::Opt;

use sfml::system::{Clock, Vector2};

use structopt::StructOpt;

use rug::{Complex, Float};

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread;

pub static PRECISION: u32 = 128; // Precision of complex and float numbers (bits)
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
fn max_iterations_increase_formula(mut max_iter: usize, zoom_speed: f64, index: usize) -> usize {
    // Returns new max iter value
    for _ in 0..index {
        max_iter = (max_iter as f64 * ((zoom_speed - 1.0) / 20.0 + 1.0)).floor() as usize
    }
    max_iter
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

    let num_cpu = if opt.cpu_num_to_use != 0 {
        opt.cpu_num_to_use
    } else {
        num_cpus::get()
    };
    let mandel: Arc<RwLock<Mandelbrot>> = Arc::new(RwLock::new(Mandelbrot::new(
        2000, //INITIAL_MAX_ITER,
        Complex::with_val(
            PRECISION,
            (Float::parse_radix(&opt.real_focus, 10).unwrap(), Float::parse_radix(&opt.imaginary_focus, 10).unwrap()),
        ),
        Float::with_val(PRECISION, 1_000_000_000_000_000_000.0)
    )));

    // Where to store generated frames
    let frames_location = Arc::new(opt.output_path.join("frames"));

    let clock = Clock::start();
    let mut last_time = clock.elapsed_time();

    let iters = (opt.frame_count as f32 / num_cpu as f32).ceil() as usize;
    //let extra_to_do = iters - opt.frame_count;
    //println!("Extra frames to do at the end: {}", extra_to_do);

    for i in 0..iters {
        let mut child_threads = Vec::with_capacity(num_cpu);

        for j in 0..num_cpu {
            let frames_location = Arc::clone(&frames_location);
            let opt = Arc::clone(&opt);
            let mandel = Arc::clone(&mandel);

            child_threads.push(thread::spawn(move || {
                let this_mandel = {
                    // Copy stuff from parent mandel
                    let parent_mandel = mandel.try_read().unwrap(); // Can just unwrap since while in the thread it should always be availabe to read.
                    let mut this_mandel = parent_mandel.clone();

                    let my_zoom_speed = opt.zoom_speed.powi(j as i32);
                    this_mandel.max_iter = max_iterations_increase_formula(this_mandel.max_iter, opt.zoom_speed, j);
                    this_mandel.zoom *= my_zoom_speed;
                    this_mandel
                };

                println!("About to generate");
                let image = this_mandel.generate_image(dims);
                println!("Finished generating");
                let save_loc = frames_location.join(&format!("frame_{}.png", (i * num_cpu) + j));

                image.save_to_file(save_loc.to_str().unwrap());

                println!("Saved image to: {:#?}", save_loc);
            }));
        }

        // Wait for each to finish
        for child in child_threads {
            child.join().unwrap();
        }

        // Increase mandelbrot values for base mandelbrot data, increasing zoom by a factor of num_core
        let next_max_iter = {
            let mut mandel_write = mandel.try_write().unwrap();
            let overall_zoom_speed = opt.zoom_speed.powi(num_cpu as i32);
            mandel_write.change_zoom_by(Float::with_val(PRECISION, overall_zoom_speed));
            mandel_write.max_iter =
                max_iterations_increase_formula(mandel_write.max_iter, overall_zoom_speed, num_cpu);
            mandel_write.max_iter
        };

        // Calculate framerate and other output
        let current_time = clock.elapsed_time();
        let frame_rate = 1.0 / (current_time - last_time).as_seconds();
        last_time = current_time;

        let frame_count_left = opt.frame_count - (i * num_cpu) + num_cpu;
        let time_left = frame_count_left as f32 / frame_rate;

        println!(
            "Frame number: {} of {}, Frames per second: {:.6}, Time left (seconds): {:.2}, Next max iterations: {}",
            (i * num_cpu) + num_cpu,
            opt.frame_count,
            frame_rate,
            time_left,
            next_max_iter
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
