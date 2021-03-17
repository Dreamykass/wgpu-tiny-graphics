// shader.vert
#version 450

layout(location=0) in vec3 a_position; // from the vertex
layout(location=1) in vec3 a_color; // from the vertex

layout(location=0) out vec3 v_color; // to the fragment shader

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view;
};

void main() {
    v_color = a_color;

    gl_Position = u_view * vec4(a_position, 1.0);
}