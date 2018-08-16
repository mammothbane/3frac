use crate::{
    component::Component,
    state::State,
    state::world::DragState,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::cell::RefCell;
use std::rc::Rc;
use super::InputResult;

pub fn toggle_wireframes(state: &mut State, event: &mut WindowEvent) -> InputResult {
    match event {
        Key(Key::Tab, _, Action::Press, _) => {
            state.toggle_wireframes();
            InputResult::Handled
        },
        _ => InputResult::Continue,
    }
}

pub fn increase_depth(state: &mut State, event: &mut WindowEvent) -> InputResult {
    match event {
        Key(Key::Right, _, Action::Press, _) => {
            state.increase_depth();
            InputResult::Handled
        },
        _ => InputResult::Continue,
    }
}

pub fn decrease_depth(state: &mut State, event: &mut WindowEvent) -> InputResult {
    match event {
        Key(Key::Right, _, Action::Press, _) => {
            state.decrease_depth();
            InputResult::Handled
        },
        _ => InputResult::Continue,
    }
}