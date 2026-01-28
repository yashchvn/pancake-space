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
    mat4 model = mat4(model0, model1, model2, model3);

    vec3 pos = in_pos;

    // If it's a wireframe pass, scale up the geometry to avoid z-fighting with solid pass
    if (color_override.a > 0.0) {
        pos *= 1.01;
    }

    gl_Position = view_proj * model * vec4(pos, 1.0);

    // Pass color
    if (color_override.a > 0.0) {
        out_color = color_override;
    } else {
        out_color = in_color;
    }
}
