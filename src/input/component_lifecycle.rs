use crate::{
    component::Component,
    state::State,
    state::world::DragState,
};
use glfw::{Action, Key, Modifiers, WindowEvent};
use std::cell::RefCell;
use std::rc::Rc;
use super::InputResult;

pub fn create(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;
    use glfw::WindowEvent::Key;

    match event {
        Key(Key::N, _, Action::Press, mods) => {
            use nc::shape::Plane3;
            use na::Unit;

            let (x, y) = state.glfw_window().get_cursor_pos();

            let (loc, dir) = camera.unproject(
                &Point2::new(x as f32, y as f32),
                &Vector2::new(window.width(), window.height())
            );

            let ray = state.project_mouse();

            let plane = Plane3::new(Unit::new_normalize( camera.eye() - camera.at()));
            let toi = plane.toi_with_ray(&Isometry3::identity(), &ray, true)
                .expect("no intersection between mouse ray and camera plane (should, practically speaking, be impossible)");

            let intersect = loc + toi * dir;

            let mut new_component = Component::new(state.root_group());
            new_component.origin = intersect.coords;

            if !(mods & Modifiers::Shift).is_empty() {
                new_component.scale *= 2.0;
            }

            components.push(Rc::new(RefCell::new(new_component)));

            InputResult::Handled
        },


        _ => InputResult::Continue,
    }
}


pub fn delete(state: &mut State, event: &mut WindowEvent) -> InputResult {
    use glfw::Key;
    use glfw::WindowEvent::Key;

    match event {
        Key(Key::Escape, _, action, _) => {
            evt.inhibited = true;

            if action == Action::Press {
                state.deselect()
            }

            InputResult::Handled
        },

    }
}
