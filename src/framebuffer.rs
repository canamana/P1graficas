use raylib::prelude::*;

pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
    background: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, background: Color) -> Self {
        let pixels = vec![background; width * height];
        Framebuffer {
            width,
            height,
            pixels,
            background,
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.pixels {
            *pixel = self.background;
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.pixels[y * self.width + x];
                d.draw_pixel(x as i32, y as i32, color);
            }
        }
    }
}
