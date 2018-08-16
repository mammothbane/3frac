use crate::{
    BOX_EDGES,
    component::Component,
    SELECTION_BBOX_SCALE,
};
use glfw;
use kiss3d::{
    camera::ArcBall,
    scene::SceneNode,
};
use nc::query::Ray3;
use self::render_state::RenderState;
use self::world::{DragState, WorldState};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

mod render_state;
pub mod world;

pub struct State {
    iteration_depth: usize,
    world: WorldState,
    render_state: RenderState,
}

impl State {
    lazy_static! {
        static ref STATE: State = State::new()
    }

    fn new() -> Self {
        let mut render_state = RenderState::default();

        State {
            iteration_depth: 0,
            world: WorldState::new(&mut render_state.window),
            render_state,
        }
    }

    pub fn render(&mut self) {

    }

    pub fn project_mouse(&self) -> Ray3<f32> {
        self.render_state.project_mouse()
    }

    pub fn nearest_intersection(&self) -> Option<(Weak<RefCell<Component>>, Point3<f32>)> {
        use std::cmp::Ordering;
        use nc::query::RayCast;

        let components = self.world.components;
        let mouse_projection = self.render_state.project_mouse();

        components.iter()
            .enumerate()
            .filter_map(|(idx, comp)| {
                let comp = comp.borrow();

                comp.cuboid()
                    .toi_with_ray(&comp.isometric_part(), &mouse_projection, true)
                    .map(|x| (idx, x))
            })
            .min_by(|x, y| {
                x.1.partial_cmp(&y.1).unwrap_or(Ordering::Less)
            })
            .map(|(idx, toi)| {
                let impact = mouse_projection.origin + toi * mouse_projection.dir;
                let component = Rc::downgrade(&components[idx]);

                (component, impact)
            })
    }

    pub fn increase_depth(&mut self) {
        self.iteration_depth += 1
    }

    pub fn decrease_depth(&mut self) {
        self.iteration_depth -= 1
    }

    pub fn is_dragging(&self) -> bool {
        self.world.drag_state.is_some()
    }

    pub fn selection(&self) -> Option<Rc<RefCell<Component>>> {
        self.world.selection.as_ref().map(|sel| sel.clone())
    }

    pub fn select(&mut self, component: Rc<RefCell<Component>>) {
        self.world.selection = Some(component)
    }

    pub fn deselect(&mut self) {
        self.world.selection = None;
    }

    pub fn drag(&mut self, drag_state: DragState) {
        self.world.drag_state = Some(drag_state);
    }

    pub fn stop_dragging(&mut self) {
        self.world.drag_state = None;
    }

    pub fn camera(&mut self) -> &mut ArcBall {
        &mut self.render_state.camera
    }

    pub fn glfw_window(&mut self) -> &glfw::Window {
        self.render_state.window.glfw_window()
    }

    pub fn new_component(&mut self, comp: Component) {
        self.world.components.push(Rc::new(RefCell::new(comp)));
    }

    pub fn root_group(&mut self) -> &mut SceneNode {
        &mut self.world.root_group
    }

    pub fn toggle_wireframes(&mut self) -> {
        self.render_state.wireframes_enabled = !self.render_state.wireframes_enabled
    }

    pub fn delete_selected(&mut self) {
        self.selection().map(|comp| {
            let mut components = self.world.components;

            let idx = components.iter().position(|c| *c.borrow() == *comp.borrow()).expect("selection didn't exist in vec");
            let comp = components.swap_remove(idx);

            {
                comp.borrow_mut().scene_node.unlink();
            }
        });
    }

    pub fn drag_update(&mut self) {
        self.world.drag_state.iter().for_each(|drag_state| {
            self.selection().map(|comp| {
                let mut comp = comp.borrow_mut();

                let rotation = comp.orientation / drag_state.origin_orientation;
                let new_terminus = mouse_projection.origin + drag_state.camera_dist * mouse_projection.dir.normalize();
                let new_origin = new_terminus - rotation.transform_vector(&drag_state.local_handle_offset);

                comp.origin = new_origin.coords;
            });
        });
    }

    pub fn draw_wireframes(&mut self) {
        let mut window = self.render_state.window;

        BOX_EDGES.iter().for_each(|(p1, p2)|
            window.draw_line(p1, p2, &Point3::new(1.0, 1.0, 1.0)));

        self.world.components.iter().for_each(|comp| {
            use alga::linear::Transformation;

            let comp = comp.borrow();

            let corner_vec = 1.05f32 * (Vector3::new(-0.5, 0.5, -0.5).component_mul(&comp.scale));
            let corner_vec = comp.orientation.transform_vector(&corner_vec) + comp.origin;

            let box_scale = 0.01 * comp.scale.norm();
            BOX_EDGES.iter().for_each(|(p1, p2)| {
                let p1 = Point3::from_coordinates(box_scale * p1.coords + corner_vec);
                let p2 = Point3::from_coordinates(box_scale * p2.coords + corner_vec);

                window.draw_line(&p1, &p2, &Point3::new(0.0, 1.0, 1.0));
            });

            if iteration_depth > 0 {
                if selection.upgrade().map_or(false, |x| *x.borrow() == *comp) {
                    return;
                }

                let transform = comp.transform();

                BOX_EDGES.iter().for_each(|(p1, p2)| {
                    window.draw_line(&transform.transform_point(&p1), &transform.transform_point(&p2), &Point3::new(0.5, 0.5, 0.9))
                });
            }
        });

        self.selection().iter().for_each(|comp| {
            let comp = comp.borrow();

            let scale = if iteration_depth > 0 {
                Matrix4::new_nonuniform_scaling(&comp.scale)
            } else {
                Matrix4::new_scaling(SELECTION_BBOX_SCALE) * Matrix4::new_nonuniform_scaling(&comp.scale)
            };

            let transform = comp.isometric_part().to_homogeneous() * scale;

            BOX_EDGES.iter()
                .for_each(|(p1, p2)|
                    window.draw_line(&transform.transform_point(&p1), &transform.transform_point(&p2), &Point3::new(1.0, 0.5, 0.5))
                );
        });
    }

    pub fn draw_overlay_text(&mut self) {
        let mut window = self.render_state.window;

        let pos = Point2::new(window.width() * 2.0 - 300.0, window.height() * 2.0 - 165.0);
        window.draw_text(&format!("iterations: {}", iteration_depth), &pos, &self.render_state.font, &Point3::new(0.9, 0.9, 0.9));

        let cube_count = components.len().pow(iteration_depth as u32 + 1);
        window.draw_text(&format!("cubes: {}", cube_count), &Point2::new(pos[0], pos[1] + 75.0), &self.render_state.font, &Point3::new(0.9, 0.9, 0.9));

        self.selection().iter().for_each(|comp| {
            let comp = comp.borrow();

            let matrix = comp.transform().matrix().clone();
            let mut matrix_fmt = String::new();
            for i in 0..4 {
                for j in 0..4 {
                    matrix_fmt.push_str(&format!("{: >6.2} ", matrix[(i, j)]));
                }
                matrix_fmt.push('\n');
            }

            let text = format!("selected transform (matrix representation)\n{}", matrix_fmt);
            window.draw_text(&text, &Point2::new(10.0, 10.0), &roboto_font, &Point3::new(0.9, 0.9, 0.9));
        }
    }
}
