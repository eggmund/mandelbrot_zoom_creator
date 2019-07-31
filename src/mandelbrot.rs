use sfml::graphics::*;
use sfml::system::Vector2;

use rug::{Assign, Complex, Float};

use palette::{rgb::Rgb, Hsl};

use crate::PRECISION;

pub const ITER_COL_LIM: usize = 200; // Iterations to repeat colour palette

const ESCAPE_RAD_SQR: f64 = 16.0;

#[derive(Clone)]
pub struct Mandelbrot {
    pub max_iter: usize,
    pub offset: Complex,
    pub zoom: Float,
}

impl Mandelbrot {
    pub fn new(max_iter: usize, offset: Complex, zoom: Float) -> Mandelbrot {
        Mandelbrot {
            max_iter,
            offset,
            zoom,
        }
    }

    #[inline]
    pub fn set_focus(&mut self, p: Complex) {
        // P is real and imaginary parts
        self.offset = p;
    }

    // #[inline]
    // pub fn change_focus_by(&mut self, diff: Vector2<f64>) {
    //     self.offset += diff;
    // }

    pub fn set_zoom(&mut self, zoom: Float) {
        self.zoom = zoom;
    }

    pub fn change_zoom_by(&mut self, dz: Float) {
        self.zoom *= dz;
    }

    // pub fn image_coords_to_mandelbrot_coords(
    //     &self,
    //     p: Vector2<u32>,
    //     image_dims: &(u32, u32),
    //     half_image_dims: &(Float, Float),
    // ) -> Complex {
    //     Complex::with_val(
    //         PRECISION,
    //         (
    //             2.5 * (Float::with_val(PRECISION, p.x) - &(half_image_dims.0))
    //                 / (&self.zoom * Float::with_val(PRECISION, image_dims.0 as f32))
    //                 + self.offset.real(),
    //             1.5 * (Float::with_val(PRECISION, p.y) - &(half_image_dims.1))
    //                 / (&self.zoom * Float::with_val(PRECISION, image_dims.1 as f32))
    //                 + self.offset.imag(),
    //         ),
    //     )
    // }

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
        let half_image_dims: (f64, f64) = (
            image_dims.0 as f64 / 2.0,
            image_dims.1 as f64 / 2.0,
        );

        let mut image = Image::new(image_dims.0, image_dims.1);

        // Increase per pixel (as mandelbrot complex num)
        let unit_increase = self.get_pixel_diff(&image_dims);

        for px in 0..image_dims.0 {
            for py in 0..image_dims.1 {
                let mand_coords = {
                    let z = Complex::with_val(
                        PRECISION,
                        (
                            unit_increase.real() * (px as f64 - half_image_dims.0),
                            unit_increase.imag() * (py as f64 - half_image_dims.1)
                        )
                    );
                    z + &self.offset
                };

                //println!("Mand coords: {}", mand_coords);

                let (iters, mut z) = self.escape(&mand_coords);

                if iters == self.max_iter {
                    image.set_pixel(px, py, &Color::rgb(0, 0, 0));
                } else {
                    // Iterate a few more times to get extra iters to smooth across.
                    z.square_mut();
                    z += &mand_coords;
                    z.square_mut();
                    z += &mand_coords;

                    let iter_limited = (iters + 2) % ITER_COL_LIM;

                    // Smoothing. See http://linas.org/art-gallery/escape/escape.html
                    let smooth_value = iter_limited as f32
                        - (z.norm().real().to_f32().log10().log10() / 2.0f32.log10());

                    let ratio = smooth_value as f32 / ITER_COL_LIM as f32;

                    image.set_pixel(px, py, &get_pixel_color(ratio));
                }
            }
        }

        image
    }

    #[inline]
    fn escape(&self, z0: &Complex) -> (usize, Complex) {
        let mut iterations = 0usize;
        let mut z = z0.clone();

        let mut z_norm = Float::with_val(PRECISION, z0.norm_ref());

        while iterations < self.max_iter && z_norm.to_f64() <= ESCAPE_RAD_SQR {
            z.square_mut();
            z += z0;

            z_norm.assign(z.norm_ref());

            iterations += 1;
        }

        (iterations, z)
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
