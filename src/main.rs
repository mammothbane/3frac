extern crate kiss3d;
extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate failure;
extern crate glfw;
#[macro_use] extern crate itertools;

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
    Point2
};

use nc::{
    query::Ray3,
    shape::Cuboid3,
};

use glfw::{MouseButtonMiddle, MouseButtonRight, MouseButtonLeft, Action};

pub(crate) type Result<T> = std::result::Result<T, failure::Error>;

fn run() -> Result<()> {
    let mut window = Window::new("fractal");

    let mut camera = ArcBall::new(Point3::new(0.0f32, 0.0, -1.0), Point3::origin());
    camera.rebind_drag_button(Some(MouseButtonMiddle));
    camera.rebind_rotate_button(Some(MouseButtonRight));

    let points = [
        Point3::new(0.5, 0.5, 0.5),
        Point3::new(-0.5, 0.5, 0.5),
        Point3::new(0.5, -0.5, 0.5),
        Point3::new(0.5, 0.5, -0.5),
        Point3::new(-0.5, -0.5, 0.5),
        Point3::new(-0.5, 0.5, -0.5),
        Point3::new(0.5, -0.5, -0.5),
        Point3::new(-0.5, -0.5, -0.5),
    ];

    let lines = iproduct!(points.iter(), points.iter())
        .filter(|(p1, p2)| (*p1 - *p2).norm() == 1.0)  // this is messy
        .collect::<Vec<_>>();

    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(70));

    let mut rayline: Option<(Point3<f32>, Point3<f32>)> = None;

    let origin_cube = Cuboid3::new(Vector3::new(0.5, 0.5, 0.5));

    let non_collision_color = Point3::new(0.8, 0.8, 0.8);
    let collision_color = Point3::new(1.0, 0.7, 0.7);

    let mut box_color = &non_collision_color;

    while window.render_with_camera(&mut camera) {
        use glfw::WindowEvent::*;

        window.events().iter().for_each(|ref evt| match evt.value {
            ref evt @ Scroll(_, _) => { camera.handle_event(window.glfw_window(), &evt) },
            MouseButton(MouseButtonLeft, Action::Press, _) => {
                let (x, y) = window.glfw_window().get_cursor_pos();

                let (loc, dir) = camera.unproject(
                    &Point2::new(x as f32, y as f32),
                    &Vector2::new(window.width(), window.height())
                );

                let _ray = Ray3::new(loc, dir);

                rayline = Some((loc, loc + dir.normalize() * 5.0));
            },

            CursorPos(x, y) => {
                use nc::query::RayCast;

                let (loc, dir) = camera.unproject(
                    &Point2::new(x as f32, y as f32),
                    &Vector2::new(window.width(), window.height())
                );

                let ray = Ray3::new(loc, dir);

                if origin_cube.toi_with_ray(&Id::new(), &ray, true).is_some() {
                    box_color = &collision_color;
                } else {
                    box_color = &non_collision_color;
                }
            },

            _ => {},
        });

        rayline.map(|(ref p1, ref p2)| {
            window.draw_line(p1, p2, &Point3::new(1.0, 1.0, 1.0));
        });

        lines.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, box_color));
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
