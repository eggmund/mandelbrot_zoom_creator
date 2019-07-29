use sfml::graphics::*;
use sfml::system::Vector2;

use na::Complex;
use nalgebra as na;

pub const MAX_ITER_COL_LIM: usize = 100;

pub struct Mandelbrot {
    pub image_dimensions: (u32, u32),
    pub half_image_dims: (f64, f64),
    pub max_iter: usize,
    offset: Vector2<f64>,
    pub zoom: f64,
}

impl Mandelbrot {
    pub fn new(image_dimensions: (u32, u32)) -> Mandelbrot {
        Mandelbrot {
            image_dimensions,
            half_image_dims: (image_dimensions.0 as f64/2.0, image_dimensions.1 as f64/2.0),
            max_iter: MAX_ITER_COL_LIM,
            offset: Vector2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }

    #[inline]
    pub fn set_focus(&mut self, p: Vector2<f64>) {
        // P is real and imaginary parts
        self.offset = p;
    }

    #[inline]
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
            ((p.re + self.half_image_dims.0) * (self.zoom / self.image_dimensions.0 as f64)) / 2.5 - self.offset.x,
            ((p.im + self.half_image_dims.1) * (self.zoom / self.image_dimensions.1 as f64)) / 1.5 - self.offset.y,
        )
    }

    pub fn screen_coords_to_mandelbrot_coords(&self, p: Vector2<f64>) -> Complex<f64> {
        Complex::new(
            2.5 * (p.x - self.half_image_dims.0) / (self.zoom * self.image_dimensions.0 as f64) + self.offset.x,
            1.5 * (p.y - self.half_image_dims.1) / (self.zoom * self.image_dimensions.1 as f64) + self.offset.y,
        )
    }

    pub fn generate_image(&mut self) -> Image {
        let mut image = Image::new(self.image_dimensions.0, self.image_dimensions.1);

        for px in 0..self.image_dimensions.0 {
            for py in 0..self.image_dimensions.1 {
                let iters = self.escape(
                    self.screen_coords_to_mandelbrot_coords(Vector2::new(px as f64, py as f64)),
                );

                if iters == self.max_iter {
                    image.set_pixel(px, py, &Color::rgb(0.0, 0.0, 0.0));
                } else {
                    let iter_limited = iters % MAX_ITER_COL_LIM;

                    let ratio = iter_limited as f64 / self.max_iter as f64;
                    let ratio_256 = (ratio * 256.0).floor() as u8;
                    let col = Color::rgb(ratio_256, ratio_256, ratio_256);

                    image.set_pixel(px, py, &col);
                }
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

#[inline]
fn complex_sqr(num: Complex<f64>) -> Complex<f64> {
    Complex::new(num.re * num.re - num.im * num.im, num.re * num.im * 2.0)
}
