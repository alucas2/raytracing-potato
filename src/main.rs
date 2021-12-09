use raytracing2::image::{tga, RgbaImage};

fn main() {
    let mut output = RgbaImage::new(256, 256);

    for y in (0..output.height()).rev() {
        for x in 0..output.width() {
            let r = x as f32 / output.width() as f32;
            let g = y as f32 / output.height() as f32;
            let b = 0.25;
            let a = 1.0;
            *output.get_mut(x, y) = [(255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8, (255.0 * a) as u8];
        }
    }

    tga::save(&output, "output.tga").unwrap();
}
