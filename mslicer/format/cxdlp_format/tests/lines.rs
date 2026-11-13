use common::{container::Run, slice::EncodableLayer};
use cxdlp_format::{LayerEncoder, Line, decode_lines};
use nalgebra::Vector2;

/// Return the runs of an image that has `width` columns, from rows of text.
///
/// A dot is an unlit pixel. A digit is a lit pixel with that digit times 25 as the gray value.
fn runs_from_rows(rows: &[&str]) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for value in rows.iter().flat_map(|row| row.chars()).map(|c| match c {
        '.' => 0,
        digit => digit.to_digit(10).unwrap() as u8 * 25,
    }) {
        match runs.last_mut() {
            Some(run) if run.value == value => run.length += 1,
            _ => runs.push(Run::new(1, value)),
        }
    }
    runs
}

fn encode(rows: &[&str]) -> Vec<Line> {
    let width = rows[0].len() as u32;
    let mut encoder = LayerEncoder::new(Vector2::new(width, rows.len() as u32));
    for run in runs_from_rows(rows) {
        encoder.add_run(run.length, run.value);
    }
    encoder.finish_with_pixel_area(1.0).lines
}

#[test]
fn line_bytes_match_the_bit_layout_of_the_format() {
    let line = Line {
        x: 0x2ABC,
        y_start: 0x1234,
        y_end: 0x0F0F,
        gray: 0x7E,
    };
    let y_start = 0x1234u32;
    let y_end = 0x0F0Fu32;
    let x = 0x2ABCu32;
    let expected = [
        ((y_start >> 5) & 0xFF) as u8,
        (((y_start << 3) + (y_end >> 10)) & 0xFF) as u8,
        ((y_end >> 2) & 0xFF) as u8,
        (((y_end << 6) + (x >> 8)) & 0xFF) as u8,
        (x & 0xFF) as u8,
        0x7E,
    ];
    assert_eq!(line.to_bytes(), expected);
}

#[test]
fn line_bytes_round_trip_at_the_largest_coordinates() {
    for line in [
        Line { x: 0, y_start: 0, y_end: 0, gray: 255 },
        Line { x: 16383, y_start: 8191, y_end: 8191, gray: 1 },
        Line { x: 4319, y_start: 0, y_end: 2559, gray: 128 },
        Line { x: 1, y_start: 2558, y_end: 2559, gray: 200 },
    ] {
        assert_eq!(Line::from_bytes(&line.to_bytes()), line);
    }
}

#[test]
fn encoder_writes_one_line_per_column_run_sorted_by_column() {
    let lines = encode(&[
        "......",
        ".99...",
        ".99.9.",
        "..9.9.",
        "......",
    ]);
    let gray = 225;
    assert_eq!(
        lines,
        vec![
            Line { x: 1, y_start: 1, y_end: 2, gray },
            Line { x: 2, y_start: 1, y_end: 3, gray },
            Line { x: 4, y_start: 2, y_end: 3, gray },
        ]
    );
}

#[test]
fn encoder_splits_a_column_where_the_gray_value_changes() {
    let lines = encode(&[".9.", ".9.", ".5.", ".9."]);
    assert_eq!(
        lines,
        vec![
            Line { x: 1, y_start: 0, y_end: 1, gray: 225 },
            Line { x: 1, y_start: 2, y_end: 2, gray: 125 },
            Line { x: 1, y_start: 3, y_end: 3, gray: 225 },
        ]
    );
}

#[test]
fn encoder_splits_a_column_at_an_unlit_gap() {
    let lines = encode(&["9", ".", "9"]);
    assert_eq!(
        lines,
        vec![
            Line { x: 0, y_start: 0, y_end: 0, gray: 225 },
            Line { x: 0, y_start: 2, y_end: 2, gray: 225 },
        ]
    );
}

#[test]
fn runs_that_cross_rows_survive_encoding_and_decoding() {
    let rows = [
        "..999999",
        "99999999",
        "9999....",
        "........",
        "3.3.3.3.",
        "99999999",
    ];
    let runs = runs_from_rows(&rows);
    let lines = encode(&rows);
    let decoded = decode_lines(&lines, Vector2::new(8, rows.len() as u32));
    assert_eq!(decoded, runs);
}

#[test]
fn empty_layer_has_no_lines() {
    assert!(encode(&["....", "...."]).is_empty());
}
