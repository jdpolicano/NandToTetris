pub struct Dimension {
    width: usize,
    height: usize,
}

impl Dimension {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn logical_width(&self) -> usize {
        self.width
    }

    pub fn logical_height(&self) -> usize {
        self.height
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::new(512, 256)
    }
}

pub struct HackScreenBuffer<'a> {
    buf: &'a [i16],
    byte_idx: usize,
    bit_idx: usize,
}

impl<'a> HackScreenBuffer<'a> {
    pub fn new(buf: &'a [i16]) -> Self {
        Self {
            buf,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    fn next_byte(&mut self) {
        self.byte_idx += 1;
    }

    fn next_bit(&mut self) {
        self.bit_idx += 1;
        if self.bit_idx == i16::BITS as usize {
            self.bit_idx = 0;
            self.next_byte();
        }
    }
}

impl Iterator for HackScreenBuffer<'_> {
    type Item = (u8, u8, u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_idx >= self.buf.len() {
            return None;
        }

        let byte = self.buf[self.byte_idx];
        let mask = 1 << (self.bit_idx);

        let color = if byte & mask == 0 {
            (255, 255, 255, 255) // White pixel
        } else {
            (0, 0, 0, 255) // Black pixel
        };

        self.next_bit();
        Some(color)
    }
}

/// transposes from one iterator over pixels to another iterator over pixels of the same image
/// but with a different scale factor. It is crucial that the input iterator is a multiple of the
/// output iterator.
pub struct Scaler {
    logical_width: usize,
    logical_height: usize,
    scale_factor_x: f64,
    scale_factor_y: f64,
}

impl Scaler {
    pub fn new(scale_factor_x: f64, scale_factor_y: f64, dim: &Dimension) -> Self {
        Self {
            logical_width: dim.logical_width(),
            logical_height: dim.logical_height(),
            scale_factor_x,
            scale_factor_y,
        }
    }

    pub fn scale<A>(&self, src: A, target: &mut [u8]) -> Result<(), String>
    where
        A: Iterator<Item = (u8, u8, u8, u8)>,
    {
        // Calculate physical dimensions using floating-point multiplication
        let physical_width = (self.logical_width as f64 * self.scale_factor_x).ceil() as usize;
        let physical_height = (self.logical_height as f64 * self.scale_factor_y).ceil() as usize;
        let required_size = physical_width * physical_height * 4;

        if required_size != target.len() {
            return Err(format!(
                "Output buffer has wrong size: expected {}, got {}",
                required_size,
                target.len()
            ));
        }

        // Collect logical pixels into a vector for indexing
        let logical_pixels: Vec<(u8, u8, u8, u8)> = src.collect();

        // Iterate over each physical pixel
        for y_physical in 0..physical_height {
            for x_physical in 0..physical_width {
                // Map physical pixel back to logical pixel using inverse scaling
                let x_logical_f = x_physical as f64 / self.scale_factor_x;
                let y_logical_f = y_physical as f64 / self.scale_factor_y;

                // Use floor to get the nearest logical pixel index (nearest-neighbor interpolation)
                let x_logical = x_logical_f.floor() as usize;
                let y_logical = y_logical_f.floor() as usize;

                // Ensure indices are within bounds
                if x_logical >= self.logical_width || y_logical >= self.logical_height {
                    continue; // Skip pixels that map outside the logical image
                }

                let idx_logical = y_logical * self.logical_width + x_logical;
                let (r, g, b, a) = logical_pixels[idx_logical];

                // Calculate the index in the target buffer
                let idx_physical = (y_physical * physical_width + x_physical) * 4;
                target[idx_physical] = r;
                target[idx_physical + 1] = g;
                target[idx_physical + 2] = b;
                target[idx_physical + 3] = a;
            }
        }

        Ok(())
    }
}

// impl Scaler {
//     pub fn new(scale_factor_x: f64, scale_factor_y: f64, dim: &Dimension) -> Self {
//         Self {
//             logical_width: dim.logical_width(),
//             logical_height: dim.logical_height(),
//             scale_factor_x,
//             scale_factor_y,
//         }
//     }

//     pub fn scale<A>(&self, src: A, target: &mut [u8]) -> Result<(), String>
//     where
//         A: Iterator<Item = (u8, u8, u8, u8)>,
//     {
//         let physical_width = self.logical_width * self.scale_factor_x as usize;
//         let physical_height = self.logical_height * self.scale_factor_y as usize;
//         let required_size = physical_width * physical_height * 4;

//         if required_size != target.len() {
//             return Err(format!(
//                 "Output buffer has wrong size: expected {}, got {}",
//                 required_size,
//                 target.len()
//             ));
//         }

//         for (idx, (r, g, b, a)) in src.enumerate() {
//             let x_logical = idx % self.logical_width;
//             let y_logical = idx / self.logical_width;
//             for i in 0..self.scale_factor_x as usize {
//                 for j in 0..self.scale_factor_y as usize {
//                     let x_physical = x_logical * (self.scale_factor_x as usize) + i;
//                     let y_physical = y_logical * (self.scale_factor_y as usize) + j;
//                     let idx_physical = (y_physical * physical_width + x_physical) * 4;
//                     // No need for additional bounds checking since we checked buffer size
//                     target[idx_physical] = r;
//                     target[idx_physical + 1] = g;
//                     target[idx_physical + 2] = b;
//                     target[idx_physical + 3] = a;
//                 }
//             }
//         }

//         Ok(())
//     }
// }

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn test_dimension() {
        let dim = Dimension::new(512, 256);
        assert_eq!(dim.size(), 512 * 256);
        assert_eq!(dim.logical_width(), 512);
        assert_eq!(dim.logical_height(), 256);
    }

