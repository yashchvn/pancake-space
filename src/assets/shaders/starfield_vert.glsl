#version 100

attribute vec3 star_pos;

uniform mat4 view_proj;

void main() {
    gl_PointSize = 1.0;
    mat4 rotation_only = view_proj;
    rotation_only[3] = vec4(0.0, 0.0, 0.0, 1.0);

    // Apply rotation-only matrix
    vec4 pos = rotation_only * vec4(star_pos, 1.0);

    // Set z = w to place stars at far plane
    gl_Position = pos.xyww;
}
