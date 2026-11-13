use std::sync::Arc;

use clone_macro::clone;
use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
};
use slicer::post_process::cavities::detect_cavities;

use crate::{
    core::slice::annotations::{Annotation, Annotations},
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

/// A background task that finds resin traps and suction cups and marks them in the slice preview.
pub struct CavityDetection {
    progress: Progress,
    handle: TaskThread<()>,
}

impl CavityDetection {
    pub fn new(config: SliceConfig, layers: Vec<Layer>, annotations: Arc<Annotations>) -> Self {
        let progress = Progress::new();
        Self {
            handle: TaskThread::spawn(clone!([progress], move || {
                let report = detect_cavities(&config, &layers, &progress);
                let marks = report.all_layer_marks(&layers, &config);

                let mut locked = annotations.lock();
                for (layer, runs) in marks {
                    let runs = runs
                        .into_iter()
                        .map(|run| Run::new(run.length, Annotation::from_cavity_mark(run.value)))
                        .collect();
                    locked.insert_runs(layer, runs);
                }
                drop(locked);
                annotations.set_cavities(report.cavities);
            })),
            progress,
        }
    }
}

impl Task for CavityDetection {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Unexpected Error Detecting Resin Traps")
            .into_poll_result(|_| PollResult::complete())
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Detecting Resin Traps".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
