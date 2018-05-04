extern crate kiss3d;
extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate failure;
extern crate glfw;
extern crate itertools;
#[macro_use] extern crate lazy_static;

use std::{
    rc::Rc,
    cell::RefCell,
};

use kiss3d::{
    window::Window,
    light::Light,
    camera::{ArcBall, Camera},
};

use na::{
    Id,
    Vector3,
    Point3,
    Vector2,
    Point2,
};

use nc::{
    query::Ray3,
    shape::Cuboid3,
};

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


fn run() -> Result<()> {
    let mut window = Window::new("fractal");

    let mut camera = ArcBall::new(Point3::new(0.0f32, 0.0, -1.0), Point3::origin());
    camera.rebind_drag_button(Some(MouseButtonMiddle));
    camera.rebind_rotate_button(Some(MouseButtonRight));

    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(70));

    let mut rayline: Option<(Point3<f32>, Point3<f32>)> = None;

    let origin_cube = Cuboid3::new(Vector3::new(0.5, 0.5, 0.5));

    let non_collision_color = Point3::new(1.0, 1.0, 1.0);
    let collision_color = Point3::new(1.0, 0.7, 0.7);

    let mut components = Vec::<Rc<RefCell<Component>>>::new();

    while window.render_with_camera(&mut camera) {
        use glfw::WindowEvent::*;
        use glfw::Key;

        window.events().iter().for_each(|ref evt| match evt.value {
            ref evt @ Scroll(_, _) => {
                camera.handle_event(window.glfw_window(), &evt)

            },
            MouseButton(MouseButtonLeft, Action::Press, _) => {
                let (x, y) = window.glfw_window().get_cursor_pos();

                let (loc, dir) = camera.unproject(
                    &Point2::new(x as f32, y as f32),
                    &Vector2::new(window.width(), window.height())
                );

                let _ray = Ray3::new(loc, dir);

                rayline = Some((loc, loc + dir.normalize() * 10.0));
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
        });

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
            Some((idx, _)) => {
                let mut comp = components[idx].borrow_mut();
                comp.color = collision_color.coords.clone();
                comp.apply();

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
            },
        }

        rayline.map(|(ref p1, ref p2)| {
            window.draw_line(p1, p2, &Point3::new(1.0, 1.0, 1.0));
        });

        BOX_EDGES.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &non_collision_color));
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
