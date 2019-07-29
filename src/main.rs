#[macro_use]
extern crate structopt;

mod options;
mod mandelbrot;

use mandelbrot::Mandelbrot;
use options::Opt;

use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::system::{Clock, Vector2};

use structopt::StructOpt;

use nalgebra as na;

use std::io;
use std::process::Command;
use std::path::Path;
use std::fs;


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

    // let clock = Clock::start();
    // let mut last_time = clock.elapsed_time();

    let mut mandel = Mandelbrot::new((opt.width, opt.height));
    mandel.set_focus(
        // https://youtu.be/2VuLCEZMYPM
        Vector2::new(
            -1.7693831791955150182138472860854737829057472636547514374655282165278881912647564588361634463895296673044858257818203031574874912384217194031282461951137475212550721803797787274290,
            0.004236847918736772214926507171367997076682670917403757279459435650112344000805545157302430995023636506313532683359652571823004948055387363061275248149392923559310270429656787009248
        )
    );
    mandel.set_zoom(0.5);

    for i in 0..240 {
        mandel.change_zoom_by(1.05);
        let image = mandel.generate_image();
        image.save_to_file(opt.output_path.join(&format!("frames/frame_{}.png", i)).to_str().unwrap());
    }

    // Turn images into final video
    let output = Command::new("sh")
        .args(&[
            "-c",
            &format!(
                "ffmpeg -y -loglevel 24 -r {framerate} -s {width}x{height} -i ./output/frames/frame_%d.png -crf 25 ./output/video/output.mp4",
                framerate=opt.framerate,
                width=opt.width,
                height=opt.height
            )
        ])
        .output()
        .expect("Failed to turn into video :(");

    println!(
        "ffmpeg warnings/errors: {}",
        String::from_utf8(output.stderr).unwrap()
    );
}
