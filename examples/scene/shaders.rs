pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 pos;

            layout(set = 0, binding = 0) uniform Camera {
                mat4 view;
                mat4 proj;
            };

            layout(set = 1, binding = 0) uniform Model {
                mat4 model;
            };

            void main() {
                gl_Position = proj * view * model * vec4(pos, 1.0);
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

            void main() {
                f_color = vec4(0.8, 0.2, 0.2, 1.0);
            }
        ",
    }
}