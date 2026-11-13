use std::fmt::Debug;

use common::serde::{Deserializer, Serializer, SliceDeserializer};
use image::{Rgba, RgbaImage, imageops::FilterType};

/// One preview image in RGB565, two bytes per pixel, big endian.
#[derive(Clone)]
pub struct PreviewImage<const WIDTH: usize, const HEIGHT: usize> {
    data: Box<[u16]>,
}

impl<const WIDTH: usize, const HEIGHT: usize> PreviewImage<WIDTH, HEIGHT> {
    /// Return a black preview image.
    pub fn empty() -> Self {
        Self {
            data: vec![0; WIDTH * HEIGHT].into_boxed_slice(),
        }
    }

    /// Return the preview image of `image` after the function scales it to the preview size.
    pub fn from_image_scaled(image: &RgbaImage, filter: FilterType) -> Self {
        let scaled = image::imageops::resize(image, WIDTH as u32, HEIGHT as u32, filter);
        let data = scaled
            .pixels()
            .map(|pixel| {
                let red = (pixel[0] as u16) >> 3;
                let green = (pixel[1] as u16) >> 2;
                let blue = (pixel[2] as u16) >> 3;
                (red << 11) | (green << 5) | blue
            })
            .collect();
        Self { data }
    }

    /// Return the preview image as an RGBA image.
    pub fn into_image(&self) -> RgbaImage {
        RgbaImage::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
            let pixel = self.data[y as usize * WIDTH + x as usize];
            let red = ((pixel >> 11) & 0x1F) as u32 * 255 / 31;
            let green = ((pixel >> 5) & 0x3F) as u32 * 255 / 63;
            let blue = (pixel & 0x1F) as u32 * 255 / 31;
            Rgba([red as u8, green as u8, blue as u8, 255])
        })
    }

    /// Return the RGB565 pixels in row order.
    pub fn inner_data(&self) -> &[u16] {
        &self.data
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        for pixel in self.data.iter() {
            ser.write_u16_be(*pixel);
        }
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Self {
        let data = (0..WIDTH * HEIGHT).map(|_| des.read_u16_be()).collect();
        Self { data }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Debug for PreviewImage<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewImage")
            .field("width", &WIDTH)
            .field("height", &HEIGHT)
            .finish()
    }
}
