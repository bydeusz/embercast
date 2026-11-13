use anyhow::{Result, ensure};
use common::serde::{Deserializer, Serializer, SliceDeserializer};

use crate::{
    MAGIC, MAX_HEIGHT, MAX_WIDTH, PAGE_BREAK, VERSION,
    reader::{ensure_remaining, expect_page_break},
    strings::{read_terminated, read_utf16, write_terminated, write_utf16},
};

const HEADER_PADDING: usize = 64;
const SLICER_INFO_NUMBERS_SIZE: usize = 11 * 2;
const SLICER_INFO_V3_NUMBERS_SIZE: usize = 1 + 4 + 4 + 1 + 2 + 1 + 2 + 5;

/// The first block of the file: magic, version, printer model and resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    /// The Creality model code, such as `CL-79` for the Halot One Plus.
    pub printer_model: String,
    pub layer_count: u16,
    pub resolution_x: u16,
    pub resolution_y: u16,
}

/// The print settings, in the integer units of the format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicerInfo {
    /// The width of the screen in millimeters, as decimal text.
    pub display_width: String,
    /// The height of the screen in millimeters, as decimal text.
    pub display_height: String,
    /// The layer height in millimeters, as decimal text.
    pub layer_height: String,
    /// The exposure time of a normal layer, in tenths of a second.
    pub exposure_time: u16,
    /// The wait time before the light comes on, in whole seconds. The printer needs at least 1.
    pub wait_time_before_cure: u16,
    /// The exposure time of a bottom layer, in whole seconds.
    pub bottom_exposure_time: u16,
    pub bottom_layers: u16,
    /// The lift height after a bottom layer, in whole millimeters.
    pub bottom_lift_height: u16,
    /// The lift speed after a bottom layer, in whole millimeters per second.
    pub bottom_lift_speed: u16,
    /// The lift height after a normal layer, in whole millimeters.
    pub lift_height: u16,
    /// The lift speed after a normal layer, in whole millimeters per second.
    pub lift_speed: u16,
    /// The retract speed for all layers, in whole millimeters per second.
    pub retract_speed: u16,
    pub bottom_light_pwm: u16,
    pub light_pwm: u16,
}

/// The extra settings block of version 3 files.
///
/// The default values are the values that UVtools writes. Files with these
/// values print on Halot printers. The compensation values are 0, so the
/// printer does not change the image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicerInfoV3 {
    pub software_name: String,
    pub material_name: String,
    pub distortion_compensation_enabled: u8,
    pub distortion_compensation_thickness: u32,
    pub distortion_compensation_focal_length: u32,
    pub xy_profile_compensation_enabled: u8,
    pub xy_profile_compensation_value: u16,
    pub z_penetration_compensation_enabled: u8,
    pub z_penetration_compensation_level: u16,
    pub anti_alias_enabled: u8,
    pub anti_alias_gray_min: u8,
    pub anti_alias_gray_max: u8,
    pub image_blur_enabled: u8,
    pub image_blur_level: u8,
}

impl Header {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_be(MAGIC.len() as u32);
        ser.write_bytes(MAGIC);
        ser.write_u16_be(self.version);
        write_terminated(ser, &self.printer_model);
        ser.write_u16_be(self.layer_count);
        ser.write_u16_be(self.resolution_x);
        ser.write_u16_be(self.resolution_y);
        ser.write_bytes(&[0; HEADER_PADDING]);
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        ensure_remaining(des, 4 + MAGIC.len() + 2, "header")?;
        ensure!(des.read_u32_be() == MAGIC.len() as u32, "Not a CXDLP file: wrong magic size.");
        ensure!(des.read_slice(MAGIC.len()) == MAGIC, "Not a CXDLP file: wrong magic.");
        let version = des.read_u16_be();
        ensure!(matches!(version, 2 | 3), "CXDLP version {version} is not supported.");
        let printer_model = read_terminated(des)?;
        ensure_remaining(des, 6 + HEADER_PADDING, "header")?;
        let header = Self {
            version,
            printer_model,
            layer_count: des.read_u16_be(),
            resolution_x: des.read_u16_be(),
            resolution_y: des.read_u16_be(),
        };
        des.advance_by(HEADER_PADDING);
        let (x, y) = (header.resolution_x as u32, header.resolution_y as u32);
        ensure!((1..=MAX_WIDTH).contains(&x), "Resolution X of {x} is not supported.");
        ensure!((1..=MAX_HEIGHT).contains(&y), "Resolution Y of {y} is not supported.");
        Ok(header)
    }
}

