use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use common::{
    progress::Progress,
    serde::{DynamicSerializer, SliceDeserializer},
    slice::profiles::halot_one_plus_size,
};
use nalgebra::Vector3;
use slicer::geometry::{
    mesh::{HalfEdgeMesh, Mesh},
    partitioning::Bvh,
};
use tools::supports::{
    auto::{AutoPlacement, PlacementConfig},
    mesh::{Raft, SupportPreset},
    set::{Pose, SupportSet},
};

/// Place automatic supports under a mesh and write the mesh, the supports and the raft as one STL file.
#[derive(Parser)]
struct Args {
    /// Mesh to support, as .stl or .obj.
    input: PathBuf,
    /// STL file to write.
    output: PathBuf,
    /// Support preset.
    #[arg(long, value_enum, default_value_t = Preset::Medium)]
    preset: Preset,
    /// Height of the bottom of the model above the build plate, in mm.
    #[arg(long, default_value_t = 5.0)]
    lift: f32,
    /// Height of the raft in mm. 0 gives no raft.
    #[arg(long, default_value_t = 1.0)]
    raft_height: f32,
    /// Extra width of the raft bottom around the support feet, in mm.
    #[arg(long, default_value_t = 1.0)]
    raft_offset: f32,
    /// Number of sides of each support cylinder.
    #[arg(long, default_value_t = 16)]
    resolution: u32,
}

#[derive(Clone, Copy, ValueEnum)]
enum Preset {
    Light,
    Medium,
    Heavy,
}

impl Preset {
    fn support_preset(self) -> SupportPreset {
        match self {
            Preset::Light => SupportPreset::LIGHT,
            Preset::Medium => SupportPreset::MEDIUM,
            Preset::Heavy => SupportPreset::HEAVY,
        }
    }
}

fn load(path: &PathBuf) -> Result<Mesh> {
    let extension = path.extension().context("Input has no extension")?.to_string_lossy();
    let format = mesh_format::Format::from_extension(&extension).context("Unknown mesh extension")?;
    let data = fs::read(path)?;
    let mesh = mesh_format::load_mesh(SliceDeserializer::new(&data), format, &Progress::new())?;
    Ok(Mesh::new_boxed_centered(mesh.verts, mesh.faces))
}

/// Return one mesh with the vertices of all `meshes` in world space.
fn combine(meshes: &[&Mesh]) -> mesh_format::Mesh {
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    for mesh in meshes {
        let first = verts.len() as u32;
        verts.extend(mesh.vertices().iter().map(|vertex| mesh.transform(vertex)));
        faces.extend(mesh.faces().iter().map(|face| face.map(|i| i + first)));
    }
    mesh_format::Mesh {
        verts: verts.into_boxed_slice(),
        faces: faces.into_boxed_slice(),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let preset = args.preset.support_preset();

    let mut mesh = load(&args.input)?;
    mesh.set_position(Vector3::new(0.0, 0.0, args.lift));
    let bvh = Bvh::build(mesh.inner(), Progress::new());
    let half_edge = HalfEdgeMesh::build_from_mesh(mesh.inner());

    let placement = AutoPlacement::new(PlacementConfig::default(), preset.clone(), halot_one_plus_size());
    let points = placement.generate_supports(&mesh, &half_edge, &bvh, &Progress::new());

    let pose = Pose::of(&mesh);
    let mut set = SupportSet::default();
    set.replace_auto(&preset, points, pose);
    let raft = Raft {
        height: args.raft_height,
        offset: args.raft_offset,
    };
    let (supports, _) = set.build_mesh(&pose, args.resolution, raft).context("No supports were placed")?;

    let combined = combine(&[&mesh, &supports]);
    let mut ser = DynamicSerializer::new();
    mesh_format::save_mesh(&mut ser, mesh_format::Format::Stl, &Progress::new(), &combined);
    fs::write(&args.output, ser.into_inner())?;

    println!("Placed {} supports. Wrote {}.", set.len(), args.output.display());
    Ok(())
}
