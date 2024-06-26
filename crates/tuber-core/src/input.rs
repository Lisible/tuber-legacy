use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use log::{info, trace};
use serde_derive::Deserialize;

use crate::input::keyboard::Key;
use crate::{CoreError, CoreResult};

pub mod keyboard {
    use serde_derive::Deserialize;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
    pub enum Key {
        A = 0,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
        Number0,
        Number1,
        Number2,
        Number3,
        Number4,
        Number5,
        Number6,
        Number7,
        Number8,
        Number9,
        Spacebar,
        Return,
        LShift,
        RShift,
        LControl,
        RControl,
        Escape,
        UpArrow,
        DownArrow,
        LeftArrow,
        RightArrow,
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
    }
}

const KEY_COUNT: usize = 59;

pub mod mouse {
    #[derive(Debug, Copy, Clone)]
    pub enum Button {
        Left,
        Right,
        Middle,
    }
}

#[derive(Debug)]
pub enum Input {
    ActionDown(String),
    ActionUp(String),
    KeyDown(Key),
    KeyUp(Key),
    MouseMotion((f32, f32)),
    MouseButtonDown(mouse::Button),
    MouseButtonUp(mouse::Button),
}

pub struct State {
    key_state: [bool; KEY_COUNT],
    previous_key_state: [bool; KEY_COUNT],
    mouse_button_state: [bool; 3],
    previous_mouse_button_state: [bool; 3],
    last_mouse_position: (f32, f32),
    mouse_moved: bool,
    keymap: Keymap,
}

impl State {
    #[must_use]
    pub fn new(keymap: Keymap) -> Self {
        Self {
            key_state: [false; KEY_COUNT],
            previous_key_state: [false; KEY_COUNT],
            mouse_button_state: [false; 3],
            previous_mouse_button_state: [false; 3],
            last_mouse_position: (0.0, 0.0),
            mouse_moved: false,
            keymap,
        }
    }

    #[must_use]
    pub fn is(&self, input: Input) -> bool {
        match input {
            Input::KeyDown(key) => self.key_state[key as usize],
            Input::KeyUp(key) => !self.key_state[key as usize],
            Input::MouseButtonDown(button) => self.mouse_button_state[button as usize],
            Input::MouseButtonUp(button) => !self.mouse_button_state[button as usize],
            Input::MouseMotion(..) => self.mouse_moved,
            Input::ActionDown(action) => {
                self.key_state[self.keymap.reversed_keymap[&Action(action)] as usize]
            }
            Input::ActionUp(action) => {
                !self.key_state[self.keymap.reversed_keymap[&Action(action)] as usize]
            }
        }
    }

    #[must_use]
    pub fn was(&self, input: Input) -> bool {
        match input {
            Input::KeyDown(key) => self.previous_key_state[key as usize],
            Input::KeyUp(key) => !self.previous_key_state[key as usize],
            Input::MouseButtonDown(button) => self.previous_mouse_button_state[button as usize],
            Input::MouseButtonUp(button) => !self.previous_mouse_button_state[button as usize],
            Input::MouseMotion(..) => unimplemented!(),
            Input::ActionDown(action) => {
                self.previous_key_state[self.keymap.reversed_keymap[&Action(action)] as usize]
            }
            Input::ActionUp(action) => {
                !self.previous_key_state[self.keymap.reversed_keymap[&Action(action)] as usize]
            }
        }
    }

    pub fn handle_input(&mut self, input: &Input) {
        self.mouse_moved = false;
        self.previous_key_state = self.key_state;
        self.previous_mouse_button_state = self.mouse_button_state;
        trace!("Handling input {:?}", input);
        match *input {
            Input::KeyDown(key) => self.key_state[key as usize] = true,
            Input::KeyUp(key) => self.key_state[key as usize] = false,
            Input::MouseButtonDown(button) => {
                self.mouse_button_state[button as usize] = true;
            }
            Input::MouseButtonUp(button) => {
                self.mouse_button_state[button as usize] = false;
            }
            Input::MouseMotion(new_position) => {
                self.last_mouse_position = new_position;
                self.mouse_moved = true;
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn mouse_position(&self) -> (f32, f32) {
        self.last_mouse_position
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Action(String);

#[derive(Default, Debug, Deserialize)]
pub struct Keymap {
    _keymap: HashMap<Key, Action>,
    reversed_keymap: HashMap<Action, Key>,
}

impl Keymap {
    pub fn from_file(file_path: &Path) -> CoreResult<Self> {
        info!(
            "Loading keymap from file \"{}\"",
            file_path.to_str().unwrap()
        );
        let file = File::open(file_path).map_err(CoreError::KeymapFileOpenError)?;
        let reader = BufReader::new(file);
        let keymap: HashMap<Key, Action> =
            serde_json::from_reader(reader).map_err(CoreError::KeymapParseError)?;
        let reversed_keymap: HashMap<Action, Key> = keymap
            .iter()
            .map(|(key, value)| (value.clone(), *key))
            .collect();

        Ok(Self {
            _keymap: keymap,
            reversed_keymap,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn deserialize() {
        let json =
            "{\"A\": \"do_something\", \"B\": \"do_something_else\", \"C\": \"do_something\"}";

        let keymap = serde_json::from_str::<HashMap<Key, Action>>(json).unwrap();
        assert!(keymap.contains_key(&Key::A));
        assert!(keymap.contains_key(&Key::B));
        assert!(keymap.contains_key(&Key::C));
        assert_eq!(keymap[&Key::A], Action("do_something".into()));
        assert_eq!(keymap[&Key::B], Action("do_something_else".into()));
        assert_eq!(keymap[&Key::C], Action("do_something".into()));
    }
}
