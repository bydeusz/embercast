use std::collections::HashSet;

use common::container::ArrayCluster;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use nalgebra::{Vector2, Vector3};
use slicer::geometry::{
    Ray,
    mesh::{HalfEdgeMesh, Mesh},
    primitive,
    triangle::triangle_intersection,
};

use crate::supports::auto::{AutoPlacement, quazirandom::quazirandom_rect_2d};

pub struct SupportPlacement {
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl AutoPlacement {
    pub fn overhanging_faces(&self, mesh: &Mesh) -> Vec<(usize, Vector3<f32>)> {
        let max_angle = self.placement.max_angle.to_radians();
        let mut overhangs = Vec::new();

        for face in 0..mesh.face_count() {
            let normal = mesh.transform_normal(&mesh.normal(face));

            let angle = (-normal.z).acos();
            (angle <= max_angle).then(|| overhangs.push((face, normal)));
        }

        overhangs
    }

    // Find edges that border two faces with different angles, at least one of
    // the faces must be overhanging.
    pub fn overhanging_edges(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
        overhangs: &[(usize, Vector3<f32>)],
    ) -> Vec<HashSet<u32>> {
        let mut cluster = ArrayCluster::new(mesh.vertex_count());

        for (face, normal) in overhangs.iter() {
            let verts = mesh.face(*face);
            for i in 0..3 {
                let Some(edge) = half_edge.edge_for((verts[i], verts[(i + 1) % 3])) else {
                    continue;
                };

                let Some(twin) = edge.twin else { continue }; // ✌️
                let neighbor = half_edge.get_edge(twin).face as usize;

                let neighbor_normal = mesh.transform_normal(&mesh.normal(neighbor));
                let angle_diff = normal.angle(&neighbor_normal);

                // 0.1 rad ≈ 5°
                if angle_diff > self.placement.edge_angle_delta {
                    cluster.union(edge.origin_vertex, edge.vertex);
                }
            }
        }

        cluster
            .clusters()
            .map(|x| x.iter().copied().collect::<HashSet<_>>())
            .collect()
    }

    /// Return supports on the overhanging faces, about `face_support_spacing` apart.
    ///
    /// The function spreads sample points over the area under the overhanging
    /// faces, not over the whole build plate. A vertical ray from each sample
    /// point gives one support on each overhanging face that the ray hits.
    pub fn place_face_supports(
        &self,
        mesh: &Mesh,
        overhangs: &[(usize, Vector3<f32>)],
    ) -> Vec<SupportPlacement> {
        let mut out = Vec::new();
        let Some((low, high)) = overhang_area(mesh, overhangs) else {
            return out;
        };

        for pos in quazirandom_rect_2d(high - low, self.placement.face_support_spacing) {
            let pos = pos + low;
            let mut intersections = Vec::new();
            for (idx, _angle) in overhangs.iter() {
                let ray = Ray {
                    origin: mesh.inv_transform(&pos.to_homogeneous()),
                    direction: mesh.inv_transform_normal(&Vector3::z()),
                };
                let face = (mesh.face_verts_raw(*idx), mesh.normal(*idx));
                if let Some(mut intersection) = triangle_intersection::<primitive::Ray>(face, ray) {
                    intersection.1 = mesh.transform(&intersection.1);
                    intersections.push(SupportPlacement {
                        point: intersection.1,
                        normal: mesh.transform_normal(&face.1),
                    });
                }
            }

            out.extend(intersections);
        }

        out
    }

    pub fn place_edge_supports(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
        overhangs: &[(usize, Vector3<f32>)],
    ) -> Vec<SupportPlacement> {
        let edge_runs = self.overhanging_edges(mesh, half_edge, overhangs);
        let mut out = Vec::new();

        for run in edge_runs.iter() {
            let mut seen = HashSet::new();
            let start = *run.iter().next().unwrap();
            let mut stack = vec![(start, 0.0)];

            while let Some((vertex, t)) = stack.pop() {
                let start_edge = half_edge.incident_edge(vertex).unwrap();
                let mut edge = start_edge;

                while {
                    if run.contains(&edge.vertex) {
                        let canonical = (
                            edge.origin_vertex.min(edge.vertex),
                            edge.origin_vertex.max(edge.vertex),
                        );

                        if seen.insert(canonical) {
                            let [a, b] = [edge.origin_vertex, edge.vertex]
                                .map(|x| mesh.transform(&mesh.vertices()[x as usize]));

                            let vector = b - a;
                            let len = vector.magnitude();
                            let unit = vector / len;

                            let normal =
                                (mesh.transform_normal(&mesh.normal(edge.face as usize))
                                    + mesh.transform_normal(&mesh.normal(
                                        half_edge.get_edge(edge.twin.unwrap()).face as usize,
                                    )))
                                .normalize();

                            let mut t = t;
                            while t < len {
                                let point = a + unit * t;
                                out.push(SupportPlacement { point, normal });
                                t += self.placement.edge_support_spacing;
                            }

                            stack.push((edge.vertex, t - len));
                        }
                    }

                    edge = half_edge.get_edge(half_edge.get_edge(edge.twin.unwrap()).next);
                    edge != start_edge
                } {}
            }
        }

        out
    }

    /// Find all points that are both lower than their surrounding points and have down facing normals
    pub fn place_point_supports(
        &self,
        mesh: &Mesh,
        half_edge: &HalfEdgeMesh,
    ) -> Vec<SupportPlacement> {
        let mut overhangs = Vec::new();
        let mut seen = HashSet::new();

        let vertices = mesh.vertices();
        for edge in 0..half_edge.half_edge_count() {
            let origin = half_edge.get_edge(edge as u32);
            if !seen.insert(origin.origin_vertex) {
                continue;
            }

            // Ignore points that are not on the bottom of the mesh
            let origin_normal = mesh.transform_normal(&mesh.normal(origin.face as usize));
            if origin_normal.z >= 0.0 {
                continue;
            }

            // Only add to overhangs if the original point is lower than all connected points by one layer
            let origin_pos = mesh.transform(&vertices[origin.origin_vertex as usize]);
            let neighbors = half_edge.connected_vertices(edge as u32);
            if (neighbors.iter()).all(|connected| {
                origin_pos.z < mesh.transform(&vertices[connected.vertex as usize]).z
            }) {
                let point = mesh.transform(&mesh.vertices()[origin.origin_vertex as usize]);
                let normal = (neighbors.iter())
                    .map(|x| mesh.transform_normal(&mesh.normal(x.face as usize)))
                    .sum::<Vector3<_>>()
                    .normalize();

                overhangs.push(SupportPlacement { point, normal });
            }
        }

        overhangs
    }
}

/// Return the lowest and the highest X and Y of the overhanging faces in world space.
fn overhang_area(mesh: &Mesh, overhangs: &[(usize, Vector3<f32>)]) -> Option<(Vector2<f32>, Vector2<f32>)> {
    let mut points = overhangs
        .iter()
        .flat_map(|(face, _)| mesh.face(*face).map(|vertex| mesh.transform(&mesh.vertices()[vertex as usize]).xy()));
    let first = points.next()?;
    Some(points.fold((first, first), |(low, high), point| {
        (low.inf(&point), high.sup(&point))
    }))
}

/// The distance from an overhang to the point that the exposure test examines, in mm.
const PROBE_DISTANCE: f32 = 0.05;
/// A small shift of the probe, so that the vertical ray does not run exactly along a mesh edge.
const PROBE_JITTER: Vector2<f32> = Vector2::new(1.414_213_5e-4, 1.732_050_8e-4);

/// Return the overhangs of `placements` that are on the outside of the model.
///
/// A model can have closed parts that overlap, and a part can have a down
/// facing face inside another part. A support to that face would start
/// inside the model. For each placement, the function moves a short distance
/// out along the normal and counts how many parts hold that point. The
/// function keeps the placement only if no part holds the point.
pub fn exposed_placements(mesh: &Mesh, placements: Vec<SupportPlacement>) -> Vec<SupportPlacement> {
    let vertices: Vec<Vector3<f32>> = mesh.vertices().iter().map(|vertex| mesh.transform(vertex)).collect();
    let triangles: Vec<[Vector3<f32>; 3]> = mesh
        .faces()
        .iter()
        .map(|face| face.map(|i| vertices[i as usize]))
        .collect();

    placements
        .into_par_iter()
        .filter(|placement| {
            let probe = placement.point + placement.normal * PROBE_DISTANCE;
            let probe = (probe.xy() + PROBE_JITTER).push(probe.z);
            winding_below(&triangles, probe) <= 0
        })
        .collect()
}

/// Return the number of closed parts that hold `point`.
///
/// The function follows a vertical ray down from `point`. A face that points
/// down is the bottom of a part above the end of the ray, so the count goes
/// up by one. A face that points up is the top of a part, so the count goes
/// down by one.
fn winding_below(triangles: &[[Vector3<f32>; 3]], point: Vector3<f32>) -> i32 {
    let mut winding = 0;
    for [a, b, c] in triangles {
        let Some(z) = vertical_hit([*a, *b, *c], point.xy()) else {
            continue;
        };
        if z >= point.z {
            continue;
        }
        let normal_z = (b - a).cross(&(c - a)).z;
        if normal_z < 0.0 {
            winding += 1;
        } else if normal_z > 0.0 {
            winding -= 1;
        }
    }
    winding
}

/// Return the height where the vertical line through `point` meets the triangle, or `None` if it misses.
fn vertical_hit([a, b, c]: [Vector3<f32>; 3], point: Vector2<f32>) -> Option<f32> {
    let (a2, b2, c2) = (a.xy(), b.xy(), c.xy());
    let area = (b2 - a2).perp(&(c2 - a2));
    if area.abs() < f32::EPSILON {
        return None;
    }
    let u = (c2 - b2).perp(&(point - b2)) / area;
    let v = (a2 - c2).perp(&(point - c2)) / area;
    let w = 1.0 - u - v;
    (u >= 0.0 && v >= 0.0 && w >= 0.0).then(|| a.z * u + b.z * v + c.z * w)
}
