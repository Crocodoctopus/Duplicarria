use super::render_frame::RenderFrame;
use std::collections::HashMap;

fn load_textures() -> HashMap<String, ezgl::Texture2D> {
    let root = crate::io::get_root().join("resources");
    let load_list = ["tile_sheet.png", "mask_sheet.png"];
    let mut hmap = HashMap::new();

    for string in load_list {
        let mut texture = ezgl::Texture2D::new();
        texture.load_from_file(&root.join(string)).unwrap();
        hmap.insert(string.to_owned(), texture);
    }

    hmap
}

fn load_programs() -> HashMap<String, ezgl::Program> {
    let root = crate::io::get_root().join("resources");
    let mut hmap = HashMap::new();

    let frag = ezgl::Shader::from_file(&root.join("fg_tile.frag")).unwrap();
    let vert = ezgl::Shader::from_file(&root.join("fg_tile.vert")).unwrap();
    let program = ezgl::ProgramBuilder::new()
        .with(frag)
        .with(vert)
        .build()
        .unwrap();
    hmap.insert(String::from("fg_tile"), program);

    let frag = ezgl::Shader::from_file(&root.join("bg_tile.frag")).unwrap();
    let vert = ezgl::Shader::from_file(&root.join("bg_tile.vert")).unwrap();
    let program = ezgl::ProgramBuilder::new()
        .with(frag)
        .with(vert)
        .build()
        .unwrap();
    hmap.insert(String::from("bg_tile"), program);

    let frag = ezgl::Shader::from_file(&root.join("light.frag")).unwrap();
    let vert = ezgl::Shader::from_file(&root.join("light.vert")).unwrap();
    let program = ezgl::ProgramBuilder::new()
        .with(frag)
        .with(vert)
        .build()
        .unwrap();
    hmap.insert(String::from("light"), program);

    hmap
}

pub struct RenderState {
    textures: HashMap<String, ezgl::Texture2D>,
    programs: HashMap<String, ezgl::Program>,

    // General purpose IBO.
    ibo: ezgl::Buffer<u16>,

    // Tile state data.
    max_tiles: usize,
    tile_xyz: ezgl::Buffer<(f32, f32, f32)>,
    tile_tex_uv: ezgl::Buffer<(f32, f32)>,
    tile_msk_uv: ezgl::Buffer<(f32, f32)>,

    // Texture state data.
    light_xy: ezgl::Buffer<(f32, f32)>,
    light_uv: ezgl::Buffer<(f32, f32)>,
    light_tex: ezgl::Texture2D,
}

impl RenderState {
    pub fn new() -> Self {
        // Prebuilt IBO for 11089 quads.
        let mut vec = Vec::with_capacity(66534);
        for i in 0..11089 {
            vec.extend_from_slice(&[4 * i, 4 * i + 1, 4 * i + 2, 4 * i + 2, 4 * i + 3, 4 * i]);
        }
        let ibo = ezgl::Buffer::from(ezgl::gl::ELEMENT_ARRAY_BUFFER, &vec);

        Self {
            textures: load_textures(),
            programs: load_programs(),

            ibo,

            max_tiles: 0,
            tile_xyz: ezgl::Buffer::new(),
            tile_tex_uv: ezgl::Buffer::new(),
            tile_msk_uv: ezgl::Buffer::new(),

            light_xy: ezgl::Buffer::new(),
            light_uv: ezgl::Buffer::new(),
            light_tex: ezgl::Texture2D::new(),
        }
    }

    pub fn render(&mut self, render_frame: RenderFrame) {
        // view calculation
        let view = {
            use cgmath::*;
            let (x, y, w, h) = (
                render_frame.view_x as f32,
                render_frame.view_y as f32,
                render_frame.view_w as f32,
                render_frame.view_h as f32,
            );
            let mut matrix = Matrix3::identity();
            matrix = matrix * Matrix3::from_nonuniform_scale(2. / w, -2. / h);
            matrix = matrix * Matrix3::from_translation(Vector2::new(-w / 2. - x, -h / 2. - y));
            matrix
        };

        // Fill bg tile buffers with data
        let tile_count = super::functions::gen_tile_buffers(
            &mut self.tile_xyz,
            &mut self.tile_tex_uv,
            &mut self.tile_msk_uv,
            render_frame.tiles_x,
            render_frame.tiles_y,
            render_frame.background_tiles,
        );

        // Render tiles.
        ezgl::Draw::start_tri_draw(tile_count as u32 / 2, &self.programs["bg_tile"], &self.ibo)
            .with_buffer(&self.tile_xyz, 0)
            .with_buffer(&self.tile_tex_uv, 1)
            .with_buffer(&self.tile_msk_uv, 2)
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], 0)
            .with_texture(&self.textures["tile_sheet.png"], 1)
            .with_texture(&self.textures["mask_sheet.png"], 2)
            .draw();

        // Fill fg tile buffers with data
        let tile_count = super::functions::gen_tile_buffers(
            &mut self.tile_xyz,
            &mut self.tile_tex_uv,
            &mut self.tile_msk_uv,
            render_frame.tiles_x,
            render_frame.tiles_y,
            render_frame.foreground_tiles,
        );

        // Render tiles.
        ezgl::Draw::start_tri_draw(tile_count as u32 / 2, &self.programs["fg_tile"], &self.ibo)
            .with_buffer(&self.tile_xyz, 0)
            .with_buffer(&self.tile_tex_uv, 1)
            .with_buffer(&self.tile_msk_uv, 2)
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], 0)
            .with_texture(&self.textures["tile_sheet.png"], 1)
            .with_texture(&self.textures["mask_sheet.png"], 2)
            .draw();

        // Fill light buffers with data.
        super::functions::gen_light_buffers(
            &mut self.light_xy,
            &mut self.light_uv,
            &mut self.light_tex,
            render_frame.light_x,
            render_frame.light_y,
            render_frame.light_map,
        );

        // Render light map.
        unsafe {
            ezgl::gl::TexParameteri(
                ezgl::gl::TEXTURE_2D,
                ezgl::gl::TEXTURE_MIN_FILTER,
                ezgl::gl::NEAREST as _,
            );
            ezgl::gl::TexParameteri(
                ezgl::gl::TEXTURE_2D,
                ezgl::gl::TEXTURE_MAG_FILTER,
                ezgl::gl::LINEAR as _,
            );
        }
        ezgl::Draw::start_tri_draw(2, &self.programs["light"], &self.ibo)
            .with_buffer(&self.light_xy, 0)
            .with_buffer(&self.light_uv, 1)
            .with_uniform(view.as_ref() as &[[f32; 3]; 3], 0)
            .with_texture(&self.light_tex, 1)
            .enable_blend(ezgl::gl::SRC_ALPHA, ezgl::gl::ONE_MINUS_SRC_ALPHA)
            .draw();
    }
}
