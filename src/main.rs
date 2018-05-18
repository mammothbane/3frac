#![feature(vec_remove_item)]

extern crate kiss3d;
extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate alga;
extern crate failure;
extern crate glfw;
extern crate itertools;
extern crate palette;
#[macro_use] extern crate lazy_static;

use std::{
    rc::{Weak, Rc},
    cell::RefCell,
};

use kiss3d::{
    window::Window,
    light::Light,
    camera::{ArcBall, Camera},
    text::Font,
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

static ROBOTO_TTF: &'static [u8] = include_bytes!("resources/roboto.ttf");

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

const TRANSLATE_ADJUST_BASE: f32 = 0.1;
const TRANSLATE_ADJUST_FINE: f32 = 0.1;

const ROTATE_ADJUST_BASE: f32 = (2.0 * std::f32::consts::PI) / 24.0;
const ROTATE_ADJUST_FINE: f32 = 1.0 / 12.0;

const SCALE_ADJUST_BASE: f32 = 0.06;
const SCALE_ADJUST_FINE: f32 = 0.25;

const COLOR_ADJUST_BASE: f32 = 2.0;
const COLOR_ADJUST_FINE: f32 = 0.25;

const MAX_CUBES: usize = 25_000;

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let mut window = Window::new_with_size(&format!("{} {} (dev)", NAME, VERSION), 1400, 800);

    #[cfg(not(debug_assertions))]
    let mut window = Window::new_with_size(&format!("{} {}", NAME, VERSION), 1400, 800);

    let roboto_font = Font::from_memory(ROBOTO_TTF, 45);

    let mut camera = ArcBall::new(Point3::new(0.0f32, 0.0, -4.0), Point3::origin());
    camera.rebind_drag_button(Some(MouseButtonMiddle));
    camera.rebind_rotate_button(Some(MouseButtonRight));

    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(70));
    window.set_background_color(0.1, 0.1, 0.1);

    let mut root_group = window.add_group();
    let mut iterated_group = window.add_group();
    let mut components = Vec::<Rc<RefCell<Component>>>::new();
    let mut selection: Weak<RefCell<Component>> = Weak::new();

    #[derive(Clone, Debug, PartialEq)]
    struct DragState {
        pub origin_orientation: UnitQuaternion<f32>, // original orientation of the selected box
        pub local_handle_offset: Vector3<f32>,       // vector describing the local "attachment point" of the cursor to the box in its original orientation
        pub camera_dist: f32,
    }

    let mut drag_state: Option<DragState> = None;

    let mut iteration_depth: usize = 0;

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
                    comp.cuboid().toi_with_ray(&comp.isometric_part(), &mouse_projection, true).map(|x| (idx, x))
                })
                .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(Ordering::Less))
                .map(|(idx, toi)| (Rc::downgrade(&components[idx]), mouse_projection.origin + toi * mouse_projection.dir))
        };

        let mut render_dirty = false;
        
        match ray_intersect {
            Some((ref comp, _)) => {
                let comp = comp.upgrade().expect("failed to upgrade newly-created Weak");
                {
                    let mut comp = comp.borrow_mut();
                    comp.hovered = true;
                    comp.apply();
                }

                let comp_base = comp.borrow();

                for ref comp in components.iter() {
                    if *comp.borrow() == *comp_base {
                        continue;
                    }

                    let mut comp = comp.borrow_mut();
                    comp.hovered = false;
                    comp.apply();
                }
            },
            None => {
                components.iter().for_each(|comp| {
                    let mut comp = comp.borrow_mut();
                    comp.hovered = false;
                    comp.apply();
                });
            },
        }

        for ref mut evt in window.events().iter() { match evt.value {
            MouseButton(MouseButtonLeft, Action::Press, _) => {
                match ray_intersect {
                    Some((ref comp, ref intersect)) => {
                        selection = comp.clone();

                        comp.upgrade().iter().for_each(|comp| {
                            let comp = comp.borrow();

                            drag_state = Some(DragState {
                                origin_orientation: comp.orientation,
                                local_handle_offset: intersect.coords - comp.origin,
                                camera_dist: (mouse_projection.origin.coords - intersect.coords).norm(),
                            });
                        });
                    },

                    None => {
                        selection = Weak::new();
                    },
                }
            },

            MouseButton(MouseButtonLeft, Action::Release, _) => {
                render_dirty = true;
                drag_state = None;
            },

            Scroll(unused, offset) => {
                evt.inhibited = true;

                let glfw_window = window.glfw_window();

                let mut fine = glfw_window.get_key(Key::LeftShift) == Action::Press || glfw_window.get_key(Key::RightShift) == Action::Press;

                if let Some(comp) = selection.upgrade() {
                    let offset = offset as f32;
                    let mut comp = comp.borrow_mut();

                    let adjustment = if fine {
                        SCALE_ADJUST_BASE * SCALE_ADJUST_FINE
                    } else {
                        SCALE_ADJUST_BASE
                    };
                    
                    render_dirty = true;

                    if glfw_window.get_key(Key::B) == Action::Press {
                        comp.scale[0] = 0.0f32.max(comp.scale[0] + adjustment * offset);
                        comp.scale[1] = 0.0f32.max( comp.scale[1] + adjustment * offset);
                        comp.scale[2] = 0.0f32.max(comp.scale[2] + adjustment * offset);
                        comp.apply();
                    } else if glfw_window.get_key(Key::X) == Action::Press {
                        comp.scale[0] = 0.0f32.max(comp.scale[0] + adjustment * offset);
                        comp.apply();
                    } else if glfw_window.get_key(Key::Y) == Action::Press {
                        comp.scale[1] = 0.0f32.max( comp.scale[1] + adjustment * offset);
                        comp.apply();
                    } else if glfw_window.get_key(Key::Z) == Action::Press {
                        comp.scale[2] = 0.0f32.max( comp.scale[2] + adjustment * offset);
                        comp.apply();
                    } else if glfw_window.get_key(Key::C) == Action::Press {
                        use palette::{Hsl, LinSrgb, FromColor};

                        let mut color: Hsl<_, _> = LinSrgb::new(comp.color[0], comp.color[1], comp.color[2]).into();

                        let adjustment = if fine {
                            COLOR_ADJUST_BASE * COLOR_ADJUST_FINE
                        } else {
                            COLOR_ADJUST_BASE
                        };

                        color.hue = color.hue + adjustment * offset;

                        let color = LinSrgb::from_hsl(color);

                        comp.color = Vector3::new(color.red, color.green, color.blue);
                        comp.apply();
                    } else {
                        render_dirty = false;
                        
                        camera.handle_event(window.glfw_window(), &Scroll(unused, -(offset as f64)));
                    }
                } else {
                    camera.handle_event(window.glfw_window(), &Scroll(unused, -offset));
                }
            },
            
            Key(Key::Right, _, Action::Press, _) => {
                iteration_depth += 1;

                render_dirty = true;
            },
            
            Key(Key::Left, _, Action::Press, _) => {
                if iteration_depth == 0 {
                    continue;
                }

                iteration_depth -= 1;
                render_dirty = true;
            },

            Key(Key::Escape, _, action, _) => {
                evt.inhibited = true;

                if action == Action::Press {
                    selection = Weak::new();
                }
            },

            // create a new box
            Key(Key::N, _, Action::Press, mods) => {
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

                let mut new_component = Component::new(&mut root_group);
                new_component.origin = intersect.coords;
                if !(mods & Modifiers::Shift).is_empty() {
                    new_component.scale *= 2.0;
                }

                new_component.apply();

                components.push(Rc::new(RefCell::new(new_component)));
                
                render_dirty = true;
            },

            Key(Key::Backspace, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    if (mods & Modifiers::Shift).is_empty() {  // reset box orientation
                        comp.borrow_mut().orientation = UnitQuaternion::identity();                
                    } else {
                        let idx = components.iter().position(|c| *c.borrow() == *comp.borrow()).expect("selection didn't exist in vec");
                        let comp = components.swap_remove(idx);

                        {
                            comp.borrow_mut().scene_node.unlink();
                        }
                    }
                });
            },

            // -- Translation --
            Key(Key::W, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        TRANSLATE_ADJUST_BASE
                    } else {
                        TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.origin += translate_factor * Vector3::z();
                    comp.apply();
                });
            },

            Key(Key::A, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        TRANSLATE_ADJUST_BASE
                    } else {
                        TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.origin += translate_factor * Vector3::x();
                    comp.apply();
                });
            },

            Key(Key::S, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        TRANSLATE_ADJUST_BASE
                    } else {
                        TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.origin += -translate_factor * Vector3::z();
                    comp.apply();
                });
            },

            Key(Key::D, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        TRANSLATE_ADJUST_BASE
                    } else {
                        TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.origin += -translate_factor * Vector3::x();
                    comp.apply();
                });
            },

            Key(Key::R, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        TRANSLATE_ADJUST_BASE
                    } else {
                        TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.origin += translate_factor * Vector3::y();
                    comp.apply();
                });
            },

            Key(Key::F, _, Action::Press, mods) => {
                if drag_state.is_some() {
                    continue;
                }

                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let translate_factor = if (mods & Modifiers::Shift).is_empty() {
                        TRANSLATE_ADJUST_BASE
                    } else {
                        TRANSLATE_ADJUST_BASE * TRANSLATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.origin += -translate_factor * Vector3::y();
                    comp.apply();
                });
            },

            // -- Rotation --
            // NOTE: rotation is permitted while dragging
            Key(Key::I, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                        ROTATE_ADJUST_BASE
                    } else {
                        ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.orientation *= UnitQuaternion::from_axis_angle(&Vector3::x_axis(), rotate_factor);
                    comp.apply();
                });
            },

            Key(Key::J, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                        ROTATE_ADJUST_BASE
                    } else {
                        ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.orientation *= UnitQuaternion::from_axis_angle(&Vector3::z_axis(), -rotate_factor);
                    comp.apply();
                });
            },

            Key(Key::K, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                        ROTATE_ADJUST_BASE
                    } else {
                        ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                    };

                    render_dirty = true;
                    
                    comp.orientation *= UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -rotate_factor);
                    comp.apply();
                });
            },

            Key(Key::L, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                        ROTATE_ADJUST_BASE
                    } else {
                        ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.orientation *= UnitQuaternion::from_axis_angle(&Vector3::z_axis(), rotate_factor);
                    comp.apply();
                });
            },

            Key(Key::U, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                        ROTATE_ADJUST_BASE
                    } else {
                        ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                    };
                    
                    render_dirty = true;

                    comp.orientation *= UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -rotate_factor);
                    comp.apply();
                });
            },

            Key(Key::O, _, Action::Press, mods) => {
                selection.upgrade().map(|comp| {
                    let mut comp = comp.borrow_mut();
                    let rotate_factor = if (mods & Modifiers::Shift).is_empty() {
                        ROTATE_ADJUST_BASE
                    } else {
                        ROTATE_ADJUST_BASE * ROTATE_ADJUST_FINE
                    };

                    render_dirty = true;

                    comp.orientation *= UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rotate_factor);
                    comp.apply();
                });
            },

            _ => {},
        } }

        let mouse_projection = project_mouse(&window, &camera);

        selection.upgrade().map(|comp| {
            use na::Point2;
            use na::Matrix4;
            use alga::linear::Transformation;

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

        BOX_EDGES.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &Point3::new(1.0, 1.0, 1.0)));

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

        components.iter().for_each(|comp| {
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

        let pos = Point2::new(window.width() * 2.0 - 300.0, window.height() * 2.0 - 165.0);
        window.draw_text(&format!("iterations: {}", iteration_depth), &pos, &roboto_font, &Point3::new(0.9, 0.9, 0.9));

        let cube_count = components.len().pow(iteration_depth as u32 + 1);
        window.draw_text(&format!("cubes: {}", cube_count), &Point2::new(pos[0], pos[1] + 75.0), &roboto_font, &Point3::new(0.9, 0.9, 0.9));

        if !render_dirty {
            continue;
        }

        if cube_count > MAX_CUBES {
            // render with points
            continue;
        }

        if iteration_depth == 0 {
            iterated_group.set_visible(false);
            iterated_group.unlink();
            iterated_group = window.add_group();
            
            root_group.set_visible(true);
            
            continue;
        }

        root_group.set_visible(false);
        iterated_group.unlink();
        iterated_group = window.add_group();

        use itertools::Itertools;
        use na::{Rotation3, Matrix4};
        use palette::{LinSrgb, Blend};

        let transforms = components.iter().map(|c| c.borrow().transform().to_homogeneous());

        let colors = components.iter().map(|c| {
            let c = c.borrow();
            LinSrgb::new(c.color[0], c.color[1], c.color[2])
        });


        let transforms = (0..iteration_depth + 1)
            .map(|_| transforms.clone())
            .multi_cartesian_product()
            .map(|tsfm| tsfm.iter().product());

        let colors = (0..iteration_depth + 1)
            .map(|_| colors.clone())
            .multi_cartesian_product()
            .map(|colors| {
                let mut iter = colors.iter();
                let first = iter.next().expect("no first element in iterator");

                let result = iter.fold(*first, |acc, x| acc.multiply(*x));

                Vector3::new(result.red, result.green, result.blue)
            });

        transforms.zip(colors)
            .for_each(|(tsfm, color): (Matrix4<f32>, Vector3<f32>)| {
                let translation = na::Translation3::from_vector(tsfm.fixed_slice::<na::U3, na::U1>(0, 3).into_owned());
                
                let linear_matrix = tsfm.fixed_slice::<na::U3, na::U3>(0, 0).into_owned();
                let scale = Vector3::new(linear_matrix.fixed_rows::<na::U1>(0).norm(), linear_matrix.fixed_rows::<na::U1>(1).norm(), linear_matrix.fixed_rows::<na::U1>(2).norm());
                
                let mut rotation = linear_matrix;
                rotation.fixed_columns_mut::<na::U1>(0).apply(|x| x / scale[0]); 
                rotation.fixed_columns_mut::<na::U1>(1).apply(|x| x / scale[1]); 
                rotation.fixed_columns_mut::<na::U1>(2).apply(|x| x / scale[2]); 

                let rotation = Rotation3::from_matrix_unchecked(rotation);
                let rotation = UnitQuaternion::from_rotation_matrix(&rotation);

                let iso = Isometry3::from_parts(translation, rotation);

                let mut node = iterated_group.add_cube(scale[0], scale[1], scale[2]);
                node.set_local_transformation(iso);
                node.set_color(color[0], color[1], color[2]);
            })
    }

    Ok(())
}
