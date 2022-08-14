#version 410

in vec3 vert_tile_xyz;
in vec2 vert_tile_uv;
in vec2 vert_mask_uv;

uniform mat3 view_matrix;

out vec2 frag_tile_uv;
out vec2 frag_mask_uv;

void main() {
	gl_Position = vec4((view_matrix * vec3(vert_tile_xyz.xy, 1)).xy, -vert_tile_xyz.z/256., 1);

	frag_tile_uv = vert_tile_uv;
	frag_mask_uv = vert_mask_uv;
}