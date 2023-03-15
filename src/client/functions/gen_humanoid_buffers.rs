use ezgl::{gl, Buffer};

pub fn gen_humanoid_buffers(
    xy: &mut Buffer<(f32, f32)>,
    rgb: &mut Buffer<(f32, f32, f32)>,
    pos: &Vec<(f32, f32)>,
) {
    let mut xy_data = Vec::with_capacity(pos.len() * 4);
    let mut rgb_data = Vec::with_capacity(pos.len() * 4);
    const HUMANOID_WIDTH: f32 = 16.;
    const HUMANOID_HEIGHT: f32 = 24.;
    for &(x, y) in pos {
        xy_data.extend_from_slice(&[
            (x, y),
            (x + HUMANOID_WIDTH, y),
            (x + HUMANOID_WIDTH, y + HUMANOID_HEIGHT),
            (x, y + HUMANOID_HEIGHT),
        ]);
        rgb_data.extend_from_slice(&[
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
        ]);
    }

    xy.init(gl::ARRAY_BUFFER, &xy_data[..]);
    rgb.init(gl::ARRAY_BUFFER, &rgb_data[..]);
}
