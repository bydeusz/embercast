use cxdlp_format::{Span, largest_enclosed_area};

/// Return the lit spans of an image from rows of text, where `#` is a lit pixel.
fn spans_from_rows(rows: &[&str]) -> (u32, Vec<Span>) {
    let mut spans = Vec::new();
    for (y, row) in rows.iter().enumerate() {
        let mut start = None;
        for (x, c) in row.chars().chain(std::iter::once('.')).enumerate() {
            match (c, start) {
                ('#', None) => start = Some(x as u32),
                ('#', Some(_)) => {}
                (_, Some(x_start)) => {
                    spans.push(Span { y: y as u32, x_start, x_end: x as u32 - 1 });
                    start = None;
                }
                (_, None) => {}
            }
        }
    }
    (rows[0].len() as u32, spans)
}

fn area(rows: &[&str]) -> u64 {
    let (width, spans) = spans_from_rows(rows);
    largest_enclosed_area(width, &spans)
}

#[test]
fn empty_image_has_no_area() {
    assert_eq!(area(&["....", "...."]), 0);
}

#[test]
fn solid_block_area_is_its_pixel_count() {
    assert_eq!(area(&["......", ".###..", ".###..", "......"]), 6);
}

#[test]
fn ring_area_includes_its_hole() {
    assert_eq!(
        area(&[
            ".......",
            ".#####.",
            ".#...#.",
            ".#...#.",
            ".#####.",
            ".......",
        ]),
        20
    );
}

#[test]
fn island_inside_a_ring_belongs_to_the_ring_area() {
    assert_eq!(
        area(&[
            "#######",
            "#.....#",
            "#..#..#",
            "#.....#",
            "#######",
        ]),
        35
    );
}

#[test]
fn ring_that_is_open_to_the_outside_has_no_filled_hole() {
    assert_eq!(
        area(&[
            ".......",
            ".#####.",
            ".#.....",
            ".#...#.",
            ".#####.",
            ".......",
        ]),
        13
    );
}

#[test]
fn largest_of_two_separate_shapes_wins() {
    assert_eq!(area(&["##....###", "##....###", "......###"]), 9);
}

#[test]
fn diagonal_neighbours_are_one_shape() {
    assert_eq!(area(&["#...", ".#..", "..#.", "...#"]), 4);
}

#[test]
fn shape_on_the_image_edge_keeps_holes_that_touch_no_edge() {
    assert_eq!(area(&["#####", "#...#", "#####"]), 15);
}
