use crate::{
    SCALE_ADJUST_BASE,
    SCALE_ADJUST_FINE,
    state::State,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::collections::HashSet;
use super::InputResult;

static SCALE_KEYS: HashSet<Key> = {
    use glfw::Key::*;

    vec! {X, Y, Z, B}.into();
};

pub fn rotation(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;
    use glfw::WindowEvent::Scroll;

    if state.selection().is_none() {
        return InputResult::Continue
    }

    let glfw_window = state.glfw_window();
    let keys_depressed = SCALE_KEYS.iter().any(|k| glfw_window.get_key(k) == Action::Press);

    match event {
        Scroll(unused, offset) if keys_depressed => {
            event.inhibited = true;

            let comp = state.selection().unwrap();

            let offset = offset as f32;
            let mut comp = comp.borrow_mut();

            let adjustment = if fine {
                SCALE_ADJUST_BASE * SCALE_ADJUST_FINE
            } else {
                SCALE_ADJUST_BASE
            };

            if glfw_window.get_key(Key::B) == Action::Press {
                comp.scale[0] = 0.0f32.max(comp.scale[0] + adjustment * offset);
                comp.scale[1] = 0.0f32.max(comp.scale[1] + adjustment * offset);
                comp.scale[2] = 0.0f32.max(comp.scale[2] + adjustment * offset);
            } else if glfw_window.get_key(Key::X) == Action::Press {
                comp.scale[0] = 0.0f32.max(comp.scale[0] + adjustment * offset);
            } else if glfw_window.get_key(Key::Y) == Action::Press {
                comp.scale[1] = 0.0f32.max(comp.scale[1] + adjustment * offset);
            } else if glfw_window.get_key(Key::Z) == Action::Press {
                comp.scale[2] = 0.0f32.max(comp.scale[2] + adjustment * offset);
            } else {
                return InputResult::Continue
            }

            InputResult::Handled
        },

        _ => {
            InputResult::Continue
        }
    }
}
