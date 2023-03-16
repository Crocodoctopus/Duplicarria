#version 410

in vec3 frag_rgb;

out vec4 rgba;

void main() {
    rgba = vec4(frag_rgb, 1.0); 
}
