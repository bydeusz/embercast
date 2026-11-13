use std::iter;

use ab_glyph::{Font, FontRef, OutlineCurve, Point};
use anyhow::{Context, Result};
use nalgebra::{Vector2, Vector3};

/// The number of straight pieces for one curve of a glyph outline.
const CURVE_STEPS: usize = 8;
/// Points closer than this, in millimeters, count as the same point.
const SAME_POINT: f32 = 1e-5;

/// Return the vertices and faces of `text`, extruded from Z = 0 to Z = `depth`.
///
/// The font size is `size` millimeters per em. The function reads the
/// outline of each glyph, makes the curves straight, and finds which
/// contours are holes. Then the function fills the top and the bottom of
/// each shape and adds a wall along each contour. The mesh is closed and
/// its faces point out.
pub fn text_mesh(font_data: &[u8], text: &str, size: f32, depth: f32) -> Result<(Vec<Vector3<f32>>, Vec<[u32; 3]>)> {
    let font = FontRef::try_from_slice(font_data).context("The font cannot be read.")?;
    let scale = size / font.units_per_em().context("The font has no em size.")?;

    let mut contours: Vec<Vec<Vector2<f32>>> = Vec::new();
    let mut pen = 0.0;
    let mut previous = None;
    for character in text.chars() {
        let glyph = font.glyph_id(character);
        if let Some(previous) = previous {
            pen += font.kern_unscaled(previous, glyph) * scale;
        }
        if let Some(outline) = font.outline(glyph) {
            let offset = Vector2::new(pen, 0.0);
            contours.extend(outline_contours(&outline.curves, scale).into_iter().map(|contour| {
                contour.into_iter().map(|point| point + offset).collect()
            }));
        }
        pen += font.h_advance_unscaled(glyph) * scale;
        previous = Some(glyph);
    }

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for (outer, holes) in group_shapes(contours) {
        let (points, triangles) = triangulate(&outer, &holes);
        let first = vertices.len() as u32;
        vertices.extend(points.iter().map(|p| p.push(0.0)));
        vertices.extend(points.iter().map(|p| p.push(depth)));
        let count = points.len() as u32;
        for [a, b, c] in &triangles {
            faces.push([first + a, first + c, first + b]);
            faces.push([first + count + a, first + count + b, first + count + c]);
        }
        for contour in iter::once(&outer).chain(holes.iter()) {
            add_wall(contour, depth, &mut vertices, &mut faces);
        }
    }
    Ok((vertices, faces))
}

/// Return the closed contours of a glyph outline as polygons in millimeters.
fn outline_contours(curves: &[OutlineCurve], scale: f32) -> Vec<Vec<Vector2<f32>>> {
    let to_vector = |point: &Point| Vector2::new(point.x, point.y) * scale;
    let mut contours: Vec<Vec<Vector2<f32>>> = Vec::new();
    let mut last_end: Option<Point> = None;

    for curve in curves {
        let (start, end) = match curve {
            OutlineCurve::Line(a, b) => (*a, *b),
            OutlineCurve::Quad(a, _, b) => (*a, *b),
            OutlineCurve::Cubic(a, _, _, b) => (*a, *b),
        };
        if last_end != Some(start) {
            contours.push(vec![to_vector(&start)]);
        }
        let contour = contours.last_mut().unwrap();
        match curve {
            OutlineCurve::Line(_, b) => contour.push(to_vector(b)),
            OutlineCurve::Quad(a, c, b) => {
                for step in 1..=CURVE_STEPS {
                    let t = step as f32 / CURVE_STEPS as f32;
                    let point = to_vector(a) * (1.0 - t).powi(2) + to_vector(c) * (2.0 * t * (1.0 - t)) + to_vector(b) * t * t;
                    contour.push(point);
                }
            }
            OutlineCurve::Cubic(a, c1, c2, b) => {
                for step in 1..=CURVE_STEPS {
                    let t = step as f32 / CURVE_STEPS as f32;
                    let u = 1.0 - t;
                    let point = to_vector(a) * u.powi(3)
                        + to_vector(c1) * (3.0 * u * u * t)
                        + to_vector(c2) * (3.0 * u * t * t)
                        + to_vector(b) * t.powi(3);
                    contour.push(point);
                }
            }
        }
        last_end = Some(end);
    }

    contours
        .into_iter()
        .map(|mut contour| {
            contour.dedup_by(|a, b| (*a - *b).norm() < SAME_POINT);
            while contour.len() > 1 && (contour[0] - contour[contour.len() - 1]).norm() < SAME_POINT {
                contour.pop();
            }
            contour
        })
        .filter(|contour| contour.len() >= 3 && signed_area(contour).abs() > SAME_POINT)
        .collect()
}

