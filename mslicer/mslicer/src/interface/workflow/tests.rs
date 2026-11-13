use crate::{
    core::state::Tool,
    interface::workflow::{
        RailTool, Step,
        estimate::{layer_count, short_duration},
    },
};

#[test]
fn every_step_has_its_own_tools() {
    let mut seen = Vec::new();
    for step in Step::ALL {
        assert!(!step.tools().is_empty(), "{step:?} has no tools.");
        for tool in step.tools() {
            assert!(!seen.contains(tool), "{tool:?} is in two steps.");
            seen.push(*tool);
        }
    }
}

#[test]
fn the_default_tool_is_the_first_tool_of_the_default_step() {
    assert_eq!(Step::default().tools()[0], RailTool::default());
}

#[test]
fn step_index_follows_the_order_of_the_work() {
    let indexes: Vec<usize> = Step::ALL.iter().map(|step| step.index()).collect();
    assert_eq!(indexes, vec![0, 1, 2]);
}

#[test]
fn only_the_click_tools_change_the_click_in_the_3d_view() {
    assert!(RailTool::Orient.viewport_tool() == Tool::Orient);
    assert!(RailTool::Measure.viewport_tool() == Tool::Measure);
    assert!(RailTool::EditSupports.viewport_tool() == Tool::Support);
    for tool in [
        RailTool::Transform,
        RailTool::Hollow,
        RailTool::AutoSupports,
        RailTool::Layers,
    ] {
        assert!(
            tool.viewport_tool() == Tool::Select,
            "{tool:?} changes the click."
        );
    }
}

#[test]
fn a_model_needs_one_layer_for_each_started_layer_height() {
    assert_eq!(layer_count(11.5, 0.05), 230);
    assert_eq!(layer_count(11.51, 0.05), 231);
    assert_eq!(layer_count(0.0, 0.05), 0);
}

#[test]
fn a_layer_count_ignores_rounding_noise_of_the_model_height() {
    assert_eq!(layer_count(16.500002, 0.05), 330);
}

#[test]
fn a_zero_layer_height_gives_no_layers() {
    assert_eq!(layer_count(10.0, 0.0), 0);
}

#[test]
fn a_print_time_shows_hours_and_whole_minutes() {
    assert_eq!(short_duration(3689.0), "1 h 01 min");
    assert_eq!(short_duration(5209.0), "1 h 27 min");
}

#[test]
fn a_print_time_under_an_hour_shows_only_minutes() {
    assert_eq!(short_duration(45.0 * 60.0), "45 min");
}

#[test]
fn a_print_time_rounds_to_the_nearest_minute() {
    assert_eq!(short_duration(3599.0), "1 h 00 min");
    assert_eq!(short_duration(89.0), "1 min");
}

#[test]
fn a_very_short_print_time_is_less_than_one_minute() {
    assert_eq!(short_duration(20.0), "< 1 min");
    assert_eq!(short_duration(0.0), "0 min");
}
