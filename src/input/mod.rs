use crate::{
    state::State,
    TRANSLATE_ADJUST_BASE,
    TRANSLATE_ADJUST_FINE,
};
use glfw::{
    Action,
    Key,
    WindowEvent,
};
use na::Vector3;
use std::collections::HashSet;

mod translation;
mod rotation;
mod scale;
mod color;
mod camera;
mod component_lifecycle;
mod selection;
mod misc;

pub fn process_input(state: &mut State, event: &mut WindowEvent) {

}

enum InputResult {
    Continue,
    Handled,
}
