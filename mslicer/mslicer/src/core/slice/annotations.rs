use std::{
    collections::HashMap,
    mem,
    sync::atomic::{AtomicBool, Ordering},
};

use common::container::Run;
use egui::Color32;
use parking_lot::{Mutex, MutexGuard};
use slicer::post_process::cavities::{Cavity, CavityKind};

pub const ISLAND_COLOR: Color32 = Color32::from_rgb(0x9e, 0x2b, 0x25);
pub const RESIN_TRAP_COLOR: Color32 = Color32::from_rgb(0xd9, 0x65, 0x5c);
pub const SUCTION_CUP_COLOR: Color32 = Color32::from_rgb(0xd1, 0x8b, 0x2c);

#[derive(Default)]
pub struct Annotations {
    layers: Mutex<HashMap<usize, Vec<Run<Annotation>>>>,
    cavities: Mutex<Option<Vec<Cavity>>>,
    updated: AtomicBool,
}

pub struct LockedAnnotations<'a> {
    layers: MutexGuard<'a, HashMap<usize, Vec<Run<Annotation>>>>,
    updated: &'a AtomicBool,
}

/// The problem of one pixel in the sliced preview.
///
/// The values are the indices of the colors in the slice preview shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Annotation {
    None = 0b00,
    Island = 0b01,
    ResinTrap = 0b10,
    SuctionCup = 0b11,
}

impl Annotation {
    /// Return the annotation for a cavity mark from [`slicer::post_process::cavities`].
    pub fn from_cavity_mark(mark: u8) -> Self {
        match mark {
            m if m == CavityKind::ResinTrap.mark() => Annotation::ResinTrap,
            m if m == CavityKind::SuctionCup.mark() => Annotation::SuctionCup,
            _ => Annotation::None,
        }
    }

    /// Return the color of the annotation on the layer slider.
    pub fn color(self) -> Option<Color32> {
        match self {
            Annotation::None => None,
            Annotation::Island => Some(ISLAND_COLOR),
            Annotation::ResinTrap => Some(RESIN_TRAP_COLOR),
            Annotation::SuctionCup => Some(SUCTION_CUP_COLOR),
        }
    }

    fn severity(self) -> u8 {
        match self {
            Annotation::None => 0,
            Annotation::SuctionCup => 1,
            Annotation::ResinTrap => 2,
            Annotation::Island => 3,
        }
    }
}

impl Annotations {
    pub fn lock(&self) -> LockedAnnotations<'_> {
        LockedAnnotations {
            layers: self.layers.lock(),
            updated: &self.updated,
        }
    }

    pub fn take_updated(&self) -> bool {
        self.updated.swap(false, Ordering::Relaxed)
    }

    /// Return the cavities that the last check found, or `None` before the first check.
    pub fn cavities(&self) -> Option<Vec<Cavity>> {
        self.cavities.lock().clone()
    }

    pub fn set_cavities(&self, cavities: Vec<Cavity>) {
        *self.cavities.lock() = Some(cavities);
        self.updated.store(true, Ordering::Relaxed);
    }
}

impl<'a> LockedAnnotations<'a> {
    /// Return the most severe annotation of `layer`, or `None` if the layer has no problem.
    pub fn worst(&self, layer: usize) -> Option<Annotation> {
        let runs = self.layers.get(&layer)?;
        (runs.iter().map(|run| run.value))
            .filter(|value| *value != Annotation::None)
            .max_by_key(|value| value.severity())
    }

    pub fn get_layer(&self, layer: usize) -> Vec<Run> {
        let Some(layer) = self.layers.get(&layer) else {
            return Vec::new();
        };

        // SAFETY: Annotation has repr(u8), so can be safely interpreted as a u8
        unsafe { mem::transmute::<Vec<Run<Annotation>>, Vec<Run<u8>>>(layer.clone()) }
    }

    /// Mark `annotation` on `layer`, where `runs` alternates unmarked and marked lengths.
    pub fn insert_layer(&mut self, annotation: Annotation, layer: usize, runs: &[u64]) {
        let runs = runs
            .iter()
            .enumerate()
            .map(|(i, &l)| Run {
                length: l,
                value: [Annotation::None, annotation][(i % 2 != 0) as usize],
            })
            .collect::<Vec<_>>();
        self.insert_runs(layer, runs);
    }

    /// Mark `runs` on `layer`. A marked pixel of `runs` replaces the mark that the layer had.
    pub fn insert_runs(&mut self, layer: usize, runs: Vec<Run<Annotation>>) {
        let merged = match self.layers.remove(&layer) {
            Some(existing) => overlay(&existing, &runs),
            None => runs,
        };
        self.layers.insert(layer, merged);
        self.updated.store(true, Ordering::Relaxed);
    }
}

/// Return `base` with every marked pixel of `top` on it.
fn overlay(base: &[Run<Annotation>], top: &[Run<Annotation>]) -> Vec<Run<Annotation>> {
    let mut out: Vec<Run<Annotation>> = Vec::new();
    let (mut base_runs, mut top_runs) = (base.iter().copied(), top.iter().copied());
    let (mut a, mut b) = (base_runs.next(), top_runs.next());
    while let (Some(mut base_run), Some(mut top_run)) = (a, b) {
        let length = base_run.length.min(top_run.length);
        let value = if top_run.value != Annotation::None { top_run.value } else { base_run.value };
        match out.last_mut() {
            Some(run) if run.value == value => run.length += length,
            _ => out.push(Run::new(length, value)),
        }
        base_run.length -= length;
        top_run.length -= length;
        a = if base_run.length == 0 { base_runs.next() } else { Some(base_run) };
        b = if top_run.length == 0 { top_runs.next() } else { Some(top_run) };
    }
    out.extend(a.into_iter().chain(base_runs));
    out.extend(b.into_iter().chain(top_runs));
    out
}
