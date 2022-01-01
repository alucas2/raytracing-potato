/*
In this file:
- Image storage
- Image saving and loading
- Image tiling
*/

// ------------------------------------------- Image storage -------------------------------------------

#[derive(Debug, Clone)]
pub struct Array2d<T> {
    width: u32,
    height: u32,
    storage: Vec<T>,
}

impl<'a, T: Clone + Default + 'a> Array2d<T> {
    pub fn new(width: u32, height: u32) -> Self {
        let storage = vec![T::default(); (width * height) as usize];
        Array2d {width, height, storage}
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get(&self, i: u32, j: u32) -> &T {
        &self.storage[(i + j * self.width) as usize]
    }

    pub fn get_mut(&mut self, i: u32, j: u32) -> &mut T {
        &mut self.storage[(i + j * self.width) as usize]
    }
}

// ------------------------------------------- Image loading and saving -------------------------------------------

pub mod tga {
    use super::*;
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::{Read, Write, BufReader, BufWriter};
    use std::error::Error;

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

    pub fn load(path: &str) -> Result<Array2d<[u8; 4]>, Box<dyn Error>> {
        let mut file = BufReader::new(File::open(path)?);
        
        // Read header
        let mut header = TgaHeader::default();
        file.read_exact(header.buffer())?;

        // Check header
        let mut header_ok = true;
        header_ok &= header.id_length == 0;
        header_ok &= header.colormap_type == 0;
        header_ok &= header.datatype_code == 2; // 2 = uncompressed color data
        header_ok &= header.bits_per_pixel == 24 || header.bits_per_pixel == 32; // BGR or BGRA
        if !header_ok {
            return Err(format!("This tga header is not supported: {:?}", header).into())
        }

        // Read data
        let mut image = Array2d::new(header.width as u32, header.height as u32);
        for y in 0..image.height {
            for x in 0..image.width {
                // To flip vertically or not
                let y = if (header.image_desc & 1 << 5) != 0 {
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

    pub fn save(image: &Array2d<[u8; 4]>, path: &str) -> Result<(), Box<dyn Error>> {
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
}

// ------------------------------------------- Image tiling -------------------------------------------

#[derive(Debug, Clone)]
pub struct Tile {
    pub offset_i: u32,
    pub offset_j: u32,
    pub width: u32,
    pub height: u32,
}

impl Tile {
    pub fn split_in_tiles(full_width: u32, full_height: u32, tile_width: u32, tile_height: u32) -> Vec<Tile>
    {
        let num_tiles_i = (full_width + tile_width - 1) / tile_width;
        let num_tiles_j = (full_height + tile_height - 1) / tile_height;
        let mut tile_descriptions = Vec::new();
        
        for tj in 0..num_tiles_j {
            for ti in 0..num_tiles_i {
                let offset_i = ti * tile_width;
                let offset_j = tj * tile_height;
                let width = tile_width.min(full_width - offset_i);
                let height = tile_height.min(full_height - offset_j);
                tile_descriptions.push(Tile {offset_i, offset_j, width, height}); 
            }
        }
        tile_descriptions
    }
}
