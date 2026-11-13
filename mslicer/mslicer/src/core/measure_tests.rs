use nalgebra::Vector3;

use crate::core::measure::Measurement;

#[test]
fn two_points_give_the_distance_and_the_gap_along_each_axis() {
    let mut measure = Measurement::default();
    measure.add(Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(measure.distance(), None);
    measure.add(Vector3::new(4.0, -2.0, 3.0));

    let (distance, delta) = measure.distance().unwrap();
    assert!((distance - 5.0).abs() < 1e-6);
    assert_eq!(delta, Vector3::new(3.0, 4.0, 0.0));
}

#[test]
fn third_point_starts_a_new_measurement() {
    let mut measure = Measurement::default();
    for x in [0.0, 1.0, 7.0] {
        measure.add(Vector3::new(x, 0.0, 0.0));
    }
    assert_eq!(measure.points(), &[Vector3::new(7.0, 0.0, 0.0)]);
}
