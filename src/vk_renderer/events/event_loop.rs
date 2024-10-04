use glam::vec2;
use glfw::*;

use super::super::{renderer::Renderer, ui::imgui::ImGui};

use super::event_handler::EventHandler;

pub struct EventLoop {
    pub event_handler: EventHandler,
    pub window: PWindow,
    pub ui: ImGui,
    pub glfw: Glfw,
    events: GlfwReceiver<(f64, WindowEvent)>,
    pub now: std::time::Instant,
    pub dt: f32,
    pub time: f32,

    pub timescale: f32,
}


impl EventLoop {
    pub fn new(w: u32, h: u32) -> Self {
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        
        // glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
        // glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

        let (mut window, events) = glfw.create_window(w, h, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_resizable(false);

        window.make_current();
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        // window.set_size_callback(|window: &mut Window, width: i32, height: i32| resize_callback(&*window, width, height));
    
        let mut event_handler = EventHandler::new();
        event_handler.on_window_resize(w as i32, h as i32);

        let ui = ImGui::new(&mut window);

        Self {
            event_handler,
            window,
            glfw,
            ui,
            events,
            now: std::time::Instant::now(),
            dt: 0.0,
            time: 0.0,
            timescale: 1.0,
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        self.dt = self.now.elapsed().as_secs_f32() * self.timescale;
        self.time += self.dt;
        self.now = std::time::Instant::now();


        self.window.swap_buffers();
        self.glfw.poll_events();

        self.event_handler.update();

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                },
                glfw::WindowEvent::Key(key, _, Action::Press, _ ) => {
                    self.event_handler.on_key_press(key);
                }
                glfw::WindowEvent::Key(key, _, Action::Release, _ ) => {
                    self.event_handler.on_key_release(key);
                }

                glfw::WindowEvent::CursorPos(x, y) => {
                    self.ui.on_mouse_move(x as f32, y as f32);
                    self.event_handler.on_mouse_move(x, y);
                }

                glfw::WindowEvent::MouseButton(button, Action::Press, _) => {
                    match button {
                        glfw::MouseButton::Button1 => {
                            self.ui.on_mouse_click(button, Action::Press);
                            self.event_handler.on_lmb_press();
                        },
                        glfw::MouseButton::Button2 => {
                            self.event_handler.on_rmb_press();
                        },
                        _ => ()
                    }
                }

                glfw::WindowEvent::MouseButton(button, Action::Release, _) => {
                    match button {
                        glfw::MouseButton::Button1 => {
                            self.ui.on_mouse_click(button, Action::Release);
                            self.event_handler.on_lmb_release();
                        },
                        glfw::MouseButton::Button2 => {
                            self.event_handler.on_rmb_release();
                        },
                        
                        _ => ()
                    }
                }

                glfw::WindowEvent::Scroll(xoff, yoff) => {
                    self.ui.on_mouse_scroll(xoff as f32, yoff as f32);
                    self.event_handler.on_scroll_change(vec2(xoff as f32, yoff as f32));
                }

                glfw::WindowEvent::FramebufferSize(w, h) => {
                    self.ui.ctx.io_mut().display_size = [
                        w as f32, h as f32
                    ];
                    self.event_handler.on_window_resize(w, h);
                    renderer.presenter.window_resized = true;
                }
                _ => {},
            }
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        if self.window.get_key(key) == Action::Press {
            true
        } else { 
            false 
        }
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        if self.window.get_key(key) == Action::Release {
            true
        } else {
            false
        }
    }

    // TODO: Fix this for the love of gohf
    pub fn set_fullscreen(&mut self, fullscreen: &mut bool) {
        if self.event_handler.key_just_pressed(Key::F11) {
            if !*fullscreen {
                self.glfw.with_primary_monitor(|_, monitor| {
                    let monitor = monitor.unwrap();
                    let mode = monitor.get_video_mode().unwrap();
                    self.window.set_monitor(
                        glfw::WindowMode::FullScreen(&monitor), 
                        0, 
                        0, 
                        mode.width, 
                        mode.height, 
                        Some(mode.refresh_rate),
                    );
                    *fullscreen = true;
                });
            } else {
                self.glfw.with_primary_monitor(|_, monitor| {
                    let monitor = monitor.unwrap();
                    let mode = monitor.get_video_mode().unwrap();
                    self.window.set_monitor(
                        glfw::WindowMode::Windowed, 
                        200, 
                        200, 
                        800, 
                        800, 
                        Some(mode.refresh_rate),
                    );
                    *fullscreen = false;
                });
            }
        }
    }
}