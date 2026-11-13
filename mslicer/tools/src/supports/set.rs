use anyhow::{Result, ensure};
use common::{
    serde::{Deserializer, SerdeExt, Serializer},
    units::Milimeter,
};
use nalgebra::Vector3;
use slicer::geometry::mesh::Mesh;

use crate::supports::{
    Support, SupportId,
    mesh::{FaceMap, Raft, SupportPreset, build_support_mesh, offset_points},
};

/// The position, scale and rotation of one model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
}

/// The supports of one model, in the frame of the model.
///
/// Each support point is stored relative to the position of the model. Thus
/// the supports move with the model. A change of scale or rotation makes the
/// supports wrong, so the set keeps the scale and the rotation that the
/// supports were made for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SupportSet {
    auto: Vec<Support>,
    manual: Vec<Support>,
    made_for: Option<Pose>,
}

impl Pose {
    /// Return the pose of `mesh`.
    pub fn of(mesh: &Mesh) -> Self {
        Self {
            position: mesh.position(),
            scale: mesh.scale(),
            rotation: mesh.rotation(),
        }
    }

    /// Return True if `other` has the same scale and rotation, so supports for one pose fit the other.
    pub fn same_shape(&self, other: &Pose) -> bool {
        self.scale == other.scale && self.rotation == other.rotation
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.position.serialize(ser);
        self.scale.serialize(ser);
        self.rotation.serialize(ser);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            position: Vector3::deserialize(des),
            scale: Vector3::deserialize(des),
            rotation: Vector3::deserialize(des),
        }
    }
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            scale: Vector3::repeat(1.0),
            rotation: Vector3::zeros(),
        }
    }
}

const SUPPORT_SIZE: usize = 3 * 3 * 4 + 2 * 4;

fn read_supports<T: Deserializer>(des: &mut T) -> Result<Vec<Support>> {
    let count = des.read_u32_be() as usize;
    let remaining = des.size().saturating_sub(des.pos());
    ensure!(
        count.saturating_mul(SUPPORT_SIZE) <= remaining,
        "Support count {count} does not fit in the file."
    );

    let mut supports = Vec::with_capacity(count);
    for _ in 0..count {
        let support = Support {
            id: SupportId::new(),
            points: [(); 3].map(|_| Vector3::deserialize(des)),
            tip_radius: des.read_f32_be(),
            radius: des.read_f32_be(),
        };
        let finite = support.points.iter().all(|point| point.iter().all(|x| x.is_finite()));
        ensure!(
            finite && support.tip_radius.is_finite() && support.radius.is_finite(),
            "Support has a value that is not a finite number."
        );
        ensure!(support.tip_radius > 0.0 && support.radius > 0.0, "Support has a radius of 0 or less.");
        supports.push(support);
    }
    Ok(supports)
}

fn new_support(preset: &SupportPreset, points: [Vector3<f32>; 3]) -> Support {
    Support {
        id: SupportId::new(),
        points,
        tip_radius: preset.tip_radius.get::<Milimeter>(),
        radius: preset.radius.get::<Milimeter>(),
    }
}

impl SupportSet {
    pub fn is_empty(&self) -> bool {
        self.auto.is_empty() && self.manual.is_empty()
    }

    pub fn len(&self) -> usize {
        self.auto.len() + self.manual.len()
    }

    /// Return all supports: first the automatic ones, then the manual ones.
    pub fn iter(&self) -> impl Iterator<Item = &Support> {
        self.auto.iter().chain(self.manual.iter())
    }

    pub fn get(&self, id: SupportId) -> Option<&Support> {
        self.iter().find(|support| support.id == id)
    }

    /// Return True if the supports fit a model with `pose`.
    pub fn fits(&self, pose: &Pose) -> bool {
        self.made_for.is_none_or(|made_for| made_for.same_shape(pose))
    }

    /// Replace the automatic supports with `points`, which are in world space for a model with `pose`.
    ///
    /// The manual supports stay if they fit `pose`. Otherwise the function removes them too.
    pub fn replace_auto(&mut self, preset: &SupportPreset, points: Vec<[Vector3<f32>; 3]>, pose: Pose) {
        if !self.fits(&pose) {
            self.manual.clear();
        }
        self.auto = points
            .into_iter()
            .map(|world| new_support(preset, offset_points(world, -pose.position)))
            .collect();
        self.made_for = Some(pose);
    }

    /// Add one manual support with `points` in world space, for a model with `pose`.
    pub fn add_manual(&mut self, preset: &SupportPreset, points: [Vector3<f32>; 3], pose: Pose) {
        if !self.fits(&pose) {
            self.clear();
        }
        self.manual.push(new_support(preset, offset_points(points, -pose.position)));
        self.made_for = Some(pose);
    }

    pub fn remove(&mut self, id: SupportId) {
        self.auto.retain(|support| support.id != id);
        self.manual.retain(|support| support.id != id);
    }

    pub fn clear(&mut self) {
        self.auto.clear();
        self.manual.clear();
        self.made_for = None;
    }

    /// Return the mesh of all supports for a model with `pose`, or `None` if there are no supports.
    ///
    /// The mesh has the position of the model and no scale or rotation. The
    /// supports go down to the build plate, which is at `-pose.position.z` in
    /// the frame of the model.
    pub fn build_mesh(&self, pose: &Pose, resolution: u32, raft: Raft) -> Option<(Mesh, FaceMap)> {
        let (mut mesh, map) = build_support_mesh(self.iter(), -pose.position.z, resolution, raft)?;
        mesh.set_position(pose.position);
        Some((mesh, map))
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        match &self.made_for {
            Some(pose) => {
                ser.write_bool(true);
                pose.serialize(ser);
            }
            None => ser.write_bool(false),
        }
        for list in [&self.auto, &self.manual] {
            ser.write_u32_be(list.len() as u32);
            for support in list {
                support.points.iter().for_each(|point| point.serialize(ser));
                ser.write_f32_be(support.tip_radius);
                ser.write_f32_be(support.radius);
            }
        }
    }

    /// Read a support set that [`SupportSet::serialize`] wrote.
    ///
    /// The function returns an error if a support count does not fit in the
    /// rest of the data, or if a value is not a finite number. Thus a damaged
    /// file cannot make the app reserve a very large amount of memory.
    pub fn deserialize<T: Deserializer>(des: &mut T) -> Result<Self> {
        let made_for = des.read_bool().then(|| Pose::deserialize(des));
        let auto = read_supports(des)?;
        let manual = read_supports(des)?;
        Ok(Self {
            auto,
            manual,
            made_for,
        })
    }
}
