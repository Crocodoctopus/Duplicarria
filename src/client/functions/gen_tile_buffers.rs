use crate::shared::tile::*;
use array2d::Array2D;
use ezgl::{gl::ARRAY_BUFFER, Buffer};

pub fn gen_tile_buffers(
    xyz: &mut Buffer<(f32, f32, f32)>,
    tex_uv: &mut Buffer<(f32, f32)>,
    msk_uv: &mut Buffer<(f32, f32)>,
    tiles_x: usize, // units in tiles
    tiles_y: usize, // units in tiles
    tiles: Array2D<Tile>,
) -> u32 {
    // Calculate onscreen tiles.
    let (tiles_w, tiles_h) = tiles.size();
    let tile_count = (tiles_w - 2) * (tiles_h - 2);

    // Fill vectors.
    let mut xyz_vec = Vec::<(f32, f32, f32)>::with_capacity(4 * tile_count);
    let mut tex_uv_vec = Vec::<(f32, f32)>::with_capacity(4 * tile_count);
    let mut msk_uv_vec = Vec::<(f32, f32)>::with_capacity(4 * tile_count);

    for y in 1..tiles_h - 1 {
        for x in 1..tiles_w - 1 {
            let id = *tiles.get(x, y).unwrap();

            // Get tile UV (skip None tiles).
            let (u, v) = match id {
                Tile::None => continue,
                Tile::Stone => (32, 0),
                Tile::Dirt => (16, 0),
            };

            // Convert tile ID to f32.
            let id = id as u8;

            // Caluclate xyz.
            let tile_x = ((x + tiles_x) * 16) as f32; // In pixels.
            let tile_y = ((y + tiles_y) * 16) as f32; // In pixels.
            xyz_vec.extend_from_slice(&[
                (tile_x - 7.5, tile_y - 7.5, id as f32),
                (tile_x + 24.5, tile_y - 7.5, id as f32),
                (tile_x + 24.5, tile_y + 24.5, id as f32),
                (tile_x - 7.5, tile_y + 24.5, id as f32),
            ]);

            // Calculate uv.
            let (u_f32, v_f32) = (u as f32, v as f32);
            tex_uv_vec.extend_from_slice(&[
                (u_f32 + 0.5, v_f32 + 0.5),
                (u_f32 + 15.5, v_f32 + 0.5),
                (u_f32 + 15.5, v_f32 + 15.5),
                (u_f32 + 0.5, v_f32 + 15.5),
            ]);

            // Calculate mask uv.
            let t = *tiles.get(x, y - 1).unwrap() as u8;
            let tr = *tiles.get(x + 1, y - 1).unwrap() as u8;
            let r = *tiles.get(x + 1, y).unwrap() as u8;
            let br = *tiles.get(x + 1, y + 1).unwrap() as u8;
            let b = *tiles.get(x, y + 1).unwrap() as u8;
            let bl = *tiles.get(x - 1, y + 1).unwrap() as u8;
            let l = *tiles.get(x - 1, y).unwrap() as u8;
            let tl = *tiles.get(x - 1, y - 1).unwrap() as u8;
            let mut mx = 0u8;
            mx |= ((t < id) as u8) << 0;
            mx |= ((tr < id) as u8) << 1;
            mx |= ((r < id) as u8) << 2;
            mx |= ((br < id) as u8) << 3;
            let mut my = 0u8;
            my |= ((b < id) as u8) << 0;
            my |= ((bl < id) as u8) << 1;
            my |= ((l < id) as u8) << 2;
            my |= ((tl < id) as u8) << 3;
            let mx = (mx << 2) as f32;
            let my = (my << 2) as f32;
            msk_uv_vec.extend_from_slice(&[
                (mx, my),
                (mx + 4., my),
                (mx + 4., my + 4.),
                (mx, my + 4.),
            ]);
        }
    }

    xyz.init(ARRAY_BUFFER, &xyz_vec[..]);
    tex_uv.init(ARRAY_BUFFER, &tex_uv_vec[..]);
    msk_uv.init(ARRAY_BUFFER, &msk_uv_vec[..]);

    return xyz_vec.len() as _;

    // Tile calculation
    /*let mut tile_xyz_vec = Vec::<(f32, f32, f32)>::with_capacity(4 * tile_count);
    let mut tile_tex_uv_vec = Vec::<(f32, f32)>::with_capacity(4 * tile_count);
    let mut tile_msk_uv_vec = Vec::<(f32, f32)>::with_capacity(4 * tile_count);
    let mut lens = Vec::with_capacity(4);
    for sy in 0..2 {
        for sx in 0..2 {
            let start = tile_xyz_vec.len();
            for y in (1..tiles_h - 1).skip(sy).step_by(2) {
                for x in (1..tiles_w - 1).skip(sx).step_by(2) {
                    // Get tile ID.
                    let id = *tiles.get(x, y).unwrap();

                    // Get tile UV (skip None tiles).
                    let (u, v) = match id {
                        Tile::None => continue,
                        Tile::Stone => (32, 0),
                        Tile::Dirt => (16, 0),
                    };

                    // Convert tile ID to u8.
                    let id = id as u8;

                    // Caluclate xyz.
                    let tx = x + tiles_x;
                    let ty = y + tiles_y;
                    tile_xyz_vec.extend_from_slice(&[
                        ((tx * 16) as f32 - 8.0, (ty * 16) as f32 - 8.0, id as f32),
                        ((tx * 16) as f32 + 24.0, (ty * 16) as f32 - 8.0, id as f32),
                        ((tx * 16) as f32 + 24.0, (ty * 16) as f32 + 24.0, id as f32),
                        ((tx * 16) as f32 - 8.0, (ty * 16) as f32 + 24.0, id as f32),
                    ]);

                    // Calculate uv.
                    let (u_f32, v_f32) = (u as f32, v as f32);
                    tile_tex_uv_vec.extend_from_slice(&[
                        (u_f32 + 0.5, v_f32 + 0.5),
                        (u_f32 + 15.5, v_f32 + 0.5),
                        (u_f32 + 15.5, v_f32 + 15.5),
                        (u_f32 + 0.5, v_f32 + 15.5),
                    ]);

                    // Calculate mask uv.
                    let t = *tiles.get(x, y - 1).unwrap() as u8;
                    let tr = *tiles.get(x + 1, y - 1).unwrap() as u8;
                    let r = *tiles.get(x + 1, y).unwrap() as u8;
                    let br = *tiles.get(x + 1, y + 1).unwrap() as u8;
                    let b = *tiles.get(x, y + 1).unwrap() as u8;
                    let bl = *tiles.get(x - 1, y + 1).unwrap() as u8;
                    let l = *tiles.get(x - 1, y).unwrap() as u8;
                    let tl = *tiles.get(x - 1, y - 1).unwrap() as u8;
                    let mut mx = 0u8;
                    mx |= ((t < id) as u8) << 0;
                    mx |= ((tr < id) as u8) << 1;
                    mx |= ((r < id) as u8) << 2;
                    mx |= ((br < id) as u8) << 3;
                    let mut my = 0u8;
                    my |= ((b < id) as u8) << 0;
                    my |= ((bl < id) as u8) << 1;
                    my |= ((l < id) as u8) << 2;
                    my |= ((tl < id) as u8) << 3;
                    let mx = (mx << 2) as f32;
                    let my = (my << 2) as f32;
                    tile_msk_uv_vec.extend_from_slice(&[
                        (mx, my),
                        (mx + 4., my),
                        (mx + 4., my + 4.),
                        (mx, my + 4.),
                    ]);
                }
            }

            // Calculate length.
            let end = tile_xyz_vec.len();
            //println!("{}", end - start);
            lens.push(end - start);
        }
    }*/
}
