#![feature(vec_remove_item)]
#![feature(extern_prelude)]

extern crate alga;
extern crate failure;
extern crate glfw;
extern crate itertools;
extern crate kiss3d;
#[macro_use] extern crate lazy_static;
extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate palette;

use alga::linear::Transformation;
use failure::Fallible;
use glfw::{Action, MouseButtonLeft, MouseButtonMiddle, MouseButtonRight};
use kiss3d::{
    camera::{ArcBall, Camera},
    light::Light,
    text::Font,
    window::Window,
};
use na::{
    Isometry3,
    Point2,
    Point3,
    UnitQuaternion,
    Vector2,
    Vector3,
};
use nc::query::{Ray3, RayCast};
use self::component::Component;
pub use self::constants::*;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};


mod component;
mod input;
mod constants;
mod state;


fn main() -> Fallible<()> {
    while window.render_with_camera(&mut camera) {
        use glfw::WindowEvent::*;
        use glfw::{Key, Modifiers};

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


        let mouse_projection = project_mouse(&window, &camera);

        point_set.iter().for_each(|(pt, color)| window.draw_point(pt, color));

        if !render_dirty {
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
        point_set.clear();
        iterated_group.unlink();
        iterated_group = window.add_group();
        iterated_group.enable_backface_culling(true);

        use itertools::Itertools;
        use na::{Rotation3, Matrix4};
        use palette::{LinSrgb, RgbHue, Hsl};

        let transforms = components.iter().map(|c| c.borrow().transform().to_homogeneous());

        let (lightness, saturation) = {
            let comp = components[0].borrow();

            let hsl: Hsl<_, _> = LinSrgb::new(comp.color[0], comp.color[1], comp.color[2]).into();

            (hsl.lightness, hsl.saturation)
        };

        let colors = components.iter().map(|c| {
            let c = c.borrow();
            let hsl: Hsl<_, _> = LinSrgb::new(c.color[0], c.color[1], c.color[2]).into();
            let hue_angle = hsl.hue.to_positive_radians();

            (hue_angle.sin(), hue_angle.cos())
        });


        let transforms = (0..iteration_depth + 1)
            .map(|_| transforms.clone())
            .multi_cartesian_product()
            .map(|tsfm| tsfm.iter().product());

        let colors = (0..iteration_depth + 1)
            .map(|_| colors.clone())
            .multi_cartesian_product()
            .map(|colors| {
                let (y, x) = colors.iter().fold((0.0f32, 0.0f32), |(acc1, acc2), (y, x)| (acc1 + y, acc2 + x));

                let hsl = Hsl::new(RgbHue::from_radians(y.atan2(x)), saturation, lightness);
                let rgb: LinSrgb<_> = hsl.into();

                Vector3::new(rgb.red, rgb.green, rgb.blue)
            });

        let origin = Point3::origin();
        transforms.zip(colors)
            .for_each(|(tsfm, color): (Matrix4<f32>, Vector3<f32>)| {
                if cube_count > MAX_CUBES {
                    point_set.push((tsfm.transform_point(&origin), Point3::from_coordinates(color)));

                    return;
                }

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
