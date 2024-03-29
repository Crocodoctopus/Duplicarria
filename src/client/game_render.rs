use super::game_frame::GameFrame;
use crate::array2d::*;
use crate::game::item::*;
use crate::game::lighting::*;
use crate::game::tile::*;
use ezgl::gl;
use ezgl::{Buffer, Texture2D};
use std::collections::HashMap;

pub struct GameRender {
    textures: HashMap<&'static str, ezgl::Texture2D>,
    programs: HashMap<&'static str, ezgl::Program>,

    // General purpose IBO.
    ibo: Buffer<u16>,

    // Debug text data.
    debug_text_xy: Buffer<(f32, f32)>,
    debug_text_uv: Buffer<(f32, f32)>,

    // Tile state data.
    item_xy: Buffer<(f32, f32)>,
    item_uv: Buffer<(f32, f32)>,

    // Humanoid state data.
    humanoid_xy: Buffer<(f32, f32)>,
    humanoid_rgb: Buffer<(f32, f32, f32)>,

    // Tile state data.
    max_tiles: usize,
    tile_xyz: Buffer<(f32, f32, f32)>,
    tile_tex_uv: Buffer<(f32, f32)>,
    tile_msk_uv: Buffer<(f32, f32)>,

    // Texture state data.
    light_xy: Buffer<(f32, f32)>,
    light_uv: Buffer<(f32, f32)>,
    light_tex: ezgl::Texture2D,
}

impl GameRender {
    pub unsafe fn new() -> Self {
        // Prebuilt IBO for 11089 quads.
        let mut vec = Vec::with_capacity(66534);
        for i in 0..11089 {
            vec.extend_from_slice(&[4 * i, 4 * i + 1, 4 * i + 2, 4 * i + 2, 4 * i + 3, 4 * i]);
        }
        let ibo = Buffer::from(gl::ELEMENT_ARRAY_BUFFER, &vec);

        Self {
            textures: load_game_textures(),
            programs: load_game_programs(),

            ibo,

            debug_text_xy: Buffer::new(),
            debug_text_uv: Buffer::new(),

            item_xy: Buffer::new(),
            item_uv: Buffer::new(),

            humanoid_xy: Buffer::new(),
            humanoid_rgb: Buffer::new(),

            max_tiles: 0,
            tile_xyz: Buffer::new(),
            tile_tex_uv: Buffer::new(),
            tile_msk_uv: Buffer::new(),

            light_xy: Buffer::new(),
            light_uv: Buffer::new(),
            light_tex: ezgl::Texture2D::new(),
        }
    }

    pub unsafe fn render(&mut self, game_frame: &GameFrame) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // view calculation
        let view = {
            use cgmath::*;
            let (x, y, w, h) = (
                game_frame.view_x as f32,
                game_frame.view_y as f32,
                game_frame.view_w as f32,
                game_frame.view_h as f32,
            );
            let mut matrix = Matrix3::identity();
            matrix = matrix * Matrix3::from_nonuniform_scale(2. / w, -2. / h);
            matrix = matrix * Matrix3::from_translation(Vector2::new(-w / 2. - x, -h / 2. - y));
            matrix
        };

        // Fill bg tile buffers with data
        let tile_count = gen_tile_buffers(
            &mut self.tile_xyz,
            &mut self.tile_tex_uv,
            &mut self.tile_msk_uv,
            game_frame.tiles_x,
            game_frame.tiles_y,
            &game_frame.background_tiles,
        );

