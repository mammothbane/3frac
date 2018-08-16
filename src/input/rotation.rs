use crate::{
    ROTATE_ADJUST_BASE,
    ROTATE_ADJUST_FINE,
    state::State,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::collections::HashSet;
use super::InputResult;

static ROTATION_KEYS: HashSet<Key> = {
    use glfw::Key::*;

    vec! {I, J, K, L, U, O}.into();
};

pub fn rotation(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;

    match event {
        Key(variant, _, Action::Press, mods) if ROTATION_KEYS.contains(variant) => {
            let axis = match variant {
                Key::I => Vector3::z(),
                Key::K => -Vector3::z(),
                Key::J => Vector3::x(),
                Key::L => -Vector3::x(),
                Key::U => Vector3::y(),
                Key::O => -Vector3::y(),
            };

            state.selection().map(|comp| {
                let mut comp = comp.borrow_mut();
                let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                    ROTATE_ADJUST_BASE
                } else {
                    ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                };

                comp.orientation *= UnitQuaternion::from_axis_angle(&axis, rotate_factor);
            });

            InputResult::Handled
        },

        Key(Key::Backspace, _, Action::Press, mods) if (mods & Modifiers::Shift).is_empty() => {
            state.selection().map(|comp| {
                comp.borrow_mut().orientation = UnitQuaternion::identity();
            });

            InputResult::Handled
        },

        _ => {
            InputResult::Continue
        }
    }
}