impl SlicerInfo {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        write_utf16(ser, &self.display_width);
        write_utf16(ser, &self.display_height);
        write_utf16(ser, &self.layer_height);
        for value in [
            self.exposure_time,
            self.wait_time_before_cure,
            self.bottom_exposure_time,
            self.bottom_layers,
            self.bottom_lift_height,
            self.bottom_lift_speed,
            self.lift_height,
            self.lift_speed,
            self.retract_speed,
            self.bottom_light_pwm,
            self.light_pwm,
        ] {
            ser.write_u16_be(value);
        }
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        Ok(Self {
            display_width: read_utf16(des)?,
            display_height: read_utf16(des)?,
            layer_height: read_utf16(des)?,
            exposure_time: {
                ensure_remaining(des, SLICER_INFO_NUMBERS_SIZE, "slicer info")?;
                des.read_u16_be()
            },
            wait_time_before_cure: des.read_u16_be(),
            bottom_exposure_time: des.read_u16_be(),
            bottom_layers: des.read_u16_be(),
            bottom_lift_height: des.read_u16_be(),
            bottom_lift_speed: des.read_u16_be(),
            lift_height: des.read_u16_be(),
            lift_speed: des.read_u16_be(),
            retract_speed: des.read_u16_be(),
            bottom_light_pwm: des.read_u16_be(),
            light_pwm: des.read_u16_be(),
        })
    }
}

impl SlicerInfoV3 {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        write_terminated(ser, &self.software_name);
        write_terminated(ser, &self.material_name);
        ser.write_u8(self.distortion_compensation_enabled);
        ser.write_u32_le(self.distortion_compensation_thickness);
        ser.write_u32_le(self.distortion_compensation_focal_length);
        ser.write_u8(self.xy_profile_compensation_enabled);
        ser.write_u16_le(self.xy_profile_compensation_value);
        ser.write_u8(self.z_penetration_compensation_enabled);
        ser.write_u16_le(self.z_penetration_compensation_level);
        ser.write_u8(self.anti_alias_enabled);
        ser.write_u8(self.anti_alias_gray_min);
        ser.write_u8(self.anti_alias_gray_max);
        ser.write_u8(self.image_blur_enabled);
        ser.write_u8(self.image_blur_level);
        ser.write_bytes(PAGE_BREAK);
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        let info = Self {
            software_name: read_terminated(des)?,
            material_name: read_terminated(des)?,
            distortion_compensation_enabled: {
                ensure_remaining(des, SLICER_INFO_V3_NUMBERS_SIZE, "slicer info v3")?;
                des.read_u8()
            },
            distortion_compensation_thickness: des.read_u32_le(),
            distortion_compensation_focal_length: des.read_u32_le(),
            xy_profile_compensation_enabled: des.read_u8(),
            xy_profile_compensation_value: des.read_u16_le(),
            z_penetration_compensation_enabled: des.read_u8(),
            z_penetration_compensation_level: des.read_u16_le(),
            anti_alias_enabled: des.read_u8(),
            anti_alias_gray_min: des.read_u8(),
            anti_alias_gray_max: des.read_u8(),
            image_blur_enabled: des.read_u8(),
            image_blur_level: des.read_u8(),
        };
        expect_page_break(des, "slicer info v3")?;
        Ok(info)
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            version: VERSION,
            printer_model: String::new(),
            layer_count: 0,
            resolution_x: 0,
            resolution_y: 0,
        }
    }
}

impl Default for SlicerInfoV3 {
    fn default() -> Self {
        Self {
            software_name: String::new(),
            material_name: String::new(),
            distortion_compensation_enabled: 0,
            distortion_compensation_thickness: 600,
            distortion_compensation_focal_length: 300_000,
            xy_profile_compensation_enabled: 1,
            xy_profile_compensation_value: 0,
            z_penetration_compensation_enabled: 0,
            z_penetration_compensation_level: 1000,
            anti_alias_enabled: 1,
            anti_alias_gray_min: 1,
            anti_alias_gray_max: 255,
            image_blur_enabled: 0,
            image_blur_level: 2,
        }
    }
}