    #[test]
    fn test_scaler() {
        let dim = Dimension::new(2, 2);
        let scale_factor_x = 2 as f64;
        let scale_factor_y = 2 as f64;
        let scaler = Scaler::new(scale_factor_x, scale_factor_y, &dim);

        // expected input:
        // Black Black
        // Black White
        //
        // expected output:
        // Black Black Black Black
        // Black Black Black Black
        // Black Black White White
        // Black Black White White
        let src = vec![
            (0, 0, 0, 255),
            (0, 0, 0, 255),
            (0, 0, 0, 255),
            (255, 255, 255, 255),
        ];

        let mut target = vec![0; 16 * 4];

        let is_white =
            |chunk: &[u8]| chunk[0] == 255 && chunk[1] == 255 && chunk[2] == 255 && chunk[3] == 255;

        let is_black =
            |chunk: &[u8]| chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 255;

        scaler.scale(src.into_iter(), &mut target).unwrap();

        let scaled = target
            .chunks_exact(4)
            .map(|chunk| {
                if is_white(chunk) {
                    "White".to_string()
                } else if is_black(chunk) {
                    "Black".to_string()
                } else {
                    panic!("Unexpected color: {:?}", chunk);
                }
            })
            .collect::<Vec<String>>();

        let expected = vec![
            "Black", "Black", "Black", "Black", "Black", "Black", "Black", "Black", "Black",
            "Black", "White", "White", "Black", "Black", "White", "White",
        ];

        assert_eq!(scaled, expected);
    }

    #[test]
    fn test_scaler_2() {
        let dim = Dimension::new(2, 2);
        let scale_factor_x = 2 as f64;
        let scale_factor_y = 2 as f64;
        let scaler = Scaler::new(scale_factor_x, scale_factor_y, &dim);

        // expected input:
        // White Black
        // Black White
        //
        // expected output:
        // White White Black Black
        // White White Black Black
        // Black Black White White
        // Black Black White White
        let src = vec![
            (255, 255, 255, 255),
            (0, 0, 0, 255),
            (0, 0, 0, 255),
            (255, 255, 255, 255),
        ];

        let mut target = vec![0; 16 * 4];

        let is_white =
            |chunk: &[u8]| chunk[0] == 255 && chunk[1] == 255 && chunk[2] == 255 && chunk[3] == 255;

        let is_black =
            |chunk: &[u8]| chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 255;

        scaler.scale(src.into_iter(), &mut target).unwrap();

        let scaled = target
            .chunks_exact(4)
            .map(|chunk| {
                if is_white(chunk) {
                    "White".to_string()
                } else if is_black(chunk) {
                    "Black".to_string()
                } else {
                    panic!("Unexpected color: {:?}", chunk);
                }
            })
            .collect::<Vec<String>>();

        let expected = vec![
            "White", "White", "Black", "Black", "White", "White", "Black", "Black", "Black",
            "Black", "White", "White", "Black", "Black", "White", "White",
        ];

        assert_eq!(scaled, expected);
    }

    #[test]
    fn test_scaler_3() {
        let dim = Dimension::new(2, 2);
        let scale_factor_x = 1.5;
        let scale_factor_y = 1.5;
        let scaler = Scaler::new(scale_factor_x, scale_factor_y, &dim);
        // expected input:
        // White Black
        // Black White
        //
        // expected output:
        // White White Black
        // White White Black
        // Black Black White

        let src = vec![
            (255, 255, 255, 255),
            (0, 0, 0, 255),
            (0, 0, 0, 255),
            (255, 255, 255, 255),
        ];

        let mut target = vec![0; 9 * 4];

        let is_white =
            |chunk: &[u8]| chunk[0] == 255 && chunk[1] == 255 && chunk[2] == 255 && chunk[3] == 255;

        let is_black =
            |chunk: &[u8]| chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 && chunk[3] == 255;

        scaler.scale(src.into_iter(), &mut target).unwrap();

        let scaled = target
            .chunks_exact(4)
            .map(|chunk| {
                if is_white(chunk) {
                    "White".to_string()
                } else if is_black(chunk) {
                    "Black".to_string()
                } else {
                    panic!("Unexpected color: {:?}", chunk);
                }
            })
            .collect::<Vec<String>>();

        let expected = vec![
            "White", "White", "Black", "White", "White", "Black", "Black", "Black", "White",
        ];

        assert_eq!(scaled, expected);
    }
}
