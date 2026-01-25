#version 100

attribute vec3 in_pos;
attribute vec4 model0;
attribute vec4 model1;
attribute vec4 model2;
attribute vec4 model3;
attribute vec4 in_color;

varying lowp vec4 out_color;

uniform mat4 view_proj;
uniform vec4 color_override;

void main() {
    // Reconstruct the model matrix from the 4 vec4 attributes
    mat4 model = mat4(model0, model1, model2, model3);

    // Transform position
    gl_Position = view_proj * model * vec4(in_pos, 1.0);

    // Pass color to fragment shader
    if (color_override.a > 0.0) {
        out_color = color_override.rgba;
    } else {
        out_color = in_color;
    }
    // out_color = in_color;
}