/// Return twice the signed area of `polygon`. The value is positive for a counterclockwise polygon.
pub fn signed_area(polygon: &[Vector2<f32>]) -> f32 {
    (0..polygon.len())
        .map(|i| polygon[i].perp(&polygon[(i + 1) % polygon.len()]))
        .sum()
}

fn contains(polygon: &[Vector2<f32>], point: Vector2<f32>) -> bool {
    let mut inside = false;
    for i in 0..polygon.len() {
        let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
        if (a.y > point.y) != (b.y > point.y) {
            let x = a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if point.x < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// Return each outer contour with its holes, the outer contour counterclockwise and the holes clockwise.
///
/// A contour inside an even number of other contours is an outer contour.
/// A contour inside an odd number is a hole of the smallest outer contour
/// around it.
fn group_shapes(contours: Vec<Vec<Vector2<f32>>>) -> Vec<(Vec<Vector2<f32>>, Vec<Vec<Vector2<f32>>>)> {
    let depth: Vec<usize> = (0..contours.len())
        .map(|i| {
            (0..contours.len())
                .filter(|&j| j != i && contains(&contours[j], contours[i][0]))
                .count()
        })
        .collect();

    let mut shapes: Vec<(Vec<Vector2<f32>>, Vec<Vec<Vector2<f32>>>)> = Vec::new();
    let mut outer_index = vec![usize::MAX; contours.len()];
    for (i, contour) in contours.iter().enumerate().filter(|(i, _)| depth[*i] % 2 == 0) {
        let mut outer = contour.clone();
        if signed_area(&outer) < 0.0 {
            outer.reverse();
        }
        outer_index[i] = shapes.len();
        shapes.push((outer, Vec::new()));
    }
    for (i, contour) in contours.iter().enumerate().filter(|(i, _)| depth[*i] % 2 == 1) {
        let parent = (0..contours.len())
            .filter(|&j| depth[j] + 1 == depth[i] && contains(&contours[j], contour[0]))
            .min_by(|&a, &b| signed_area(&contours[a]).abs().total_cmp(&signed_area(&contours[b]).abs()));
        let Some(parent) = parent else { continue };
        let mut hole = contour.clone();
        if signed_area(&hole) > 0.0 {
            hole.reverse();
        }
        shapes[outer_index[parent]].1.push(hole);
    }
    shapes
}

/// Return the points and the counterclockwise triangles that fill `outer` without the `holes`.
///
/// `outer` must be counterclockwise and each hole clockwise. First, the
/// function joins each hole to the outer contour with a bridge, from the
/// rightmost point of the hole to a point of the contour that it can see.
/// That gives one polygon. Then the function cuts off one ear after another.
pub fn triangulate(outer: &[Vector2<f32>], holes: &[Vec<Vector2<f32>>]) -> (Vec<Vector2<f32>>, Vec<[u32; 3]>) {
    let mut points: Vec<Vector2<f32>> = outer.to_vec();
    let mut ring: Vec<usize> = (0..outer.len()).collect();

    let mut ordered: Vec<&Vec<Vector2<f32>>> = holes.iter().collect();
    ordered.sort_by(|a, b| {
        let right = |hole: &Vec<Vector2<f32>>| hole.iter().map(|p| p.x).fold(f32::MIN, f32::max);
        right(b).total_cmp(&right(a))
    });
    for hole in ordered {
        bridge_hole(&mut points, &mut ring, hole);
    }

    let triangles = clip_ears(&points, ring);
    (points, triangles)
}

fn bridge_hole(points: &mut Vec<Vector2<f32>>, ring: &mut Vec<usize>, hole: &[Vector2<f32>]) {
    let (start, hole_point) = hole
        .iter()
        .copied()
        .enumerate()
        .max_by(|a, b| a.1.x.total_cmp(&b.1.x))
        .unwrap();

    let visible = |candidate: usize| {
        let target = points[ring[candidate]];
        (0..ring.len()).all(|i| {
            let (a, b) = (points[ring[i]], points[ring[(i + 1) % ring.len()]]);
            let shares_end = (a - target).norm() < SAME_POINT || (b - target).norm() < SAME_POINT;
            shares_end || !segments_cross(hole_point, target, a, b)
        })
    };
    let candidate = (0..ring.len())
        .filter(|&i| points[ring[i]].x >= hole_point.x)
        .filter(|&i| visible(i))
        .min_by(|&a, &b| {
            let distance = |i: usize| (points[ring[i]] - hole_point).norm_squared();
            distance(a).total_cmp(&distance(b))
        })
        .or_else(|| {
            (0..ring.len()).filter(|&i| visible(i)).min_by(|&a, &b| {
                let distance = |i: usize| (points[ring[i]] - hole_point).norm_squared();
                distance(a).total_cmp(&distance(b))
            })
        })
        .unwrap_or(0);

    let first_hole = points.len();
    points.extend_from_slice(hole);
    let mut inserted: Vec<usize> = (0..=hole.len()).map(|k| first_hole + (start + k) % hole.len()).collect();
    inserted.push(ring[candidate]);
    ring.splice(candidate + 1..candidate + 1, inserted);
}

fn segments_cross(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>, d: Vector2<f32>) -> bool {
    let side = |p: Vector2<f32>, q: Vector2<f32>, r: Vector2<f32>| (q - p).perp(&(r - p));
    let (d1, d2) = (side(c, d, a), side(c, d, b));
    let (d3, d4) = (side(a, b, c), side(a, b, d));
    (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0) && d1 != 0.0 && d2 != 0.0 && d3 != 0.0 && d4 != 0.0
}

fn clip_ears(points: &[Vector2<f32>], mut ring: Vec<usize>) -> Vec<[u32; 3]> {
    let mut triangles = Vec::new();
    let mut misses = 0;
    let mut i = 0;
    while ring.len() > 3 && misses < ring.len() * 2 {
        let n = ring.len();
        let (prev, current, next) = (ring[(i + n - 1) % n], ring[i % n], ring[(i + 1) % n]);
        let (a, b, c) = (points[prev], points[current], points[next]);
        let convex = (b - a).perp(&(c - b)) > 0.0;
        let empty = convex
            && ring.iter().all(|&other| {
                let p = points[other];
                other == prev
                    || other == current
                    || other == next
                    || (p - a).norm() < SAME_POINT
                    || (p - b).norm() < SAME_POINT
                    || (p - c).norm() < SAME_POINT
                    || !in_triangle(p, a, b, c)
            });
        if empty {
            triangles.push([prev as u32, current as u32, next as u32]);
            ring.remove(i % n);
            misses = 0;
        } else {
            i += 1;
            misses += 1;
        }
        i %= ring.len().max(1);
    }
    if ring.len() == 3 {
        triangles.push([ring[0] as u32, ring[1] as u32, ring[2] as u32]);
    }
    triangles
}

fn in_triangle(p: Vector2<f32>, a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> bool {
    let (d1, d2, d3) = ((b - a).perp(&(p - a)), (c - b).perp(&(p - b)), (a - c).perp(&(p - c)));
    d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0
}

fn add_wall(contour: &[Vector2<f32>], depth: f32, vertices: &mut Vec<Vector3<f32>>, faces: &mut Vec<[u32; 3]>) {
    let first = vertices.len() as u32;
    let count = contour.len() as u32;
    vertices.extend(contour.iter().map(|p| p.push(0.0)));
    vertices.extend(contour.iter().map(|p| p.push(depth)));
    for i in 0..count {
        let next = (i + 1) % count;
        let (bottom, bottom_next) = (first + i, first + next);
        let (top, top_next) = (first + count + i, first + count + next);
        faces.push([bottom, bottom_next, top_next]);
        faces.push([bottom, top_next, top]);
    }
}
