use std::{mem, time::Instant};

use common::color::SRgb;
use nalgebra::Vector3;
use slicer::geometry::mesh::Mesh;
use tools::supports::{
    Support, SupportId,
    mesh::{FaceMap, Raft, SupportPreset},
    set::{Pose, SupportSet},
};
use tracing::info;
use wgpu::Device;

use crate::{core::state::PreviewModel, render::util::MeshBuffers};

/// The supports of one model, with the cached support mesh and its GPU buffers.
#[derive(Default)]
pub struct Supports {
    set: SupportSet,
    pose: Pose,

    resolution: u32,
    raft: Raft,
    mesh: Option<(Mesh, FaceMap)>,
    buffers: Option<MeshBuffers>,
}

impl Supports {
    /// Return an empty support list for a model with `pose`.
    pub fn new(pose: Pose) -> Self {
        Self {
            pose,
            ..Default::default()
        }
    }

    pub fn set_resolution(&mut self, resolution: u32) {
        if self.resolution != resolution {
            self.resolution = resolution;
            self.invalidate_cache();
        }
    }

    pub fn set_raft(&mut self, raft: Raft) {
        if self.raft != raft {
            self.raft = raft;
            self.invalidate_cache();
        }
    }

    pub fn invalidate_cache(&mut self) {
        self.mesh.take();
        self.buffers.take();
    }

    pub fn clear(&mut self) {
        self.set.clear();
        self.invalidate_cache();
    }

    pub fn get(&self, id: SupportId) -> Option<&Support> {
        self.set.get(id)
    }

    pub fn support_set(&self) -> &SupportSet {
        &self.set
    }

    /// Replace all supports with `set`, for example after the app loads a project.
    pub fn restore(&mut self, set: SupportSet) {
        self.set = if set.fits(&self.pose) { set } else { SupportSet::default() };
        self.invalidate_cache();
    }

    /// Replace the automatic supports with `supports`, which the generator made in world space for a model with `made_for`.
    pub fn replace_auto(
        &mut self,
        config: &SupportPreset,
        supports: Vec<[Vector3<f32>; 3]>,
        made_for: Pose,
    ) {
        self.set.replace_auto(config, supports, made_for);
        if !self.set.fits(&self.pose) {
            self.set.clear();
        }
        self.invalidate_cache();
    }

    /// Add one manual support with `support` in world space, for the current pose of the model.
    pub fn add_manual(&mut self, config: &SupportPreset, support: [Vector3<f32>; 3]) {
        self.set.add_manual(config, support, self.pose);
        self.invalidate_cache();
    }

    pub fn mesh(&mut self) -> &Option<(Mesh, FaceMap)> {
        if self.mesh.is_none() && !self.set.is_empty() {
            let start = Instant::now();
            self.mesh = self.set.build_mesh(&self.pose, self.resolution, self.raft);
            if let Some((mesh, _)) = &self.mesh {
                let (faces, duration) = (mesh.face_count(), start.elapsed());
                info!("Generated support mesh with {faces} faces in {duration:?}");
            }
        }

        &self.mesh
    }

    pub fn support_for_face(&mut self, face: u32) -> Option<SupportId> {
        let (_, map) = self.mesh().as_ref()?;
        (map.iter().find(|(_, r)| r.contains(&face))).map(|(id, _)| *id)
    }

    pub fn get_buffers(&mut self, device: &Device) -> &Option<MeshBuffers> {
        if self.buffers.is_none()
            && let Some((mesh, _)) = self.mesh()
        {
            self.buffers = Some(MeshBuffers::get_for(device, mesh));
        }

        &self.buffers
    }

    pub fn try_get_buffers(&self) -> Option<(&MeshBuffers, u32)> {
        self.buffers.as_ref().map(|x| {
            let face_count = self.mesh.as_ref().unwrap().0.face_count() as u32 * 3;
            (x, face_count)
        })
    }

    /// Follow a new position, scale or rotation of the model.
    ///
    /// A move on X or Y only moves the support mesh. A move on Z makes the
    /// columns longer or shorter, so the mesh is built again. A new scale or
    /// rotation removes the supports, because they do not fit the model
    /// anymore.
    pub fn set_transform(
        &mut self,
        position: Vector3<f32>,
        scale: Vector3<f32>,
        rotation: Vector3<f32>,
    ) {
        let pose = Pose {
            position,
            scale,
            rotation,
        };
        if pose == self.pose {
            return;
        }

        let old = mem::replace(&mut self.pose, pose);
        if !self.set.fits(&pose) {
            self.clear();
        } else if old.position.z != pose.position.z {
            self.invalidate_cache();
        } else if let Some((mesh, _)) = &mut self.mesh {
            mesh.set_position(pose.position);
        }
    }

    pub fn remove(&mut self, id: SupportId) {
        self.set.remove(id);
        self.invalidate_cache();
    }

    pub fn preview(&mut self, device: &Device, color: SRgb<f32>) -> Option<PreviewModel> {
        if let Some((faces, transform)) = (self.mesh().as_ref())
            .map(|(mesh, _)| (mesh.face_count(), *mesh.transformation_matrix()))
            && let Some(buffers) = self.get_buffers(device)
        {
            return Some(PreviewModel {
                mesh: buffers.clone(),
                faces,
                transform,
                color,
            });
        }

        None
    }
}
