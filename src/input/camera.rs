use crate::{
    SCALE_ADJUST_BASE,
    SCALE_ADJUST_FINE,
    state::State,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::collections::HashSet;
use super::InputResult;

pub fn zoom(state: &mut State, event: &mut WindowEvent) -> InputResult {
    if state.selection().is_some() {
        return InputResult::Continue
    }

    match event {
        Scroll(unused, offset) => {
            event.inhibited = true;

            state.camera().handle_event(window.glfw_window(), &WindowEvent::Scroll(unused, -offset));

            InputResult::Handled
        },
        _ => InputResult::Continue,
    }
}
