use crate::utility::*;

use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::error::Error;

#[derive(Default)]
pub struct RgbaImage {
    width: usize,
    height: usize,
    data: Vec<[u8; 4]>,
}

impl RgbaImage {
    /// Create an empty image with the given width and height
    pub fn new(width: usize, height: usize) -> Self {
        RgbaImage {
            width, height,
            data: vec![[0, 0, 0, 0]; width * height]
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn aspect_ratio(&self) -> Real {
        self.width as Real / self.height as Real
    }

    /// Get the sample coordinates of a pixel, in the range [0, 1]
    pub fn sample(&self, i: usize, j: usize) -> Point2 {
        point2(
            i as Real / self.width as Real,
            j as Real / self.height as Real
        )
    }

    /// Get multiple samples coordinates for a pixel, in the range [0, 1]
    pub fn samples_jitter(&self, i: usize, j: usize, num_samples: usize) -> Vec<Point2> {
        (0..num_samples).map(|_| {
            point2(
                (i as Real + rand::random::<Real>()) / self.width as Real,
                (j as Real + rand::random::<Real>()) / self.height as Real
            )
        }).collect()
    }

    /// Access a pixel immutably
    pub fn get(&self, i: usize, j: usize) -> &[u8; 4] {
        &self.data[i + j * self.width]
    }

    /// Access a pixel mutably
    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut [u8; 4] {
        &mut self.data[i + j * self.width]
    }
}

pub mod tga {
    use std::convert::TryInto;
    use super::*;

    #[repr(C)]
    #[derive(Default, Debug)]
    // See http://paulbourke.net/dataformats/tga/
    struct TgaHeader {
        id_length:      u8,
        colormap_type:  u8,
        datatype_code:  u8,
        colormap_spec:  [u8; 5],
        x_origin:       u16,
        y_origin:       u16,
        width:          u16,
        height:         u16,
        bits_per_pixel: u8,
        image_desc:     u8,
    }

    impl TgaHeader {
        fn buffer(&mut self) -> &mut [u8] {
            unsafe {
                std::slice::from_raw_parts_mut(self as *mut _ as *mut u8, std::mem::size_of::<Self>())
            }
        }
    }

    pub fn load(path: &str) -> Result<RgbaImage, Box<dyn Error>> {
        let mut file = BufReader::new(File::open(path)?);
        
        // Read header
        let mut header = TgaHeader::default();
        file.read_exact(header.buffer())?;

        // Check header
        let mut header_ok = true;
        header_ok &= header.id_length == 0;
        header_ok &= header.colormap_type == 0;
        header_ok &= header.datatype_code == 2; // 2 = uncompressed color data
        header_ok &= header.image_desc == 0 || header.image_desc == 1 << 5; // any image origin is allowed
        header_ok &= header.bits_per_pixel == 24 || header.bits_per_pixel == 32; // BGR or BGRA
        if !header_ok {
            return Err(format!("This tga header is not supported: {:?}", header).into())
        }

        // Read data
        let mut image = RgbaImage::default();
        image.width = header.width.into();
        image.height = header.height.into();
        image.data.resize(image.width * image.height, [0; 4]);
        for y in 0..image.height {
            for x in 0..image.width {
                // To flip vertically or not
                let y = if header.image_desc == 1 << 5 {
                    image.height - 1 - y
                } else {
                    y
                };
                if header.bits_per_pixel == 32 {
                    // BGRA
                    let mut bgra = [0; 4];
                    file.read_exact(&mut bgra)?;
                    *image.get_mut(x, y) = [bgra[2], bgra[1], bgra[0], bgra[3]];
                } else if header.bits_per_pixel == 24 { 
                    // BGR
                    let mut bgr = [0; 3];
                    file.read_exact(&mut bgr)?;
                    *image.get_mut(x, y) = [bgr[2], bgr[1], bgr[0], 0xff];
                }
            }
        }
        Ok(image)
    }

    pub fn save(image: &RgbaImage, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = BufWriter::new(File::create(path)?);
        let mut header = TgaHeader::default();

        // Fill header
        header.datatype_code = 2; // 2 = uncompressed color data
        header.bits_per_pixel = 32; // BGRA
        header.width = image.width().try_into()?;
        header.height = image.height().try_into()?;

        // Write header
        file.write(header.buffer())?;

        // Write data
        for y in 0..image.height {
            for x in 0..image.width {
                let rgba = image.get(x, y);
                file.write(&[rgba[2], rgba[1], rgba[0], rgba[3]])?;
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn tga_header_size() {
            assert_eq!(std::mem::size_of::<TgaHeader>(), 18)
        }
    }  
}