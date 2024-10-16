pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 pos;

            layout (location = 1) in vec3 ofs; // per instance

            layout(set = 0, binding = 0) uniform Camera {
                mat4 view;
                mat4 proj;
            };

            layout(set = 1, binding = 0) uniform Model {
                mat4 model;
            };

            layout(location = 0) out vec4 o_pos;

            void main() {
                gl_Position = proj * view * model * vec4(pos + ofs, 1.0);

                o_pos = vec4(pos + ofs, 1.0);
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) out vec4 f_color;

            layout(location = 0) in vec4 i_pos;

            void main() {
                f_color = vec4(i_pos.xyz / 10.0, 1.0);
            }
        ",
    }
}