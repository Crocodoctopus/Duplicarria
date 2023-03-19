#version 410

in vec2 frag_uv;
in vec3 frag_rgb;

uniform sampler2D tex;

out vec4 rgba;

void main() {
    vec2 tex_size = textureSize(tex, 0).xy;
    rgba = texture(tex, frag_uv / tex_size); 
}
