use std::collections::{HashMap, HashSet};

use glfw::Key;

use glam::Vec2;

pub struct EventHandler {
    pub keys_pressed: HashMap<Key, usize>,
    pub keys_pressed_last_frame: HashSet<Key>,

    pub keys_released: HashMap<Key, usize>,
    pub keys_released_last_frame: HashSet<Key>,

    pub mouse_pos: Vec2,
    pub scroll: Vec2,

    pub width: f32,
    pub height: f32,
    
    pub lmb: bool,
    pub rmb: bool,
}
impl EventHandler {
    pub fn new() -> Self {
        Self { 
            keys_pressed: HashMap::new(),
            keys_pressed_last_frame: HashSet::new(),

            keys_released: HashMap::new(),
            keys_released_last_frame: HashSet::new(),
            
            mouse_pos: Vec2::ONE,

            width: 1.0,
            height: 1.0,

            scroll: Vec2::ZERO,

            lmb: false,
            rmb: false,
        }
    }

    pub fn on_key_press(&mut self, key: Key) {
        self.keys_released.remove(&key);

        let key_handle = self.keys_pressed.len();
        self.keys_pressed.insert(key, key_handle);
    }

    pub fn on_key_release(&mut self, key: Key) {
        self.keys_pressed.remove(&key);

        let key_handle = self.keys_released.len();
        self.keys_released.insert(key, key_handle);
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_pos.x =  x as f32 - self.width / 2.0;
        self.mouse_pos.y = -y as f32 + self.height / 2.0;
    }

    pub fn on_lmb_press(&mut self) {
        self.lmb = true;
    } 
    pub fn on_lmb_release(&mut self) {
        self.lmb = false;
    } 

    pub fn on_rmb_press(&mut self) {
        self.rmb = true;
    } 
    pub fn on_rmb_release(&mut self) {
        self.rmb = false;
    } 

    pub fn on_scroll_change(&mut self, change: Vec2){
        self.scroll = change;
    }

    pub fn on_window_resize(&mut self, w: i32, h: i32) {
        self.width = w as f32;
        self.height = h as f32;
    }

    pub fn update(&mut self) {
        self.keys_pressed_last_frame.clear();
        self.keys_released_last_frame.clear();

        self.scroll = Vec2::ZERO;
        for &key in self.keys_pressed.keys() {
            self.keys_pressed_last_frame.insert(key);
        }

        for &key in self.keys_released.keys() {
            self.keys_released_last_frame.insert(key);
        }
    }

    pub fn key_just_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains_key(&key) && !self.keys_pressed_last_frame.contains(&key)
    }
    pub fn key_just_released(&self, key: Key) -> bool {
        self.keys_released.contains_key(&key) && !self.keys_released_last_frame.contains(&key)
    }
}