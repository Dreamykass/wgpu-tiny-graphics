// shader.vert
#version 450

layout(location=0) in vec3 a_position; // from the vertex
layout(location=1) in vec3 a_color; // from the vertex

layout(location=0) out vec3 v_color; // to the fragment shader

void main() {
    // v_color = a_color;

    vec3 a = a_color;
    if(gl_VertexIndex == 1 || gl_VertexIndex == 2){
        a.r = 0;
        a.g = 0;
        a.b = 0;
    }
    v_color = a;

    gl_Position = vec4(a_position, 1.0);
}