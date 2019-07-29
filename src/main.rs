#[macro_use]
extern crate structopt;

mod options;
mod mandelbrot;

use mandelbrot::Mandelbrot;
use options::Opt;

use sfml::system::{Clock, Vector2};

use structopt::StructOpt;

use std::process::Command;
use std::path::Path;
use std::fs;
use std::collections::VecDeque;


fn create_or_clear_output_location(path: &Path) {
    if let Err(e) = fs::create_dir(path) {   // If dir already exists, clear it
        fs::remove_dir_all(path).unwrap();
        fs::create_dir(path).unwrap();
    }

    fs::create_dir(path.join("frames")).unwrap()
}

fn main() {
    let opt = Opt::from_args();
    create_or_clear_output_location(&opt.output_path);

    let mut framerate_history: VecDeque<f32> = VecDeque::with_capacity(4);
    for i in 0..4 {
        framerate_history.push_back(0.0);
    }

    let mut mandel = Mandelbrot::new((opt.width, opt.height));
    mandel.set_focus(
        Vector2::new(opt.real_focus, opt.imaginary_focus)
    );
    mandel.set_zoom(0.5);

    let frames_location = opt.output_path.join("frames");

    let clock = Clock::start();
    let mut last_time = clock.elapsed_time();

    for i in 0..opt.frame_count {
        mandel.change_zoom_by(opt.zoom_speed);

        if i % 4 == 0 {
            mandel.max_iter = (mandel.max_iter as f64 * ((opt.zoom_speed - 1.0)/2.0 + 1.0)).floor() as usize;
        }

        let image = mandel.generate_image();
        image.save_to_file(frames_location.join(&format!("frame_{}.png", i)).to_str().unwrap());

        // Calculate framerate and other output
        let current_time = clock.elapsed_time();
        let frame_rate = 1.0/(current_time - last_time).as_seconds();
        last_time = current_time;

        framerate_history.push_back(frame_rate);
        framerate_history.pop_front();

        let mut total = 0.0;
        for f in framerate_history.iter() {
            total += f;
        }
        let average_framerate = total/4.0;

        let frame_count_left = opt.frame_count - i;
        let time_left = frame_count_left as f32 * average_framerate;

        println!(
            "Frame number: {} of {}, Frames per second: {:.2}, Time left (seconds): {:.2}",
            i,
            opt.frame_count,
            average_framerate,
            time_left
        );
    }

    println!("Done! Converting to video...");

    // Turn images into final video
    let output = Command::new("sh")
        .args(&[
            "-c",
            &format!(
                "ffmpeg -y -loglevel 24 -r {framerate} -s {width}x{height} -i {frames_loc}/frame_%d.png -crf 25 {output_video_loc}",
                framerate=opt.framerate,
                width=opt.width,
                height=opt.height,
                frames_loc = frames_location.to_str().unwrap(),
                output_video_loc = opt.output_path.join("output.mp4").to_str().unwrap()
            )
        ])
        .output()
        .expect("Failed to turn into video :(");

    println!(
        "ffmpeg warnings/errors: {}",
        String::from_utf8(output.stderr).unwrap()
    );
}
