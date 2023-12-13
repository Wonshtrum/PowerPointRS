use std::fmt::Write;

use crate::Color;

pub struct Canvas<Pixel> {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

fn clamp(x: isize, min: isize, max: isize) -> isize {
    if x < min {
        return min;
    }
    if x > max {
        return max;
    }
    x
}

impl<Pixel: Copy> Canvas<Pixel> {
    pub fn new(width: usize, height: usize, background: Pixel) -> Self {
        Self {
            width,
            height,
            pixels: vec![background; width * height],
        }
    }
    pub fn fill(&mut self, background: Pixel) {
        self.pixels.fill(background);
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[y * self.width + x] = pixel;
    }
    pub fn fill_rect(&mut self, x: isize, y: isize, w: isize, h: isize, pixel: Pixel) {
        let x1 = clamp(x, 0, self.width as isize) as usize;
        let y1 = clamp(y, 0, self.height as isize) as usize;
        let x2 = clamp(x + w, 0, self.width as isize) as usize;
        let y2 = clamp(y + h, 0, self.height as isize) as usize;
        for y in y1..y2 {
            let line = y * self.width;
            for x in x1..x2 {
                unsafe { *self.pixels.get_unchecked_mut(line + x) = pixel };
            }
        }
    }
}

impl Canvas<Color> {
    pub fn to_string(&self) -> String {
        let mut string = String::new(); //String::with_capacity((self.width + 1) * (self.height / 2));
        for y in 0..(self.height / 2) {
            let line_top = y * 2 * self.width;
            let line_bot = (y * 2 + 1) * self.width;
            for x in 0..self.width {
                unsafe {
                    let Color {
                        r: r_bg,
                        g: g_bg,
                        b: b_bg,
                    } = self.pixels.get_unchecked(line_top + x);
                    let Color {
                        r: r_fg,
                        g: g_fg,
                        b: b_fg,
                    } = self.pixels.get_unchecked(line_bot + x);
                    string
                        .write_fmt(format_args!(
                            "\x1b[48;2;{r_bg};{g_bg};{b_bg};38;2;{r_fg};{g_fg};{b_fg}m▄"
                        ))
                        .unwrap();
                }
            }
            string.push_str("\x1b[0m\n");
        }
        string
    }
}
