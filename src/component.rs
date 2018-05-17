use std::cmp::{Eq, PartialEq};
use std::sync::atomic::{AtomicUsize, Ordering};

use na::{
    UnitQuaternion,
    Vector3,
    Translation3,
    Point3,
    Transform3,
    Matrix4,
    Isometry3,
};

use nc::shape::Cuboid3;

use kiss3d::{
    window::Window,
    scene::SceneNode,
};

pub struct Component { // Component isn't Clone because we need SceneNodes
    pub origin: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub color: Vector3<f32>,
    pub scene_node: SceneNode,
    uid: usize,
    pub hovered: bool,
}

static UID_CTR: AtomicUsize = AtomicUsize::new(0);

impl Component {
    // don't want to impl Default because we want to always have the scene_node captured
    pub fn new(window: &mut Window) -> Self {
        let scale = Vector3::new(0.5, 0.5, 0.5);

        Component {
            origin: Vector3::identity(),
            orientation: UnitQuaternion::identity(),
            scale,
            color: Vector3::new(0.5, 1.0, 0.5),
            scene_node: window.add_cube(scale[0], scale[1], scale[2]),
            uid: UID_CTR.fetch_add(1, Ordering::Relaxed),
            hovered: false,
        }
    }

    pub fn edges(&self) -> Vec<(Point3<f32>, Point3<f32>)> {
        use alga::linear::Transformation;
        use BOX_EDGES;

        let transform = self.transform();

        BOX_EDGES.iter()
            .map(|(a, b)| (transform.transform_point(a), transform.transform_point(b)))
            .collect()
    }

    pub fn apply(&mut self) {
        use palette::{LinSrgb, Blend};

        if self.hovered {
            let selected_color = LinSrgb::new(1.0, 0.7, 0.7);
            let cur_color = LinSrgb::new(self.color[0], self.color[1], self.color[2]);
            let result = selected_color.multiply(cur_color);

            self.scene_node.set_color(result.red, result.green, result.blue);
        } else {
            self.scene_node.set_color(self.color[0], self.color[1], self.color[2]);
        }

        self.scene_node.set_local_translation(Translation3::from_vector(self.origin.clone()));
        self.scene_node.set_local_rotation(self.orientation);
        self.scene_node.set_local_scale(self.scale[0], self.scale[1], self.scale[2]);
    }

    pub fn cuboid(&self) -> Cuboid3<f32> {
        Cuboid3::new(self.scale / 2.0)
    }

    pub fn isometric_part(&self) -> Isometry3<f32> {
        Isometry3::from_parts(Translation3::from_vector(self.origin), self.orientation)
    }

    pub fn transform(&self) -> Transform3<f32> {
         Transform3::from_matrix_unchecked( self.isometric_part().to_homogeneous() * Matrix4::new_nonuniform_scaling(&self.scale))
    }
}

impl PartialEq<Component> for Component {
    #[inline]
    fn eq(&self, other: &Component) -> bool {
        self.uid == other.uid
    }

    #[inline]
    fn ne(&self, other: &Component) -> bool {
        self.uid != other.uid
    }
}

impl Eq for Component {}
