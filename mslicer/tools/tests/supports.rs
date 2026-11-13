use common::{
    progress::Progress,
    serde::{DynamicSerializer, SliceDeserializer},
    units::{Milimeter, Milimeters},
};
use nalgebra::{Vector2, Vector3};
use slicer::geometry::{
    bounds::Bounds3D,
    mesh::{HalfEdgeMesh, Mesh, MeshBuilder},
    partitioning::Bvh,
};
use tools::supports::{
    auto::{AutoPlacement, PlacementConfig},
    mesh::{Raft, SupportPreset, build_raft, build_support_mesh},
    set::{Pose, SupportSet},
};

/// Return the volume of a closed mesh. The volume is negative if the faces point inward.
fn signed_volume(mesh: &Mesh) -> f32 {
    mesh.faces()
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|i| mesh.transform(&mesh.vertices()[i as usize]));
            a.dot(&b.cross(&c)) / 6.0
        })
        .sum()
}

/// Return the corners of a square of `size` millimeters around the origin.
fn square(size: f32) -> Vec<Vector2<f32>> {
    let h = size / 2.0;
    vec![
        Vector2::new(-h, -h),
        Vector2::new(h, -h),
        Vector2::new(h, h),
        Vector2::new(-h, h),
    ]
}

/// Return a closed cube mesh with sides of `size` millimeters, with its bottom at Z = 0.
fn cube_mesh(size: f32) -> Mesh {
    let h = size / 2.0;
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

fn sample_points() -> [Vector3<f32>; 3] {
    [
        Vector3::new(1.0, 2.0, 10.0),
        Vector3::new(1.0, 2.0, 8.0),
        Vector3::new(1.5, 2.0, 6.0),
    ]
}

fn pose_at(position: Vector3<f32>) -> Pose {
    Pose {
        position,
        ..Default::default()
    }
}

#[test]
fn presets_get_stronger_from_light_to_heavy() {
    let [(_, light), (_, medium), (_, heavy)] = SupportPreset::NAMED;
    let tips = [light.tip_radius, medium.tip_radius, heavy.tip_radius].map(|r| r.get::<Milimeter>());
    let bodies = [light.radius, medium.radius, heavy.radius].map(|r| r.get::<Milimeter>());
    assert!(tips[0] < tips[1] && tips[1] < tips[2], "Tips {tips:?} do not get thicker.");
    assert!(bodies[0] < bodies[1] && bodies[1] < bodies[2], "Bodies {bodies:?} do not get thicker.");
}

#[test]
fn preset_name_is_none_for_custom_values() {
    assert_eq!(SupportPreset::HEAVY.name(), Some("Heavy"));
    let mut custom = SupportPreset::MEDIUM;
    custom.tip_radius = Milimeters::new(0.25);
    assert_eq!(custom.name(), None);
}

#[test]
fn raft_is_a_closed_solid_with_outward_faces_and_a_wider_bottom() {
    let raft = Raft {
        height: 1.0,
        offset: 2.0,
    };
    let mut builder = MeshBuilder::new();
    build_raft(raft, 0.0, &square(10.0), &mut builder);
    let mesh = builder.build();

    let bounds = Bounds3D::from_mesh(&mesh);
    assert!((bounds.min.z - 0.0).abs() < 1e-5 && (bounds.max.z - 1.0).abs() < 1e-5);
    assert!(bounds.max.x > 5.0 + 1.0 && bounds.max.x <= 5.0 + 2.0 + 1e-4);

    let volume = signed_volume(&mesh);
    assert!(volume > 10.0 * 10.0 * 1.0, "Volume {volume} is not larger than the top area times the height.");
}

#[test]
fn raft_without_height_adds_nothing() {
    let mut builder = MeshBuilder::new();
    build_raft(Raft { height: 0.0, offset: 1.0 }, 0.0, &square(10.0), &mut builder);
    assert!(builder.is_empty());
}

#[test]
fn support_mesh_reaches_the_plate_and_maps_each_support_to_its_faces() {
    let mut set = SupportSet::default();
    set.add_manual(&SupportPreset::MEDIUM, sample_points(), Pose::default());
    set.add_manual(&SupportPreset::LIGHT, sample_points().map(|p| p + Vector3::x() * 5.0), Pose::default());

    let no_raft = Raft { height: 0.0, offset: 0.0 };
    let (mesh, map) = build_support_mesh(set.iter(), 0.0, 8, no_raft).unwrap();
    let bounds = Bounds3D::from_mesh(&mesh);
    assert!(bounds.min.z.abs() < 1e-4, "Supports end at {} and not on the plate.", bounds.min.z);
    assert_eq!(map.len(), 2);
    assert!(map.values().all(|range| range.end > range.start));
}

#[test]
fn supports_move_with_the_model() {
    let mut set = SupportSet::default();
    let made_at = pose_at(Vector3::new(10.0, 0.0, 5.0));
    set.add_manual(&SupportPreset::MEDIUM, sample_points(), made_at);

    let moved = pose_at(Vector3::new(30.0, -4.0, 5.0));
    assert!(set.fits(&moved));
    let (mesh, _) = set.build_mesh(&moved, 8, Raft::default()).unwrap();
    let tip = mesh.transform(&set.iter().next().unwrap().points[0]);
    let expected = sample_points()[0] + Vector3::new(20.0, -4.0, 0.0);
    assert!((tip - expected).norm() < 1e-4, "Tip is at {tip}, expected {expected}.");
}

#[test]
fn raised_model_gets_longer_supports_down_to_the_plate() {
    let mut set = SupportSet::default();
    set.add_manual(&SupportPreset::MEDIUM, sample_points(), pose_at(Vector3::new(0.0, 0.0, 5.0)));

    let raised = pose_at(Vector3::new(0.0, 0.0, 9.0));
    let (mesh, _) = set.build_mesh(&raised, 8, Raft { height: 0.0, offset: 0.0 }).unwrap();
    let bounds = Bounds3D::from_mesh(&mesh);
    assert!(bounds.min.z.abs() < 1e-4, "Supports end at {} and not on the plate.", bounds.min.z);
}

#[test]
fn scaled_or_rotated_model_does_not_fit_its_supports() {
    let mut set = SupportSet::default();
    set.add_manual(&SupportPreset::MEDIUM, sample_points(), Pose::default());

    let scaled = Pose { scale: Vector3::repeat(2.0), ..Default::default() };
    let rotated = Pose { rotation: Vector3::new(0.0, 0.0, 1.0), ..Default::default() };
    assert!(!set.fits(&scaled));
    assert!(!set.fits(&rotated));

    set.add_manual(&SupportPreset::MEDIUM, sample_points(), scaled);
    assert_eq!(set.len(), 1, "The old support for the other scale is still there.");
}

#[test]
fn support_set_survives_saving_and_loading() {
    let mut set = SupportSet::default();
    let pose = pose_at(Vector3::new(3.0, 4.0, 5.0));
    set.replace_auto(&SupportPreset::HEAVY, vec![sample_points()], pose);
    set.add_manual(&SupportPreset::LIGHT, sample_points().map(|p| p * 2.0), pose);

    let mut ser = DynamicSerializer::new();
    set.serialize(&mut ser);
    let bytes = ser.into_inner();
    let read = SupportSet::deserialize(&mut SliceDeserializer::new(&bytes));

    assert_eq!(read.len(), 2);
    for (a, b) in set.iter().zip(read.iter()) {
        assert_eq!((a.points, a.tip_radius, a.radius), (b.points, b.tip_radius, b.radius));
    }
    assert!(read.fits(&pose));
    assert!(!read.fits(&Pose { scale: Vector3::repeat(3.0), ..pose }));
}

#[test]
fn auto_placement_supports_the_bottom_of_a_raised_cube() {
    let mut mesh = cube_mesh(10.0);
    mesh.set_position(Vector3::new(0.0, 0.0, 5.0));
    let bvh = Bvh::build(mesh.inner(), Progress::new());
    let half_edge = HalfEdgeMesh::build_from_mesh(mesh.inner());

    let platform = Vector3::new(172.8, 102.4, 160.0).map(Milimeters::new);
    let placement = AutoPlacement::new(PlacementConfig::default(), SupportPreset::MEDIUM, platform);
    let supports = placement.generate_supports(&mesh, &half_edge, &bvh, &Progress::new());

    assert!(!supports.is_empty());
    let tip_length = SupportPreset::MEDIUM.tip_length.get::<Milimeter>();
    for [tip, body_top, body_bottom] in supports {
        assert!((tip.z - 5.0).abs() < 1e-3, "Tip at {tip} is not on the bottom face.");
        assert!(body_top.z < tip.z, "Tip at {tip} does not point down.");
        assert!(((tip - body_top).norm() - tip_length).abs() < 1e-3);
        assert!(body_bottom.z <= body_top.z && body_bottom.z >= 0.0);
    }
}
