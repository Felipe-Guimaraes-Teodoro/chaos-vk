mod vk_renderer;

pub use vk_renderer::tests::*;

fn main() {
    println!("Hello, world!");

    std::thread::spawn(|| {
        test();
    });

    loop {
        
    }
}
