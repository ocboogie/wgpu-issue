#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) in vec2 a_position;

struct Primitive {
    vec2 translate;
    // vec2 filler;
};

layout(std140, binding = 0)
uniform u_primitives { Primitive primitives[2]; };

void main() {
    Primitive prim = primitives[gl_InstanceIndex];
    
    gl_Position = vec4(a_position + prim.translate, 0.0, 1.0);
}