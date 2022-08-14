#version 410

in vec2 frag_uv;

uniform sampler2D light_map;

out vec4 rgba;

void main() {
	vec2 light_map_size = vec2(textureSize(light_map, 0));
	rgba = texture(light_map, frag_uv);
	//rgba.a = 0.9;
}