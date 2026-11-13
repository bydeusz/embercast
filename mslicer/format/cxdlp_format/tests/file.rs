use common::{
    progress::Progress,
    serde::{DynamicSerializer, SliceDeserializer},
    slice::{EncodableLayer, ExposureConfig, Height, SliceConfig, SlicedFile},
    units::{Milimeter, Milimeters, MilimetersPerMinute, Minute, Second, Seconds},
};
use cxdlp_format::{File, LayerEncoder, MAGIC, crc32_raw};
use image::{Rgba, RgbaImage};
use nalgebra::{Vector2, Vector3};

/// Return the slice settings of the Halot One Plus with the Siraya Tech Build profile.
fn halot_one_plus_config() -> SliceConfig {
    let speed = |mm_per_min: f32| MilimetersPerMinute::new(mm_per_min).convert();
    SliceConfig {
        platform_resolution: Vector2::new(4320, 2560),
        platform_size: Vector3::new(172.8, 102.4, 160.0).map(Milimeters::new),
        slice_height: Milimeters::new(0.05),
        exposure_config: ExposureConfig {
            exposure_time: Seconds::new(3.2),
            exposure_delay: Seconds::new(4.0),
            pwm: 255,
            lift_distance: Milimeters::new(6.0),
            lift_speed: speed(60.0),
            retract_speed: speed(180.0),
        },
        first_exposure_config: ExposureConfig {
            exposure_time: Seconds::new(35.0),
            exposure_delay: Seconds::new(4.0),
            pwm: 255,
            lift_distance: Milimeters::new(6.0),
            lift_speed: speed(60.0),
            retract_speed: speed(150.0),
        },
        first_layers: Height::Layers(6),
        transition_layers: Height::Layers(0),
        ..Default::default()
    }
}

/// Return a layer with one lit square of `size` pixels in the middle of the screen.
fn square_layer(config: &SliceConfig, size: u32) -> cxdlp_format::Layer {
    let resolution = config.platform_resolution;
    let (left, top) = ((resolution.x - size) / 2, (resolution.y - size) / 2);
    let mut encoder = LayerEncoder::new(resolution);
    encoder.add_run((top * resolution.x + left) as u64, 0);
    for row in 0..size {
        encoder.add_run(size as u64, 255);
        let gap = if row + 1 == size {
            (resolution.y - top - size) * resolution.x + (resolution.x - left - size)
        } else {
            resolution.x - size
        };
        encoder.add_run(gap as u64, 0);
    }
    encoder.finish(config, &config.exposure_config, config.slice_height)
}

fn serialize(file: &File) -> Vec<u8> {
    let mut ser = DynamicSerializer::new();
    file.serialize(&mut ser);
    ser.into_inner()
}

fn sample_file() -> File {
    let config = halot_one_plus_config();
    let layers = (0..10).map(|i| square_layer(&config, 100 + i * 10)).collect();
    let mut file = File::from_layers(&config, layers);
    file.set_preview(&RgbaImage::from_pixel(400, 300, Rgba([209, 139, 44, 255])));
    file
}

#[test]
fn header_holds_the_halot_one_plus_model_and_resolution() {
    let file = sample_file();
    assert_eq!(file.header.version, 3);
    assert_eq!(file.header.printer_model, "CL-79");
    assert_eq!((file.header.resolution_x, file.header.resolution_y), (4320, 2560));
    assert_eq!(file.header.layer_count, 10);
}

#[test]
fn print_settings_use_the_integer_units_of_the_format() {
    let info = sample_file().slicer_info;
    assert_eq!(info.display_width, "172.8");
    assert_eq!(info.display_height, "102.4");
    assert_eq!(info.layer_height, "0.05");
    assert_eq!(info.exposure_time, 32);
    assert_eq!(info.wait_time_before_cure, 4);
    assert_eq!(info.bottom_exposure_time, 35);
    assert_eq!(info.bottom_layers, 6);
    assert_eq!(info.bottom_lift_height, 6);
    assert_eq!(info.bottom_lift_speed, 1);
    assert_eq!(info.lift_height, 6);
    assert_eq!(info.lift_speed, 1);
    assert_eq!(info.retract_speed, 3);
    assert_eq!((info.bottom_light_pwm, info.light_pwm), (255, 255));
}

#[test]
fn largest_area_is_in_square_millimeters_times_1000() {
    let file = sample_file();
    let pixel_area_mm2 = 0.04 * 0.04;
    let expected = (100.0 * 100.0 * pixel_area_mm2 * 1000.0) as u32;
    assert!(file.layers[0].largest_area.abs_diff(expected) <= 1);
}

