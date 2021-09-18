use std::collections::HashSet;

use winit::event::{ElementState, VirtualKeyCode};

#[derive(Debug, Clone)]
pub struct InputManager {
    pressed_keys: HashSet<VirtualKeyCode>,
    
    just_pressed: HashSet<VirtualKeyCode>,
    just_released: HashSet<VirtualKeyCode>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }
    
    pub fn process_keys(&mut self, key: VirtualKeyCode, element_state: ElementState) {
        if element_state == ElementState::Pressed {
            if !self.key_down(key) {
                self.just_pressed.insert(key);
            }
            
            self.pressed_keys.insert(key);
        }

        if element_state == ElementState::Released {
            self.pressed_keys.remove(&key);

            self.just_released.insert(key);
        }
    }

    pub fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
    }

    pub fn clear_just_released(&mut self) {
        self.just_released.clear();
    }

    pub fn key_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn key_up(&self, key: VirtualKeyCode) -> bool {
        !self.key_down(key)
    }

    pub fn key_just_released(&self, key: VirtualKeyCode) -> bool {
        self.just_released.contains(&key)
    }
}