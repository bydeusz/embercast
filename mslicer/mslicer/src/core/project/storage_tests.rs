use common::{
    progress::Progress,
    serde::{DynamicSerializer, SliceDeserializer},
    units::{Milimeter, Milimeters},
};
use nalgebra::Vector3;
use slicer::geometry::mesh::Mesh;
use tools::supports::mesh::SupportPreset;

use crate::core::project::{Project, model::Model};

/// Return a closed cube mesh with sides of 10 millimeters.
fn cube_mesh() -> Mesh {
    let h = 5.0;
    let vertices: Vec<Vector3<f32>> = [
        (-h, -h, -h),
        (h, -h, -h),
        (h, h, -h),
        (-h, h, -h),
        (-h, -h, h),
        (h, -h, h),
        (h, h, h),
        (-h, h, h),
    ]
    .into_iter()
    .map(|(x, y, z)| Vector3::new(x, y, z))
    .collect();
    let faces: Vec<[u32; 3]> = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    Mesh::new_boxed_centered(vertices.into(), faces.into())
}

/// Return a project with one cube at 5 mm above the plate and two manual supports under it.
fn project_with_supports() -> Project {
    let mut project = Project::default();
    project.support_config.preset = SupportPreset::HEAVY;
    project.support_config.raft_height = Milimeters::new(1.5);

    let mut mesh = cube_mesh();
    mesh.set_position(Vector3::new(4.0, -2.0, 5.0));
    let mut model = Model::from_mesh(mesh);
    for x in [0.0, 3.0] {
        let tip = Vector3::new(4.0 + x, -2.0, 5.0);
        let support = [tip, tip - Vector3::z() * 2.0, tip - Vector3::z() * 3.0];
        model.supports.add_manual(&SupportPreset::HEAVY, support);
    }
    project.models.push(model);
    project
}

fn save_and_load(project: &Project) -> Project {
    let mut ser = DynamicSerializer::new();
    project.serialize(&mut ser, Progress::new());
    let bytes = ser.into_inner();
    Project::deserialize(&mut SliceDeserializer::new(&bytes), Progress::new()).unwrap()
}

#[test]
fn supports_and_support_settings_survive_saving_and_loading() {
    let project = project_with_supports();
    let loaded = save_and_load(&project);

    assert_eq!(loaded.support_config.preset, SupportPreset::HEAVY);
    assert_eq!(loaded.support_config.raft_height.get::<Milimeter>(), 1.5);

    let (saved, read) = (project.models[0].supports.support_set(), loaded.models[0].supports.support_set());
    assert_eq!(read.len(), 2);
    for (a, b) in saved.iter().zip(read.iter()) {
        assert_eq!(a.points, b.points);
        assert_eq!((a.tip_radius, a.radius), (b.tip_radius, b.radius));
    }
}

#[test]
fn loaded_supports_still_reach_the_build_plate() {
    let mut loaded = save_and_load(&project_with_supports());
    let (mesh, _) = loaded.models[0].supports.mesh().as_ref().unwrap();
    let lowest = (mesh.vertices().iter())
        .map(|vertex| mesh.transform(vertex).z)
        .fold(f32::MAX, f32::min);
    assert!(lowest.abs() < 1e-4, "Supports end at {lowest} and not on the plate.");
}
