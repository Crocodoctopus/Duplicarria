use array2d::Array2D;
use ezgl::{gl, Buffer, Texture2D};
use crate::shared::*;

pub fn gen_light_buffers(
    xy: &mut Buffer<(f32, f32)>,
    uv: &mut Buffer<(f32, f32)>,
    tex: &mut Texture2D,
    x: usize, // units in tiles
    y: usize, // units in tiles
    values: Array2D<u8>,
) {
    let (w, h) = values.size();

    let x_px = (x * 16) as f32;
    let y_px = (y * 16) as f32;
    let w_px = (w * 16) as f32;
    let h_px = (h * 16) as f32;

    // Generate light quad position.
    xy.init(
        gl::ARRAY_BUFFER,
        &[
            (x_px, y_px),
            (x_px + w_px, y_px),
            (x_px + w_px, y_px + h_px),
            (x_px, y_px + h_px),
        ],
    );

    // Generate light uv position.
    uv.init(
        gl::ARRAY_BUFFER,
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
    );

    // Generate texture
    let mut rgba: Vec<u8> = Vec::with_capacity(4 * w * h);
    values.for_each(|x, y, v| {
        //let a = (256. - 0.284 * (*v as f32 - 30.).powi(2)) as u8;
        let mut a = 0;
        if *v == MIN_BRIGHTNESS {
            a = 255;
        }
        //let a = (256. - 256. * (0.85f32).powi((*v / 2) as _)) as u8;
        //let a = if *v == 0 { 100 } else { 200 };
        rgba.extend_from_slice(&[0, 0, 0, a]);
    });
    tex.load_from_pixels(w as _, h as _, gl::RGBA, &rgba)
        .unwrap();
}
