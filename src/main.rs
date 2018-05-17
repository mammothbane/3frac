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
    Isometry3,
};

use nc::query::{Ray3, RayCast};

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

const SELECTION_BBOX_SCALE: f32 = 1.1;

const KEYBOARD_TRANSLATE_BASE: f32 = 0.1;
const KEYBOARD_TRANSLATE_LARGE: f32 = 10.0;

fn main() -> Result<()> {
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
        use glfw::{Key, Modifiers};

        // -- mouse ray collision --

        fn project_mouse(window: &Window, camera: &ArcBall) -> Ray3<f32> {
            let (x, y) = window.glfw_window().get_cursor_pos();

            let (loc, dir) = camera.unproject(
                &Point2::new(x as f32, y as f32),
                &Vector2::new(window.width(), window.height())
            );

            Ray3::new(loc, dir)
        }

        let mouse_projection = project_mouse(&window, &camera);

        let ray_intersect = {
            use std::cmp::Ordering;

            components.iter()
                .enumerate()
                .filter_map(|(idx, comp)| {
                    let comp = comp.borrow();
                    comp.cuboid().toi_with_ray(&comp.cuboid_transform(), &mouse_projection, true).map(|x| (idx, x))
                })
                .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(Ordering::Less))
                .map(|(idx, toi)| (Rc::downgrade(&components[idx]), mouse_projection.origin + toi * mouse_projection.dir))
        };

        match ray_intersect {
            Some((ref comp, _)) => {
                let comp = comp.upgrade().expect("failed to upgrade newly-created Weak");
                {
                    let mut comp = comp.borrow_mut();

                    comp.color = collision_color.coords.clone();
                    comp.apply();
                }

                let comp_base = comp.borrow();

                for ref comp in components.iter() {
                    if *comp.borrow() == *comp_base {
                        continue;
                    }

                    let mut comp = comp.borrow_mut();

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
            },
        }

        for ref mut evt in window.events().iter() { match evt.value {
            ref evt @ Scroll(_, _) => {
                camera.handle_event(window.glfw_window(), &evt)
            },

            MouseButton(MouseButtonLeft, Action::Press, _) => {
                match ray_intersect {
                    Some((ref comp, ref intersect)) => {
                        selection = comp.clone();

                        comp.upgrade().iter().for_each(|comp| {
                            let comp = comp.borrow();

                            drag_state = Some(DragState {
                                origin_orientation: comp.orientation,
                                local_handle_offset: intersect.coords - comp.origin,
                                camera_dist: (camera.eye().coords - intersect.coords).norm(),
                            });
                        });
                    },

                    None => {
                        selection = Weak::new();
                    },
                }
            },

            MouseButton(MouseButtonLeft, Action::Release, _) => {
                drag_state = None;
            },

            Key(Key::W, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        KEYBOARD_TRANSLATE_BASE
                    } else {
                        KEYBOARD_TRANSLATE_BASE * KEYBOARD_TRANSLATE_LARGE
                    };

                    comp.origin += translate_factor * Vector3::z();
                });
            },

            Key(Key::A, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        KEYBOARD_TRANSLATE_BASE
                    } else {
                        KEYBOARD_TRANSLATE_BASE * KEYBOARD_TRANSLATE_LARGE
                    };

                    comp.origin += translate_factor * Vector3::x();
                });
            },

            Key(Key::S, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        KEYBOARD_TRANSLATE_BASE
                    } else {
                        KEYBOARD_TRANSLATE_BASE * KEYBOARD_TRANSLATE_LARGE
                    };

                    comp.origin += -translate_factor * Vector3::z();
                });
            },

            Key(Key::D, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        KEYBOARD_TRANSLATE_BASE
                    } else {
                        KEYBOARD_TRANSLATE_BASE * KEYBOARD_TRANSLATE_LARGE
                    };

                    comp.origin += -translate_factor * Vector3::x();
                });
            },

            Key(Key::Space, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        KEYBOARD_TRANSLATE_BASE
                    } else {
                        KEYBOARD_TRANSLATE_BASE * KEYBOARD_TRANSLATE_LARGE
                    };

                    comp.origin += translate_factor * Vector3::y();
                });
            },

            Key(Key::C, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        KEYBOARD_TRANSLATE_BASE
                    } else {
                        KEYBOARD_TRANSLATE_BASE * KEYBOARD_TRANSLATE_LARGE
                    };

                    comp.origin += -translate_factor * Vector3::y();
                });
            },

            Key(Key::Escape, _, Action::Press, _) => {
                // TODO: inhibit event
                selection = Weak::new();
            },

            Key(Key::N, _, Action::Press, _) => {
                use nc::shape::Plane3;
                use na::Unit;

                let (x, y) = window.glfw_window().get_cursor_pos();

                let (loc, dir) = camera.unproject(
                    &Point2::new(x as f32, y as f32),
                    &Vector2::new(window.width(), window.height())
                );

                let ray = Ray3::new(loc, dir);

                let plane = Plane3::new(Unit::new_normalize( camera.eye() - camera.at()));
                let toi = plane.toi_with_ray(&Isometry3::identity(), &ray, true)
                    .expect("no intersection between mouse ray and camera plane (should, practically speaking, be impossible)");

                let intersect = loc + toi * dir;

                let mut new_component = Component::new(&mut window);
                new_component.origin = intersect.coords;
                new_component.apply();

                components.push(Rc::new(RefCell::new(new_component)));
            },

            _ => {},
        } }

        let mouse_projection = project_mouse(&window, &camera);

        selection.upgrade().map(|comp| {
            let comp = comp.borrow();

            comp.edges().iter()
                .for_each(|(p1, p2)| {
                    let p1_vec = p1.coords - comp.origin;
                    let p2_vec = p2.coords - comp.origin;

                    let p1 = Point3 { coords: comp.origin + SELECTION_BBOX_SCALE * p1_vec };
                    let p2 = Point3 { coords: comp.origin + SELECTION_BBOX_SCALE * p2_vec };

                    window.draw_line(&p1, &p2, &Point3::new(1.0, 0.5, 0.5))
                });
        });

        BOX_EDGES.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &non_collision_color));

//        debug_lines.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &Point3::new(0.7, 0.7, 1.0)));

        drag_state.iter().for_each(|drag_state| {
            selection.upgrade().map(|comp| {
                let mut comp = comp.borrow_mut();

                let rotation = comp.orientation / drag_state.origin_orientation;
                let new_terminus = mouse_projection.origin + drag_state.camera_dist * mouse_projection.dir.normalize();
                let new_origin = new_terminus - rotation.transform_vector(&drag_state.local_handle_offset);

                comp.origin = new_origin.coords;
                comp.apply();
            });
        });
    }

    Ok(())
}
