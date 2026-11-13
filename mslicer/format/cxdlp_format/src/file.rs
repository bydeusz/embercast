use anyhow::{Result, ensure};
use common::{
    progress::Progress,
    serde::{Deserializer, DynamicSerializer, Serializer, SliceDeserializer},
    slice::{
        self, ExposureConfig, ExposureRemap, Height, SliceConfig, SliceMode, SlicedFile,
        Supersample,
    },
    units::{CentimetersPerSecond, Milimeter, Milimeters, Second, Seconds, Velocity},
};
use image::{RgbaImage, imageops::FilterType};
use nalgebra::{Vector2, Vector3};

use crate::{
    CrealityPrinter, Header, Layer, MAGIC, PAGE_BREAK, PreviewImage, SlicerInfo, SlicerInfoV3,
    VERSION,
    checksum::{crc32_raw, xor_checksum},
    decode_lines,
    reader::{ensure_remaining, expect_page_break},
    strings::format_decimal,
};

const SOFTWARE_NAME: &str = concat!("Embercast ", env!("CARGO_PKG_VERSION"));
const DEFAULT_BUILD_HEIGHT_MM: f32 = 160.0;
const CHECKSUM_SIZE: usize = 4;

/// One CXDLP file with all its blocks.
pub struct File {
    pub header: Header,
    pub small_preview: PreviewImage<116, 116>,
    pub large_previews: [PreviewImage<290, 290>; 2],
    pub slicer_info: SlicerInfo,
    pub slicer_info_v3: SlicerInfoV3,
    pub layers: Vec<Layer>,
}

fn whole(value: f32) -> u16 {
    value.round().clamp(0.0, u16::MAX as f32) as u16
}

fn millimeters_per_second(speed: CentimetersPerSecond) -> f32 {
    speed.get::<Milimeter, Second>()
}

fn from_millimeters_per_second(speed: u16) -> CentimetersPerSecond {
    Velocity::<Milimeter, Second>::new(speed as f32).convert()
}

impl File {
    /// Return a version 3 file for the encoded `layers` and the settings in `config`.
    ///
    /// The function finds the printer model from the screen resolution. The
    /// format stores most settings as whole numbers, so the function rounds
    /// them. The wait time, the lift heights and the speeds are at least 1,
    /// because the printer does not print with a value of 0.
    pub fn from_layers(config: &SliceConfig, layers: Vec<Layer>) -> Self {
        let printer = CrealityPrinter::from_resolution(config.platform_resolution);
        let (bottom_layers, _) = config.layer_counts();
        let normal = &config.exposure_config;
        let bottom = &config.first_exposure_config;

        Self {
            header: Header {
                version: VERSION,
                printer_model: printer.map(|p| p.model.to_owned()).unwrap_or_default(),
                layer_count: layers.len() as u16,
                resolution_x: config.platform_resolution.x as u16,
                resolution_y: config.platform_resolution.y as u16,
            },
            small_preview: PreviewImage::empty(),
            large_previews: [PreviewImage::empty(), PreviewImage::empty()],
            slicer_info: SlicerInfo {
                display_width: format_decimal(config.platform_size.x.get::<Milimeter>(), 2),
                display_height: format_decimal(config.platform_size.y.get::<Milimeter>(), 2),
                layer_height: format_decimal(config.slice_height.get::<Milimeter>(), 3),
                exposure_time: whole(normal.exposure_time.get::<Second>() * 10.0),
                wait_time_before_cure: whole(normal.exposure_delay.get::<Second>()).max(1),
                bottom_exposure_time: whole(bottom.exposure_time.get::<Second>()),
                bottom_layers: bottom_layers as u16,
                bottom_lift_height: whole(bottom.lift_distance.get::<Milimeter>()).max(1),
                bottom_lift_speed: whole(millimeters_per_second(bottom.lift_speed)).max(1),
                lift_height: whole(normal.lift_distance.get::<Milimeter>()).max(1),
                lift_speed: whole(millimeters_per_second(normal.lift_speed)).max(1),
                retract_speed: whole(millimeters_per_second(normal.retract_speed)).max(1),
                bottom_light_pwm: bottom.pwm as u16,
                light_pwm: normal.pwm as u16,
            },
            slicer_info_v3: SlicerInfoV3 {
                software_name: SOFTWARE_NAME.to_owned(),
                ..Default::default()
            },
            layers,
        }
    }

    /// Write the complete file, with the checksum at the end.
    ///
    /// Version 3 ends with the CRC-32 of all bytes before the checksum.
    /// Version 2 ends with the XOR of all these bytes.
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        let start = ser.pos();
        self.header.serialize(ser);

        self.small_preview.serialize(ser);
        ser.write_bytes(PAGE_BREAK);
        for preview in &self.large_previews {
            preview.serialize(ser);
            ser.write_bytes(PAGE_BREAK);
        }

        self.slicer_info.serialize(ser);
        for layer in &self.layers {
            ser.write_u32_be(layer.largest_area);
        }
        ser.write_bytes(PAGE_BREAK);
        if self.header.version >= 3 {
            self.slicer_info_v3.serialize(ser);
        }

        for layer in &self.layers {
            layer.serialize(ser);
        }
        ser.write_u32_be(MAGIC.len() as u32);
        ser.write_bytes(MAGIC);

