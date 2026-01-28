#version 100

attribute vec3 star_pos;

uniform mat4 view_proj;

void main() {
    gl_PointSize = 1.0;
    gl_Position = view_proj * vec4(star_pos, 1.0);
}