        // Render tiles.
        ezgl::Draw::start_tri_draw(tile_count as u32 / 2, &self.programs["bg_tile"], &self.ibo)
            .with_buffer(&self.tile_xyz, "vert_tile_xyz")
            .with_buffer(&self.tile_tex_uv, "vert_tile_uv")
            .with_buffer(&self.tile_msk_uv, "vert_mask_uv")
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], "view_matrix")
            .with_texture(&self.textures["tile_sheet.png"], "tile_sheet")
            .with_texture(&self.textures["mask_sheet.png"], "mask_sheet")
            .draw();

        // Generate humanoid buffer data
        let humanoid_count = gen_humanoid_buffers(
            &mut self.humanoid_xy,
            &mut self.humanoid_rgb,
            &game_frame.humanoid_positions,
        );

        // Render humanoids
        ezgl::Draw::start_tri_draw(humanoid_count as u32 / 2, &self.programs["quad"], &self.ibo)
            .with_buffer(&self.humanoid_xy, "vert_xy")
            .with_buffer(&self.humanoid_rgb, "vert_rgb")
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], "view_matrix")
            .draw();

        // Item rendering.
        {
            fn gen_item_buffers(
                xy: &mut Buffer<(f32, f32)>,
                uv: &mut Buffer<(f32, f32)>,
                items: &Vec<(f32, f32, ItemId)>,
            ) -> u32 {
                let mut xy_vec = Vec::with_capacity(4 * items.len());
                let mut uv_vec = Vec::with_capacity(4 * items.len());
                for (x, y, item_id) in items {
                    xy_vec.extend_from_slice(&[
                        (*x, *y),
                        (*x + 16., *y),
                        (*x + 16., *y + 16.),
                        (*x, *y + 16.),
                    ]);
                    let (u, v) = match item_id {
                        ItemId::Dirt => (16., 0.),
                        ItemId::Stone => (32., 0.),
                    };
                    uv_vec.extend_from_slice(&[
                        (u, v),
                        (u + 16., v),
                        (u + 16., v + 16.),
                        (u, v + 16.),
                    ]);
                }
                xy.init(gl::ARRAY_BUFFER, &xy_vec).unwrap();
                uv.init(gl::ARRAY_BUFFER, &uv_vec).unwrap();
                (xy_vec.len() / 4) as u32
            }
            let item_count =
                gen_item_buffers(&mut self.item_xy, &mut self.item_uv, &game_frame.items);

            ezgl::Draw::start_tri_draw(item_count * 2, &self.programs["quad"], &self.ibo)
                .with_buffer(&self.item_xy, "vert_xy")
                .with_buffer(&self.item_uv, "vert_uv")
                .with_uniform(view.as_ref() as &[[f32; 3]; 3], "view_matrix")
            .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
                .with_texture(&self.textures["tile_sheet.png"], "tex")
                .draw();
        }

        // Fill fg tile buffers with data
        let tile_count = gen_tile_buffers(
            &mut self.tile_xyz,
            &mut self.tile_tex_uv,
            &mut self.tile_msk_uv,
            game_frame.tiles_x,
            game_frame.tiles_y,
            &game_frame.foreground_tiles,
        );

        // Render tiles.
        ezgl::Draw::start_tri_draw(tile_count as u32 / 2, &self.programs["fg_tile"], &self.ibo)
            .with_buffer(&self.tile_xyz, "vert_tile_xyz")
            .with_buffer(&self.tile_tex_uv, "vert_tile_uv")
            .with_buffer(&self.tile_msk_uv, "vert_mask_uv")
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], "view_matrix")
            .with_texture(&self.textures["tile_sheet.png"], "tile_sheet")
            .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
            .with_texture(&self.textures["mask_sheet.png"], "mask_sheet")
            .draw();

        // Fill light buffers with data.
        gen_light_buffers(
            &mut self.light_xy,
            &mut self.light_uv,
            &mut self.light_tex,
            game_frame.light_x,
            game_frame.light_y,
            &game_frame.light_map_r,
            (255, 0, 0),
        );

        // Render light map.
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        }
        ezgl::Draw::start_tri_draw(2, &self.programs["light"], &self.ibo)
            .with_buffer(&self.light_xy, "vert_xy")
            .with_buffer(&self.light_uv, "vert_uv")
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], "view_matrix")
            .with_texture(&self.light_tex, "light_map")
            .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
            .draw();

        // File debug text buffers with data.
        let char_count = gen_debug_text_buffers(
            &mut self.debug_text_xy,
            &mut self.debug_text_uv,
            &game_frame.debug_text,
            (game_frame.view_x as f32, game_frame.view_y as f32),
        );

        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        }
        ezgl::Draw::start_tri_draw(char_count as u32 / 2, &self.programs["quad"], &self.ibo)
            .with_buffer(&self.debug_text_xy, "vert_xy")
            .with_buffer(&self.debug_text_uv, "vert_uv")
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], "view_matrix")
            .with_texture(&self.textures["debug_font.png"], "tex")
            .enable_blend(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA)
            .draw();
    }
}

fn load_game_textures() -> HashMap<&'static str, ezgl::Texture2D> {
    let root = crate::io::get_root().join("resources");
    let load_list = ["debug_font.png", "tile_sheet.png", "mask_sheet.png"];
    let mut hmap = HashMap::new();

    for string in load_list {
        let mut texture = ezgl::Texture2D::new();
        texture.load_from_file(&root.join(string)).unwrap();
        hmap.insert(string, texture);
    }

    hmap
}