        let end = ser.pos();
        let body = ser.view_mut(start, end - start);
        let checksum = if self.header.version >= 3 {
            crc32_raw(body)
        } else {
            xor_checksum(body) as u32
        };
        ser.write_u32_be(checksum);
    }

    /// Read a complete file and make sure that the checksum and the layer lines are valid.
    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        let size = des.size();
        ensure!(size > CHECKSUM_SIZE, "File is too small to be a CXDLP file.");
        let header = Header::deserialize(des)?;

        let body = des.execute_at(0, |des| des.read_slice(size - CHECKSUM_SIZE));
        let stored = des.execute_at(size - CHECKSUM_SIZE, |des| des.read_u32_be());
        if header.version >= 3 {
            let expected = crc32_raw(body);
            ensure!(stored == expected, "Checksum is {stored:#010x}, expected {expected:#010x}.");
        } else {
            let expected = xor_checksum(body);
            ensure!(stored as u8 == expected, "Checksum is {stored:#04x}, expected {expected:#04x}.");
        }

        ensure_remaining(des, 116 * 116 * 2, "small preview")?;
        let small_preview = PreviewImage::deserialize(des);
        expect_page_break(des, "small preview")?;
        let mut large_previews = [PreviewImage::empty(), PreviewImage::empty()];
        for preview in &mut large_previews {
            ensure_remaining(des, 290 * 290 * 2, "large preview")?;
            *preview = PreviewImage::deserialize(des);
            expect_page_break(des, "large preview")?;
        }

        let slicer_info = SlicerInfo::deserialize(des)?;
        ensure_remaining(des, header.layer_count as usize * 4, "layer area table")?;
        des.advance_by(header.layer_count as usize * 4);
        expect_page_break(des, "layer area table")?;
        let slicer_info_v3 = if header.version >= 3 {
            SlicerInfoV3::deserialize(des)?
        } else {
            SlicerInfoV3::default()
        };

        let mut layers = Vec::with_capacity(header.layer_count as usize);
        for index in 0..header.layer_count {
            let layer = Layer::deserialize(des)?;
            let outside = layer.lines.iter().find(|line| {
                line.x >= header.resolution_x
                    || line.y_start > line.y_end
                    || line.y_end >= header.resolution_y
            });
            ensure!(outside.is_none(), "Layer {index} has a line outside the screen: {outside:?}.");
            layers.push(layer);
        }

        ensure_remaining(des, 4 + MAGIC.len(), "footer")?;
        ensure!(des.read_u32_be() == MAGIC.len() as u32, "Footer has the wrong magic size.");
        ensure!(des.read_slice(MAGIC.len()) == MAGIC, "Footer has the wrong magic.");

        Ok(Self {
            header,
            small_preview,
            large_previews,
            slicer_info,
            slicer_info_v3,
            layers,
        })
    }

    fn exposure_config(&self, bottom: bool) -> ExposureConfig {
        let info = &self.slicer_info;
        ExposureConfig {
            exposure_time: if bottom {
                Seconds::new(info.bottom_exposure_time as f32)
            } else {
                Seconds::new(info.exposure_time as f32 / 10.0)
            },
            exposure_delay: Seconds::new(info.wait_time_before_cure as f32),
            pwm: if bottom { info.bottom_light_pwm } else { info.light_pwm }.min(255) as u8,
            lift_distance: Milimeters::new(if bottom { info.bottom_lift_height } else { info.lift_height } as f32),
            lift_speed: from_millimeters_per_second(if bottom { info.bottom_lift_speed } else { info.lift_speed }),
            retract_speed: from_millimeters_per_second(info.retract_speed),
        }
    }

    fn resolution(&self) -> Vector2<u32> {
        Vector2::new(self.header.resolution_x as u32, self.header.resolution_y as u32)
    }
}

impl SlicedFile for File {
    fn serialize(&self, ser: &mut DynamicSerializer, progress: &Progress) {
        self.serialize(ser);
        progress.set_total(1);
        progress.set_finished();
    }

    fn set_preview(&mut self, preview: &RgbaImage) {
        self.small_preview = PreviewImage::from_image_scaled(preview, FilterType::Triangle);
        let large = PreviewImage::from_image_scaled(preview, FilterType::Triangle);
        self.large_previews = [large.clone(), large];
    }

    fn slice_config(&self) -> SliceConfig {
        let parse = |text: &str| Milimeters::new(text.parse::<f32>().unwrap_or(0.0));
        let build_height = CrealityPrinter::from_model(&self.header.printer_model)
            .map(|printer| printer.build_height())
            .unwrap_or(Milimeters::new(DEFAULT_BUILD_HEIGHT_MM));

        SliceConfig {
            mode: SliceMode::Raster,
            supersample: Supersample::default(),
            exposure_remap: ExposureRemap::default(),
            platform_resolution: self.resolution(),
            platform_size: Vector3::new(
                parse(&self.slicer_info.display_width),
                parse(&self.slicer_info.display_height),
                build_height,
            ),
            slice_height: parse(&self.slicer_info.layer_height),
            exposure_config: self.exposure_config(false),
            first_exposure_config: self.exposure_config(true),
            first_layers: Height::Layers(self.slicer_info.bottom_layers as u32),
            transition_layers: Height::Layers(0),
        }
    }

    fn layer_count(&self) -> usize {
        self.layers.len()
    }

    fn layers(&self, progress: &Progress) -> Vec<slice::Layer> {
        progress.set_total(self.layers.len() as u64);
        let layer_height = self.slice_config().slice_height;
        let bottom_layers = self.slicer_info.bottom_layers as usize;
        let (bottom, normal) = (self.exposure_config(true), self.exposure_config(false));

        self.layers
            .iter()
            .enumerate()
            .map(|(index, layer)| {
                let runs = decode_lines(&layer.lines, self.resolution());
                let exposure = if index < bottom_layers { bottom.clone() } else { normal.clone() };
                slice::Layer::new(runs, layer_height * (index + 1) as f32, exposure)
            })
            .inspect(|_| progress.add_complete(1))
            .collect()
    }

    fn previews(&self) -> Vec<RgbaImage> {
        vec![
            self.large_previews[0].into_image(),
            self.small_preview.into_image(),
        ]
    }
}
