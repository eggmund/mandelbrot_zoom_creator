use sfml::graphics::*;
use sfml::system::Vector2;

use na::Complex;
use nalgebra as na;

use crate::{HALF_IMAGE_DIMS, IMAGE_DIMS};


pub struct Mandelbrot {
    max_iter: usize,
    offset: Vector2<f64>,
    pub zoom: f64,
}

impl Mandelbrot {
    pub fn new() -> Mandelbrot {
        Mandelbrot {
            max_iter: 100,
            offset: Vector2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }

    pub fn set_focus(&mut self, p: Vector2<f64>) {
        // P is real and imaginary parts
        self.offset = p;
    }

    pub fn change_focus_by(&mut self, diff: Vector2<f64>) {
        self.offset += diff;
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom;
    }

    pub fn change_zoom_by(&mut self, dz: f64) {
        if self.zoom * dz > 0.0 {
            self.zoom *= dz;
        }
    }

    pub fn mandelbrot_coords_to_screen_coords(&self, p: Complex<f64>) -> Vector2<f64> {
        Vector2::new(
            ((p.re + HALF_IMAGE_DIMS.0) * (self.zoom / IMAGE_DIMS.0 as f64)) / 2.5 - self.offset.x,
            ((p.im + HALF_IMAGE_DIMS.1) * (self.zoom / IMAGE_DIMS.1 as f64)) / 1.5 - self.offset.y,
        )
    }

    pub fn screen_coords_to_mandelbrot_coords(&self, p: Vector2<f64>) -> Complex<f64> {
        Complex::new(
            2.5 * (p.x - HALF_IMAGE_DIMS.0) / (self.zoom * IMAGE_DIMS.0 as f64) + self.offset.x,
            1.5 * (p.y - HALF_IMAGE_DIMS.1) / (self.zoom * IMAGE_DIMS.1 as f64) + self.offset.y,
        )
    }

    pub fn generate_image(&mut self) -> Image {
        let mut image = Image::new(IMAGE_DIMS.0, IMAGE_DIMS.1);

        for px in 0..IMAGE_DIMS.0 {
            for py in 0..IMAGE_DIMS.1 {
                let iters = self.escape(
                    self.screen_coords_to_mandelbrot_coords(Vector2::new(px as f64, py as f64)),
                );

                let ratio = iters as f64 / self.max_iter as f64;
                let ratio_256 = (ratio * 256.0).floor() as u8;
                let col = Color::rgb(ratio_256, ratio_256, ratio_256);

                image.set_pixel(px, py, &col);
            }
        }

        image
    }

    fn escape(&self, z0: Complex<f64>) -> usize {
        let mut iterations = 0usize;
        let mut z = z0.clone();

        while iterations < self.max_iter && z.norm_sqr() <= 4.0 {
            z = complex_sqr(z) + z0;
            iterations += 1;
        }

        iterations
    }
}

fn complex_sqr(num: Complex<f64>) -> Complex<f64> {
    Complex::new(num.re * num.re - num.im * num.im, num.re * num.im * 2.0)
}
