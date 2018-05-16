extern crate kiss3d;
extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate alga;
extern crate failure;
extern crate glfw;
extern crate itertools;
#[macro_use] extern crate lazy_static;

use std::{
    rc::{Weak, Rc},
    cell::RefCell,
};

use kiss3d::{
    window::Window,
    light::Light,
    camera::{ArcBall, Camera},
};

use na::{
    Vector3,
    UnitQuaternion,
    Point3,
    Vector2,
    Point2,
};

use nc::query::Ray3;
use alga::linear::Transformation;
use glfw::{MouseButtonMiddle, MouseButtonRight, MouseButtonLeft, Action};

use self::component::Component;

mod component;

pub(crate) type Result<T> = std::result::Result<T, failure::Error>;

lazy_static! {
    static ref BOX_EDGES: Vec<(Point3<f32>, Point3<f32>)> = {
        let points = vec![
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(-0.5, 0.5, 0.5),
            Point3::new(0.5, -0.5, 0.5),
            Point3::new(0.5, 0.5, -0.5),
            Point3::new(-0.5, -0.5, 0.5),
            Point3::new(-0.5, 0.5, -0.5),
            Point3::new(0.5, -0.5, -0.5),
            Point3::new(-0.5, -0.5, -0.5),
        ];

        use itertools::Itertools;

        let lines = points.clone().into_iter().cartesian_product(points.into_iter())
            .filter(|(p1, p2)| (p1 - p2).norm() == 1.0)  // this is messy
            .collect();

        lines
    };
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");


fn run() -> Result<()> {
    #[cfg(debug_assertions)]
    let mut window = Window::new(&format!("{} {} (dev)", NAME, VERSION));

    #[cfg(not(debug_assertions))]
    let mut window = Window::new(&format!("{} {}", NAME, VERSION));

    let mut camera = ArcBall::new(Point3::new(0.0f32, 0.0, -1.0), Point3::origin());
    camera.rebind_drag_button(Some(MouseButtonMiddle));
    camera.rebind_rotate_button(Some(MouseButtonRight));

    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(70));

    let non_collision_color = Point3::new(1.0, 1.0, 1.0);
    let collision_color = Point3::new(1.0, 0.7, 0.7);

    let mut components = Vec::<Rc<RefCell<Component>>>::new();
    let mut selection: Weak<RefCell<Component>> = Weak::new();

    #[derive(Clone, Debug, PartialEq)]
    struct DragState {
        pub origin_orientation: UnitQuaternion<f32>, // original orientation of the selected box
        pub local_handle_offset: Vector3<f32>,       // vector describing the local "attachment point" of the cursor to the box in its original orientation
        pub camera_dist: f32,
    }

    let mut drag_state: Option<DragState> = None;

//    let mut debug_lines: Vec<(Point3<f32>, Point3<f32>)> = vec!();

    while window.render_with_camera(&mut camera) {
        use glfw::WindowEvent::*;
        use glfw::Key;

        for ref mut evt in window.events().iter() { match evt.value {
            ref evt @ Scroll(_, _) => {
                camera.handle_event(window.glfw_window(), &evt)
            },

            CursorPos(x, y) => {
                drag_state.iter().for_each(|ref drag_state| {
                    selection.upgrade().map(|comp| {
                        let (pos, dir) =
                            camera.unproject(&Point2::new(x as f32, y as f32), &Vector2::new(window.width(), window.height()));

                        let comp = comp.borrow_mut();

                        let camera_rel = comp.origin - camera.eye().coords;
                    });
                });
            },

            MouseButton(MouseButtonLeft, Action::Release, _) => {
                drag_state = None;
            },

            Key(Key::Escape, _, Action::Press, _) => {
                // todo: inhibit event
                selection = Weak::new();
            },

            Key(Key::N, _, Action::Press, _) => {
                let (x, y) = window.glfw_window().get_cursor_pos();

                let (loc, dir) = camera.unproject(
                    &Point2::new(x as f32, y as f32),
                    &Vector2::new(window.width(), window.height())
                );

                // ray-plane intersection
                // place the new cube on a plane intersecting the origin and normal to the
                // camera direction
                let plane_normal = camera.at() - camera.eye();
                let t = -(loc.coords.dot(&plane_normal)) / dir.dot(&plane_normal);
                let intersect = loc + t * dir;

                let mut new_component = Component::new(&mut window);
                new_component.origin = intersect.coords;
                new_component.apply();

                components.push(Rc::new(RefCell::new(new_component)));
            },

            _ => {},
        } }

        use nc::query::RayCast;

        let (x, y) = window.glfw_window().get_cursor_pos();

        let (loc, dir) = camera.unproject(
            &Point2::new(x as f32, y as f32),
            &Vector2::new(window.width(), window.height())
        );

        let ray = Ray3::new(loc, dir);

        use std::cmp::Ordering;
        let toi = components.iter()
            .enumerate()
            .filter_map(|(idx, comp)| {
                let comp = comp.borrow();
                comp.cuboid().toi_with_ray(&comp.cuboid_transform(), &ray, true).map(|x| (idx, x))
            })
            .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(Ordering::Less));

        match toi {
            Some((idx, toi)) => {
                let mut comp = components[idx].borrow_mut();
                comp.color = collision_color.coords.clone();
                comp.apply();

                if window.glfw_window().get_mouse_button(MouseButtonLeft) == Action::Press {
                    selection = Rc::downgrade(&components[idx]);

                    drag_state = Some(DragState {
                        origin_orientation: comp.orientation,
                        local_handle_offset: (loc + dir * toi).coords - comp.origin,
                        camera_dist: (camera.eye().coords - comp.origin).norm(),
                    });
                }

                for i in 0..components.len() {
                    if i == idx {
                        continue;
                    }

                    let mut comp = components[i].borrow_mut();

                    comp.color = non_collision_color.coords.clone();
                    comp.apply();
                }
            },
            None => {
                components.iter().for_each(|comp| {
                    let mut comp = comp.borrow_mut();

                    comp.color = non_collision_color.coords.clone();
                    comp.apply();
                });

                if window.glfw_window().get_mouse_button(MouseButtonLeft) == Action::Press {
                    selection = Weak::new();
                }
            },
        }

        selection.upgrade().map(|comp| {
            comp.borrow().edges().iter()
                .for_each(|(p1, p2)| window.draw_line(p1, p2, &Point3::new(1.0, 0.5, 0.5)));
        });

        BOX_EDGES.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &non_collision_color));

//        debug_lines.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &Point3::new(0.7, 0.7, 1.0)));

        drag_state.iter().for_each(|drag_state| {
            selection.upgrade().map(|comp| {
                let comp = comp.borrow();

                let origin = Point3 { coords: comp.origin };
                let rotation = comp.orientation / drag_state.origin_orientation;
                let terminus = origin + rotation.transform_vector(&drag_state.local_handle_offset);

                window.draw_line(&origin, &terminus, &Point3::new(0.7, 0.7, 1.0));
            });
        });
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {},
        Err(e) => {
            println!("error: {}", e);
        },
    }
}
