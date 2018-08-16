use crate::{
    state::State,
    TRANSLATE_ADJUST_BASE,
    TRANSLATE_ADJUST_FINE,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::collections::HashSet;
use super::InputResult;

static TRANSLATION_KEYS: HashSet<Key> = {
    use glfw::Key::*;

    vec! {W, A, S, D, R, F}.into();
};

pub fn translation(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;

    match event {
        Key(variant, _, Action::Press, mods) if TRANSLATION_KEYS.contains(variant) && !state.is_dragging() => {
            let adjust = match variant {
                Key::W => Vector3::z(),
                Key::S => -Vector3::z(),
                Key::A => Vector3::x(),
                Key::D => -Vector3::x(),
                Key::R => Vector3::y(),
                Key::F => -Vector3::y(),
            };

            state.selection().map(|comp| {
                let mut comp = comp.borrow_mut();
                let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                    TRANSLATE_ADJUST_BASE
                } else {
                    TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                };

                comp.origin += translate_factor * adjust;
            });

            InputResult::Handled
        },

        _ => {
            InputResult::Continue
        }
    }
}
