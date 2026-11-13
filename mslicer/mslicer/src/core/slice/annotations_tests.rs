use common::container::{Run, rle};

use crate::core::slice::annotations::{Annotation, Annotations};

#[test]
fn cavity_marks_and_island_marks_share_one_layer() {
    let annotations = Annotations::default();
    let mut locked = annotations.lock();
    locked.insert_layer(Annotation::Island, 3, &[2, 3, 5]);
    locked.insert_runs(
        3,
        vec![
            Run::new(4, Annotation::None),
            Run::new(4, Annotation::ResinTrap),
            Run::new(2, Annotation::None),
        ],
    );

    let pixels = rle::decode_vec(&locked.get_layer(3));
    assert_eq!(pixels, vec![0, 0, 1, 1, 2, 2, 2, 2, 0, 0]);
    assert_eq!(locked.worst(3), Some(Annotation::Island));
    assert_eq!(locked.worst(4), None);
}

#[test]
fn layer_with_only_a_suction_cup_shows_the_suction_cup() {
    let annotations = Annotations::default();
    let mut locked = annotations.lock();
    locked.insert_runs(0, vec![Run::new(3, Annotation::None), Run::new(1, Annotation::SuctionCup)]);
    assert_eq!(locked.worst(0), Some(Annotation::SuctionCup));
}
