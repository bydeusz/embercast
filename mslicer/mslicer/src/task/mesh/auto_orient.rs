use clone_macro::clone;
use common::progress::Progress;
use tools::auto_orient::{OrientConfig, Orientation, best_orientation};

use crate::{
    core::{
        history::ModelAction,
        project::model::{Model, ModelId},
    },
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

/// A background task that turns a model into the orientation that needs the least supports.
pub struct AutoOrient {
    progress: Progress,
    handle: TaskThread<Orientation>,

    model: ModelId,
}

impl AutoOrient {
    pub fn new(model: &Model) -> Self {
        let progress = Progress::new();
        let mesh = model.mesh.inner().clone();
        let scale = model.mesh.scale().x;

        let handle = TaskThread::spawn(clone!([progress], move || {
            progress.set_total(1);
            let best = best_orientation(&mesh.vertices, &mesh.faces, scale, &OrientConfig::default());
            progress.set_finished();
            best
        }));

        Self {
            progress,
            handle,
            model: model.id,
        }
    }
}

impl Task for AutoOrient {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Orient the Model")
            .into_poll_result(|best| {
                let platform = app.project.slice_config.platform_size;
                let Some(model) = app.project.model(self.model) else {
                    return PollResult::complete();
                };

                let (old_rotation, old_position) = (model.mesh.rotation(), model.mesh.position());
                app.history.track_model(self.model, ModelAction::Rotation(old_rotation));
                app.history.track_model(self.model, ModelAction::Position(old_position));

                let center = model.bounds.center();
                model.set_rotation(&platform, best.rotation);
                model.set_bounds_center(&platform, center);
                model.align_to_bed();
                model.update_oob(&platform);
                PollResult::complete()
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Orienting Model".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