#[test]
fn file_starts_and_ends_with_the_magic_and_a_valid_checksum() {
    let bytes = serialize(&sample_file());
    assert_eq!(&bytes[0..4], &[0, 0, 0, 9]);
    assert_eq!(&bytes[4..13], MAGIC);
    assert_eq!(&bytes[13..15], &[0, 3]);

    let footer_start = bytes.len() - 4 - MAGIC.len();
    assert_eq!(&bytes[footer_start..bytes.len() - 4], MAGIC);
    let stored = u32::from_be_bytes(bytes[bytes.len() - 4..].try_into().unwrap());
    assert_eq!(stored, crc32_raw(&bytes[..bytes.len() - 4]));
}

#[test]
fn file_reads_back_with_the_same_blocks_and_layers() {
    let file = sample_file();
    let bytes = serialize(&file);
    let read = File::deserialize(&mut SliceDeserializer::new(&bytes)).unwrap();

    assert_eq!(read.header, file.header);
    assert_eq!(read.slicer_info, file.slicer_info);
    assert_eq!(read.slicer_info_v3, file.slicer_info_v3);
    assert_eq!(read.layers, file.layers);
    assert_eq!(read.small_preview.inner_data(), file.small_preview.inner_data());
    assert_eq!(serialize(&read), bytes);
}

#[test]
fn read_file_gives_back_the_slice_settings() {
    let bytes = serialize(&sample_file());
    let read = File::deserialize(&mut SliceDeserializer::new(&bytes)).unwrap();
    let config = read.slice_config();

    assert_eq!(config.platform_resolution, Vector2::new(4320, 2560));
    assert_eq!(config.platform_size.z.get::<Milimeter>(), 160.0);
    assert!((config.slice_height.get::<Milimeter>() - 0.05).abs() < 1e-6);
    assert!((config.exposure_config.exposure_time.get::<Second>() - 3.2).abs() < 1e-5);
    assert_eq!(config.first_exposure_config.exposure_time.get::<Second>(), 35.0);
    assert_eq!(config.first_layers, Height::Layers(6));
    let lift_speed = config.exposure_config.lift_speed.get::<Milimeter, Minute>();
    assert!((lift_speed - 60.0).abs() < 1e-3);
}

#[test]
fn read_layers_hold_the_lit_square() {
    let bytes = serialize(&sample_file());
    let read = File::deserialize(&mut SliceDeserializer::new(&bytes)).unwrap();
    let layers = read.layers(&Progress::new());

    assert_eq!(layers.len(), 10);
    assert_eq!(layers[0].area, 100 * 100);
    assert_eq!(layers[9].area, 190 * 190);
    assert_eq!(layers[5].exposure.exposure_time.get::<Second>(), 35.0);
    assert!((layers[6].exposure.exposure_time.get::<Second>() - 3.2).abs() < 1e-5);
}

#[test]
fn changed_byte_fails_the_checksum() {
    let mut bytes = serialize(&sample_file());
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x01;
    let error = File::deserialize(&mut SliceDeserializer::new(&bytes)).err().unwrap();
    assert!(error.to_string().contains("Checksum"));
}

#[test]
fn raw_crc_starts_at_zero_and_uses_the_reflected_polynomial() {
    assert_eq!(crc32_raw(&[]), 0);
    assert_eq!(crc32_raw(&[0x00]), 0);
    assert_eq!(crc32_raw(&[0x01]), 0x7707_3096);
    let (a, b) = (b"embercast", b"cxdlp v3!");
    let xored: Vec<u8> = a.iter().zip(b).map(|(x, y)| x ^ y).collect();
    assert_eq!(crc32_raw(&xored), crc32_raw(a) ^ crc32_raw(b));
}

/// Return `body` with a valid version 3 checksum at the end.
fn with_checksum(body: &[u8]) -> Vec<u8> {
    let mut bytes = body.to_vec();
    bytes.extend_from_slice(&crc32_raw(body).to_be_bytes());
    bytes
}

#[test]
fn cut_file_with_a_valid_checksum_gives_an_error_and_no_crash() {
    let bytes = serialize(&sample_file());
    let body = &bytes[..bytes.len() - 4];
    let cuts = (20..body.len()).step_by(997).chain([body.len() - 1, body.len() - 20]);
    for cut in cuts {
        let damaged = with_checksum(&body[..cut]);
        let result = File::deserialize(&mut SliceDeserializer::new(&damaged));
        assert!(result.is_err(), "A file cut at byte {cut} was accepted.");
    }
}

#[test]
fn zero_resolution_gives_an_error() {
    let bytes = serialize(&sample_file());
    let mut body = bytes[..bytes.len() - 4].to_vec();
    let resolution_x = 4 + MAGIC.len() + 2 + 4 + "CL-79".len() + 1 + 2;
    assert_eq!(&body[resolution_x..resolution_x + 2], &4320u16.to_be_bytes());
    body[resolution_x..resolution_x + 2].copy_from_slice(&[0, 0]);
    let error = File::deserialize(&mut SliceDeserializer::new(&with_checksum(&body))).err().unwrap();
    assert!(error.to_string().contains("Resolution X"));
}
