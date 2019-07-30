mod mandelbrot;
mod options;

use mandelbrot::Mandelbrot;
use options::Opt;

use sfml::system::{Clock, Vector2};

use structopt::StructOpt;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread;

fn create_or_clear_output_location(path: &Path) {
    if let Err(_) = fs::create_dir(path) {
        // If dir already exists, clear it
        fs::remove_dir_all(path).unwrap();
        fs::create_dir(path).unwrap();
    }

    fs::create_dir(path.join("frames")).unwrap()
}

#[inline]
fn max_iterations_increase_formula(mut max_iter: usize, zoom_speed: f64, index: usize) -> usize {  // Returns new max iter value
    for _ in 0..index {
        max_iter = (max_iter as f64 * ((zoom_speed - 1.0)/100.0 + 1.0)).floor() as usize
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

    // let mut framerate_history: VecDeque<f32> = VecDeque::with_capacity(4);
    // for _ in 0..4 {
    //     framerate_history.push_back(0.0);
    // }

    let num_cpu = if opt.cpu_num_to_use != 0 {
        opt.cpu_num_to_use
    } else {
        num_cpus::get()
    };
    let mandel: Arc<RwLock<Mandelbrot>> = Arc::new(RwLock::new(Mandelbrot::new()));

    {
        let mut mandel_write = mandel.try_write().unwrap();
        mandel_write.set_focus(Vector2::new(opt.real_focus, opt.imaginary_focus));
        mandel_write.set_zoom(0.5);
    }

    let frames_location = Arc::new(opt.output_path.join("frames"));

    // let clock = Clock::start();
    // let mut last_time = clock.elapsed_time();

    for i in 0..(opt.frame_count as f32 / num_cpu as f32).ceil() as usize {
        let mut child_threads = Vec::with_capacity(num_cpu);

        for j in 0..num_cpu {
            let frames_location = Arc::clone(&frames_location);
            let opt = Arc::clone(&opt);
            let mandel = Arc::clone(&mandel);

            child_threads.push(thread::spawn(move || {
                let mut this_mandel = Mandelbrot::new();
                {
                    // Copy stuff from parent mandel
                    let parent_mandel = mandel.try_read().unwrap();
                    this_mandel.zoom = parent_mandel.zoom;
                    this_mandel.offset = parent_mandel.offset;
                    this_mandel.max_iter = parent_mandel.max_iter;
                }
                let my_zoom_speed = opt.zoom_speed.powi(j as i32);
                this_mandel.max_iter = max_iterations_increase_formula(this_mandel.max_iter, opt.zoom_speed, j);
                this_mandel.zoom *= my_zoom_speed;

                let image = this_mandel.generate_image(dims);
                let save_loc = frames_location
                    .join(&format!("frame_{}.png", (i * num_cpu) + j));
            
                image.save_to_file(save_loc.to_str().unwrap());

                println!("Saved image to: {:#?}", save_loc);
            }));
        }

        // Wait for each to finish
        for child in child_threads {
            child.join().unwrap();
        }

        // Increase mandelbrot values for base mandelbrot data, increasing zoom by a factor of num_core
        {
            let mut mandel_write = mandel.try_write().unwrap();
            let overall_zoom_speed = opt.zoom_speed.powi(num_cpu as i32);
            mandel_write.change_zoom_by(overall_zoom_speed);
            mandel_write.max_iter = max_iterations_increase_formula(mandel_write.max_iter, overall_zoom_speed, num_cpu);
            println!("Iterations: {}", mandel_write.max_iter);
        }

        // // Calculate framerate and other output
        // let current_time = clock.elapsed_time();
        // let frame_rate = 1.0/(current_time - last_time).as_seconds();
        // last_time = current_time;

        // framerate_history.push_back(frame_rate);
        // framerate_history.pop_front();

        // let mut total = 0.0;
        // for f in framerate_history.iter() {
        //     total += f;
        // }
        // let average_framerate = total/4.0;

        // let frame_count_left = opt.frame_count - i;
        // let time_left = frame_count_left as f32/average_framerate;

        // println!(
        //     "Frame number: {} of {}, Frames per second: {:.2}, Time left (seconds): {:.2}, Max iterations: {}",
        //     i,
        //     opt.frame_count,
        //     average_framerate,
        //     time_left,
        //     mandel.max_iter
        // );
    }

    println!("Done! Converting to video...");

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