fn load_game_programs() -> HashMap<&'static str, ezgl::Program> {
    let root = crate::io::get_root().join("resources");
    let load_list = ["fg_tile", "bg_tile", "light", "quad"];
    let mut hmap = HashMap::new();

    for string in load_list {
        hmap.insert(
            string,
            ezgl::ProgramBuilder::new()
                .with(ezgl::Shader::from_file(&root.join(format!("{}.frag", string))).unwrap())
                .with(ezgl::Shader::from_file(&root.join(format!("{}.vert", string))).unwrap())
                .build()
                .unwrap(),
        );
    }

    hmap
}

pub fn gen_tile_buffers(
    xyz: &mut Buffer<(f32, f32, f32)>,
    tex_uv: &mut Buffer<(f32, f32)>,
    msk_uv: &mut Buffer<(f32, f32)>,
    tiles_x: usize, // units in tiles
    tiles_y: usize, // units in tiles
    tiles: &Array2D<Tile>,
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

    xyz.init(gl::ARRAY_BUFFER, &xyz_vec[..]).unwrap();
    tex_uv.init(gl::ARRAY_BUFFER, &tex_uv_vec[..]).unwrap();
    msk_uv.init(gl::ARRAY_BUFFER, &msk_uv_vec[..]).unwrap();

    return xyz_vec.len() as _;
}

pub fn gen_light_buffers(
    xy: &mut Buffer<(f32, f32)>,
    uv: &mut Buffer<(f32, f32)>,
    tex: &mut Texture2D,
    x: usize, // units in tiles
    y: usize, // units in tiles
    values: &Array2D<u8>,
    _rgb: (u8, u8, u8),
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
    )
    .unwrap();

    // Generate light uv position.
    uv.init(
        gl::ARRAY_BUFFER,
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
    )
    .unwrap();

    // Generate texture
    let mut rgba: Vec<u8> = Vec::with_capacity(4 * w * h);
    //let mut color = [rgb.0, rgb.1, rgb.2, 0];
    values.for_each(|_, _, &v| {
        let r = 0;
        let g = 0;
        let b = 0;
        let a = 255 - 255 / MAX_BRIGHTNESS * v;
        rgba.extend_from_slice(&[r, g, b, a]);
    });
    tex.load_from_pixels(w as _, h as _, gl::RGBA, &rgba)
        .unwrap();
}

fn gen_humanoid_buffers(
    xy: &mut Buffer<(f32, f32)>,
    rgb: &mut Buffer<(f32, f32, f32)>,
    positions: &Vec<(f32, f32)>,
) -> usize {
    let len = positions.len();
    let mut xy_vec = Vec::with_capacity(len * 4);
    let mut rgb_vec = Vec::with_capacity(len * 4);
    let red = (1.0, 0.0, 0.0);
    const HUMANOID_WIDTH: f32 = crate::game::humanoid::HUMANOID_WIDTH as f32;
    const HUMANOID_HEIGHT: f32 = crate::game::humanoid::HUMANOID_HEIGHT as f32;
    for &(x, y) in positions {
        xy_vec.extend_from_slice(&[
            (x, y),
            (x + HUMANOID_WIDTH, y),
            (x + HUMANOID_WIDTH, y + HUMANOID_HEIGHT),
            (x, y + HUMANOID_HEIGHT),
        ]);
        rgb_vec.extend_from_slice(&[red, red, red, red]);
    }
    xy.init(gl::ARRAY_BUFFER, &xy_vec[..]).unwrap();
    rgb.init(gl::ARRAY_BUFFER, &rgb_vec[..]).unwrap();
    len * 4
}

fn gen_debug_text_buffers(
    xy: &mut Buffer<(f32, f32)>,
    uv: &mut Buffer<(f32, f32)>,
    string: &String,
    offset: (f32, f32),
) -> usize {
    // The width and height of each character.
    let w = 8.;
    let h = 14.;
    let tex_w = 10; // in chars
    let _tex_h = 10; // in chars

    // Fill xy and uv vecs.
    let mut x = offset.0;
    let mut y = offset.1;
    let mut xy_vec = Vec::with_capacity(string.len());
    let mut uv_vec = Vec::with_capacity(string.len());
    for c in string.chars() {
        // Special logic on newline.
        if c == '\n' {
            x = offset.0;
            y += h;
            continue;
        }

        // Insert vertices.
        xy_vec.extend_from_slice(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)]);

        // Insert uv.
        let index = c as u8 - 0x20;
        let u = (index % tex_w) as f32 * w;
        let v = (index / tex_w) as f32 * h;
        uv_vec.extend_from_slice(&[(u, v), (u + w, v), (u + w, v + h), (u, v + h)]);

        // Increment x.
        x += w;
    }

    // Fill
    xy.init(gl::ARRAY_BUFFER, &xy_vec).unwrap();
    uv.init(gl::ARRAY_BUFFER, &uv_vec).unwrap();

    xy_vec.len()
}
