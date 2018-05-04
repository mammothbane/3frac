use na::{
    UnitQuaternion,
    Vector3,
    Translation3,
    Point3,
    Isometry3,
};

use nc::shape::Cuboid3;

use kiss3d::{
    window::Window,
    scene::SceneNode,
};

#[derive(Clone)]
pub struct Component {
    pub origin: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub color: Vector3<f32>,
    pub scene_node: SceneNode,
}


impl Component {
    // don't want to impl Default because we want to always have the scene_node captured
    pub fn new(window: &mut Window) -> Self {
        let scale = Vector3::new(1.0, 1.0, 1.0);

        Component {
            origin: Vector3::identity(),
            orientation: UnitQuaternion::identity(),
            scale,
            color: Vector3::new(1.0, 1.0, 1.0),
            scene_node: window.add_cube(scale[0], scale[1], scale[2]),
        }
    }

    pub fn edges(&self) -> Vec<(Point3<f32>, Point3<f32>)> {
        use BOX_EDGES;

        BOX_EDGES.iter()
            .map(|(a, b)| {
                let a = Point3::from_coordinates(a.coords.component_mul(&self.scale));
                let b = Point3::from_coordinates(b.coords.component_mul(&self.scale));
                (a, b)
            })
            .collect()
    }

    pub fn apply(&mut self) {
        self.scene_node.set_local_translation(Translation3::from_vector(self.origin.clone()));
        self.scene_node.set_color(self.color[0], self.color[1], self.color[2]);
        self.scene_node.set_local_rotation(self.orientation);
        self.scene_node.set_local_scale(self.scale[0], self.scale[1], self.scale[2]);
    }

    pub fn cuboid(&self) -> Cuboid3<f32> {
        Cuboid3::new(self.scale / 2.0)
    }

    pub fn cuboid_transform(&self) -> Isometry3<f32> {
        Isometry3::from_parts(Translation3::from_vector( self.origin), self.orientation)
    }
}