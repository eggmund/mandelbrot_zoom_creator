use sfml::graphics::*;
use sfml::system::Vector2;

use nalgebra as na;
use na::Complex;

use palette::{Hsl, rgb::Rgb};

pub const ITER_COL_LIM: usize = 200;        // Iterations to repeat colour palette
pub const INITIAL_MAX_ITER: usize = 100;

const ESCAPE_RAD_SQR: f64 = 16.0;

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
            max_iter: INITIAL_MAX_ITER,
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

    pub fn image_coords_to_mandelbrot_coords(&self, p: Vector2<f64>) -> Complex<f64> {
        Complex::new(
            2.5 * (p.x - self.half_image_dims.0) / (self.zoom * self.image_dimensions.0 as f64) + self.offset.x,
            1.5 * (p.y - self.half_image_dims.1) / (self.zoom * self.image_dimensions.1 as f64) + self.offset.y,
        )
    }

    fn sfml_color_from_palette_color(rgb: Rgb) -> Color {
        Color::rgb((rgb.red * 256.0).floor() as u8, (rgb.blue * 256.0).floor() as u8, (rgb.green * 256.0).floor() as u8)
    }

    pub fn generate_image(&mut self) -> Image {
        let mut image = Image::new(self.image_dimensions.0, self.image_dimensions.1);

        for px in 0..self.image_dimensions.0 {
            for py in 0..self.image_dimensions.1 {
                let mand_coords = self.image_coords_to_mandelbrot_coords(Vector2::new(px as f64, py as f64));
                let (iters, mut z) = self.escape(mand_coords);

                if iters == self.max_iter {
                    image.set_pixel(px, py, &Color::rgb(0, 0, 0));
                } else {
                    // Iterate a few more times to get extra iters to smooth across.
                    z = complex_sqr(z) + mand_coords;
                    z = complex_sqr(z) + mand_coords;
                    let iter_limited = (iters + 2) % ITER_COL_LIM;

                    // Smoothing. See http://linas.org/art-gallery/escape/escape.html
                    let smooth_value = iter_limited as f32 - ((z.norm_sqr() as f32).sqrt().log10().log10()/2.0f32.log10());

                    let ratio = smooth_value as f32 / ITER_COL_LIM as f32;
                    let col_hue = Hsl::new(ratio * 360.0, 1.0, 0.5);
                    let sfml_color = Self::sfml_color_from_palette_color(Rgb::from(col_hue));

                    image.set_pixel(px, py, &sfml_color);
                }
            }
        }

        image
    }

    fn escape(&self, z0: Complex<f64>) -> (usize, Complex<f64>) {
        let mut iterations = 0usize;
        let mut z = z0.clone();

        while iterations < self.max_iter && z.norm_sqr() <= ESCAPE_RAD_SQR {
            z = complex_sqr(z) + z0;
            iterations += 1;
        }

        (iterations, z)
    }
}

#[inline]
fn complex_sqr(num: Complex<f64>) -> Complex<f64> {
    Complex::new(num.re * num.re - num.im * num.im, num.re * num.im * 2.0)
}
