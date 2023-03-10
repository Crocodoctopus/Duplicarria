#version 410

in vec2 frag_tile_uv;
in vec2 frag_mask_uv;

uniform sampler2D tile_sheet;
uniform sampler2D mask_sheet;

out vec4 rgba;

void main() {
	vec2 tile_tex_size = vec2(textureSize(tile_sheet, 0));
	vec2 mask_tex_size = vec2(textureSize(mask_sheet, 0));
	rgba = texture(tile_sheet, frag_tile_uv/tile_tex_size) * texture(mask_sheet, frag_mask_uv/mask_tex_size);
	if (rgba.a < 0.9) discard;
}