use crate::{
    NAME,
    ROBOTO_TTF,
    VERSION,
};
use glfw::MouseButton::*;
use kiss3d::{
    camera::ArcBall,
    light::Light,
    text::Font,
    window::Window,
};
use std::{
    cell::RefCell,
    default::Default,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub(super) struct RenderState {
    pub wireframes_enabled: bool,
    pub window: Window,
    pub camera: ArcBall,
    pub font: Rc<Font>,
    pub point_set: Vec<(Point3<f32>, Point3<f32>)>,
    pub dirty: bool,
}

impl RenderState {
    pub fn project_mouse(&self) -> Ray3<f32> {
        let (x, y) = self.window.glfw_window().get_cursor_pos();

        let (loc, dir) = self.camera.unproject(
            &Point2::new(x as f32, y as f32),
            &Vector2::new(self.window.width(), self.window.height())
        );

        Ray3::new(loc, dir)
    }
}

impl Default for RenderState {
    fn default() -> Self {
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



        RenderState {
            wireframes_enabled: true,
            window,
            camera,
            font: roboto_font,
            point_set: Vec::new(),
            dirty: false,
        }
    }
}