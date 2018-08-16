use crate::{
    state::State,
    state::world::DragState,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::collections::HashSet;
use super::InputResult;

pub fn select(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::MouseButton;
    use glfw::WindowEvent::MouseButton;

    match event {
        MouseButton(MouseButtonLeft, Action::Press, _) => {
            match state.nearest_intersection() {
                Some((ref comp, ref intersect)) => {
                    comp.upgrade().iter().for_each(|comp| {
                        state.select(comp.clone());

                        let comp = comp.borrow();

                        let drag_state = DragState {
                            origin_orientation: comp.orientation,
                            local_handle_offset: intersect.coords - comp.origin,
                            camera_dist: (mouse_projection.origin.coords - intersect.coords).norm(),
                        };

                        state.drag(drag_state);
                    });
                },

                None => state.deselect(),
            }

            InputResult::Handled
        },


        _ => InputResult::Continue,
    }
}


pub fn deselect(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;
    use glfw::WindowEvent::Key;

    match event {
        Key(Key::Backspace, _, Action::Press, _) if (mods & Modifiers::Shift).is_empty() => {
            state.delete_selected();

            InputResult::Handled
        },

    }
}