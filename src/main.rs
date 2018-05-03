extern crate kiss3d;
extern crate nalgebra as na;
extern crate failure;
extern crate glfw;
#[macro_use] extern crate itertools;

use kiss3d::{
    window::Window,
    light::Light,
    camera::ArcBall,
};
use na::{UnitQuaternion, Vector3, Point3};

pub(crate) type Result<T> = std::result::Result<T, failure::Error>;

fn run() -> Result<()> {
    let mut window = Window::new("fractal");
//    let mut c = window.add_cube(1.0, 1.0, 1.0);

    let box_color = Point3::new(0.8, 0.8, 0.8);

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
        .filter(|(p1, p2)| (*p1 - *p2).norm() == 1.0)
        .collect::<Vec<_>>();

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        use glfw::WindowEvent::*;

        window.events().iter().for_each(|evt| match evt.value {

            _ => {},
        });

        lines.iter().for_each(|(p1, p2)| window.draw_line(p1, p2, &box_color));
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
