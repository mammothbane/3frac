use na::Point3;

pub static ROBOTO_TTF: &'static [u8] = include_bytes!("resources/roboto.ttf");

lazy_static! {
    pub static ref BOX_EDGES: Vec<(Point3<f32>, Point3<f32>)> = {
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

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const NAME: &'static str = env!("CARGO_PKG_NAME");

pub const SELECTION_BBOX_SCALE: f32 = 1.1;

pub const TRANSLATE_ADJUST_BASE: f32 = 0.1;
pub const TRANSLATE_ADJUST_FINE: f32 = 0.1;

pub const ROTATE_ADJUST_BASE: f32 = (2.0 * std::f32::consts::PI) / 24.0;
pub const ROTATE_ADJUST_FINE: f32 = 1.0 / 12.0;

pub const SCALE_ADJUST_BASE: f32 = 0.06;
pub const SCALE_ADJUST_FINE: f32 = 0.25;

pub const COLOR_ADJUST_BASE: f32 = 2.0;
pub const COLOR_ADJUST_FINE: f32 = 0.25;

pub const MAX_CUBES: usize = 2_000;
