use crate::game::lighting::*;
use array2d::Array2D;
use ezgl::{gl, Buffer, Texture2D};

pub fn gen_light_buffers(
    xy: &mut Buffer<(f32, f32)>,
    uv: &mut Buffer<(f32, f32)>,
    tex: &mut Texture2D,
    x: usize, // units in tiles
    y: usize, // units in tiles
    values: Array2D<u8>,
    rgb: (u8, u8, u8),
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
    let mut color = [rgb.0, rgb.1, rgb.2, 0];
    values.for_each(|_, _, &v| {
        /*let h: usize = (255 * 351 / 360);
        let s: usize = (255 * 92 / 100);
        let v: usize = ((255 * *v as usize) / MAX_BRIGHTNESS as usize);

        let region = (h / 34) & 0xFF;
        let remainder = ((h.wrapping_sub(region * 43)).wrapping_mul(6)) & 0xFF;
        let p = (v * (255 - s)) >> 8;
        let q = (v * (255 - ((s * remainder) >> 8))) >> 8;
        let t = (v * (255 - ((s * (255 - remainder)) >> 8))) >> 8;

        let (r, g, b) = match region {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        */

        let r = 0;
        let g = 0;
        let b = 0;
        let a = 255 - 255 / MAX_BRIGHTNESS * v;

        rgba.extend_from_slice(&[r, g, b, a]);

        //let a = (256. - 0.284 * (*v as f32 - 30.).powi(2)) as u8;
        //let a = 255 - ((255 * *v as usize) / MAX_BRIGHTNESS as usize) as u8;
        //let a = (256. - 256. * (0.85f32).powi((*v / 2) as _)) as u8;
        //let a = if *v == 0 { 100 } else { 200 };
    });
    tex.load_from_pixels(w as _, h as _, gl::RGBA, &rgba)
        .unwrap();
}
