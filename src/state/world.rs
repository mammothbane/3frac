use crate::component::Component;
use kiss3d::{
    scene::SceneNode,
    window::Window,
};
use std::{
    cell::RefCell,
    default::Default,
    rc::{Rc, Weak},
};

#[derive(Clone, Debug)]
pub(super) struct WorldState {
    /// The set of boxes in the world.
    pub components: Vec<Rc<RefCell<Component>>>,

    /// Description of currently-dragged box. Exists iff a box is being dragged.
    pub drag_state: Option<DragState>,

    /// A reference to the currently-selected box.
    pub selection: Option<Rc<RefCell<Component>>>,

    pub root_group: SceneNode,

    pub iterated_group: SceneNode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DragState {
    /// Original world-space orientation of the selected box.
    pub origin_orientation: UnitQuaternion<f32>,

    /// Vector describing the local "attachment point" of the cursor to the box, relative to its
    /// original orientation.
    pub local_handle_offset: Vector3<f32>,

    /// The camera's distance from its intersection with the box.
    pub camera_dist: f32,
}

impl WorldState
    pub(super) fn new(window: &mut Window) -> Self {
        WorldState {
            components: Vec::new(),
            drag_state: None,
            selection: None,
            root_group: window.add_group(),
            iterated_group: window.add_group(),
        }
    }
}
