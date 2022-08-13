#version 310 es

precision mediump float;

in vec2 frag_uv;

layout(location = 1) uniform sampler2D light_map;

out vec4 rgba;

void main() {
	vec2 light_map_size = vec2(textureSize(light_map, 0));
	rgba = texture(light_map, frag_uv);
	//rgba.a = 0.9;
}