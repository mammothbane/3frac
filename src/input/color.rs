use crate::{
    COLOR_ADJUST_BASE,
    COLOR_ADJUST_FINE,
    state::State,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::collections::HashSet;
use super::InputResult;

pub fn color(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;
    use glfw::WindowEvent::Scroll;

    if state.selection().is_none() {
        return InputResult::Continue
    }

    match event {
        Scroll(unused, offset) if glfw_window.get_key(Key::C) == Action::Press => {
            use palette::{Hsl, LinSrgb, FromColor};

            event.inhibited = true;

            let comp = state.selection().unwrap();

            let offset = offset as f32;
            let mut comp = comp.borrow_mut();

            let mut color: Hsl<_, _> = LinSrgb::new(comp.color[0], comp.color[1], comp.color[2]).into();

            let adjustment = if fine {
                COLOR_ADJUST_BASE * COLOR_ADJUST_FINE
            } else {
                COLOR_ADJUST_BASE
            };

            color.hue = color.hue + adjustment * offset;

            let color = LinSrgb::from_hsl(color);

            comp.color = Vector3::new(color.red, color.green, color.blue);

            InputResult::Handled
        },

        _ => {
            InputResult::Continue
        }
    }
}
