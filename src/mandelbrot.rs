use sfml::graphics::*;
use sfml::system::Vector2;

use rug::{Assign, Complex, Float};

use palette::{rgb::Rgb, Hsl};

use std::boxed::Box;
use std::sync::Arc;

use crate::{
    threadpool::{Job, JobResult, ThreadPool},
    PRECISION
};

pub const ITER_COL_LIM: usize = 200; // Iterations to repeat colour palette

const ESCAPE_RAD_SQR: f64 = 16.0;

pub struct Mandelbrot {
    pub max_iter: usize,
    pub offset: Complex,
    pub zoom: Float,
    pool: ThreadPool,
}

impl Mandelbrot {
    pub fn new(max_iter: usize, offset: Complex, zoom: Float, num_cpu: usize) -> Mandelbrot {
        Mandelbrot {
            max_iter,
            offset,
            zoom,
            pool: ThreadPool::new(num_cpu),
        }
    }

    // #[inline]
    // pub fn set_focus(&mut self, p: Complex) {
    //     // P is real and imaginary parts
    //     self.offset = p;
    // }

    // #[inline]
    // pub fn change_focus_by(&mut self, diff: Vector2<f64>) {
    //     self.offset += diff;
    // }

    // pub fn set_zoom(&mut self, zoom: Float) {
    //     self.zoom = zoom;
    // }

    pub fn change_zoom_by(&mut self, dz: f64) {
        self.zoom *= dz;
    }

    pub fn image_coords_to_mandelbrot_coords(
        &self,
        p: Vector2<u32>,
        image_dims: &(u32, u32),
        half_image_dims: &(f64, f64),
    ) -> Complex {
        Complex::with_val(
            PRECISION,
            (
                2.5 * Float::with_val(PRECISION, p.x as f64 - half_image_dims.0)
                    / (&self.zoom * Float::with_val(PRECISION, image_dims.0 as f32))
                    + self.offset.real(),
                1.5 * Float::with_val(PRECISION, p.y as f64 - half_image_dims.1)
                    / (&self.zoom * Float::with_val(PRECISION, image_dims.1 as f32))
                    + self.offset.imag(),
            )
        )
    }

    // Difference in offset between each pixel
    fn get_pixel_diff(&self, image_dims: &(u32, u32)) -> Complex {
        Complex::with_val(
            PRECISION,
            (
                2.5 * Float::with_val(PRECISION, 1.0)
                    / (&self.zoom * Float::with_val(PRECISION, image_dims.0 as f32)),
                1.5 * Float::with_val(PRECISION, 1.0)
                    / (&self.zoom * Float::with_val(PRECISION, image_dims.1 as f32)),
            ),
        )
    }

    pub fn generate_image(&self, image_dims: (u32, u32)) -> Image {
        let half_image_dims: (f64, f64) = (image_dims.0 as f64 / 2.0, image_dims.1 as f64 / 2.0);

        let mut image = Image::new(image_dims.0, image_dims.1);

        // Increase per pixel (as mandelbrot complex num)
        let unit_increase = Box::new(self.get_pixel_diff(&image_dims));
        // Increase of real unit.
        let dr = Arc::new(Box::new(Float::with_val(PRECISION, unit_increase.real())));
        // Left of image, each row is done seperately, so this is the left most mandelbrot coord of the image, starting at the top left,
        // and will move down (real will decrease) by 1 pixel vertically each iteration.
        let mut left = self.image_coords_to_mandelbrot_coords(
            Vector2::new(0, 0),
            &image_dims,
            &half_image_dims,
        );

        for py in 0..image_dims.1 {
            let dr = Arc::clone(&dr);

            self.pool.send_job(Job {
                row: py,
                image_width: image_dims.0,
                z_start: Box::new(left.clone()),
                z_real_increase: dr,
                max_iter: self.max_iter,
            });
            *left.mut_imag() += unit_increase.imag();
        }

        let mut results: Vec<JobResult> = Vec::with_capacity(image_dims.1 as usize);
        for _ in 0..image_dims.1 {
            let result: JobResult = self.pool.get_job_result().unwrap();
            results.push(result);
        }

        for res in results.iter() {
            for (i, ratio) in res.ratios.iter().enumerate() {
                // If it is in the set, it will be None, so set colour to black.
                match ratio {
                    Some(ratio) => {
                        image.set_pixel(i as u32, res.row, &get_pixel_color(*ratio))
                    },
                    None => image.set_pixel(i as u32, res.row, &Color::rgb(0, 0, 0))
                };
            }            
        }

        image
    }
}

fn get_pixel_color(ratio: f32) -> Color {
    let col_hue = Hsl::new(ratio * 360.0, 1.0, 0.5);
    sfml_color_from_palette_color(Rgb::from(col_hue))
}

#[inline]
fn sfml_color_from_palette_color(rgb: Rgb) -> Color {
    Color::rgb(
        (rgb.red * 256.0).floor() as u8,
        (rgb.blue * 256.0).floor() as u8,
        (rgb.green * 256.0).floor() as u8,
    )
}

#[inline]
pub fn escape_and_get_ratio(max_iter: usize, z0: &Complex) -> Option<f32> { // Returns None if in set
    let mut iterations = 0usize;
    let mut z = z0.clone();
    let mut z_norm = Float::with_val(PRECISION, z0.norm_ref());

    while iterations < max_iter && z_norm.to_f64() <= ESCAPE_RAD_SQR {
        z.square_mut();
        z += z0;

        z_norm.assign(z.norm_ref());

        iterations += 1;
    }

    if iterations == max_iter {
        None
    } else {
        // Iterate a few more times to get extra iters to smooth across.
        z.square_mut();
        z += z0;
        z.square_mut();
        z += z0;

        let iter_limited = (iterations + 2) % ITER_COL_LIM;

        // Smoothing. See http://linas.org/art-gallery/escape/escape.html
        let smooth_value =
            iter_limited as f32 - (z.norm().real().to_f32().log10().log10() / 2.0f32.log10());

        Some(smooth_value as f32 / ITER_COL_LIM as f32)
    }
}
