use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

pub struct Kanji<'a> {
    left: i32,
    top: i32,
    data: &'a [u8],
}

impl Drawable for Kanji<'_> {
    type Color = BinaryColor;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: DrawTarget<Color = BinaryColor> {
        let mut x = self.left;
        let mut y = self.top;
        let mut t = self.top;
        self.byte_to_bool_array().into_iter().map(|r| {
            let row_bxy = r.map(|b| {
                let bxy = (b, x, y);
                y += 1;
                bxy
            });
            y = t;
            if x < self.left + 15{
                x += 1;
            } else {
                x = self.left;
                t += 8;
            }
            row_bxy
        }).flatten().filter(|f| f.0)
        .map(|t| Pixel(Point::new(t.1, t.2), BinaryColor::On))
        .draw(target)?;
        Ok(())
    }
}

impl Kanji<'_> {
    pub fn dot_matrix(x: i32, y: i32, d: &[u8]) -> Kanji {
        Kanji {
            left: x,
            top: y,
            data: d,
        }
    }

    fn byte_to_bool_array(&self) -> Vec<[bool; 8]> {
        self.data.iter().map(|u| {
            let mut bool_array = [false; 8];
    
            for i in 0..8 {
                let bitmask = 1 << i;
                bool_array[i] = (u & bitmask) != 0;
            }
        
            bool_array
        }).collect()
    }
}