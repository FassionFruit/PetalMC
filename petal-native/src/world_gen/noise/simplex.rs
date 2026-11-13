use super::gradient::{GRADIENTS, GradientTable};
use crate::world_gen::random::RandomSource;

#[derive(Clone, Debug)]
pub struct SimplexNoise {
    table: GradientTable,
}

impl SimplexNoise {
    pub const STANDARD_DEVIATION: f64 = 0.42544;

    pub fn new(random: &mut impl RandomSource) -> Self {
        Self::with_skipped_offsets(random, false)
    }

    pub fn with_skipped_offsets(random: &mut impl RandomSource, skip_offsets: bool) -> Self {
        Self {
            table: GradientTable::new(random, if skip_offsets { 0.0 } else { 256.0 }),
        }
    }

    pub fn table(&self) -> &GradientTable {
        &self.table
    }

    pub fn range(&self) -> [f32; 2] {
        [-2.0, 2.0]
    }

    fn corner(gradient: usize, point: [f64; 3], radius: f64) -> f64 {
        let attenuation = radius - point[0] * point[0] - point[1] * point[1] - point[2] * point[2];

        if attenuation < 0.0 {
            return 0.0;
        }

        let squared = attenuation * attenuation;
        let gradient = GRADIENTS[gradient];

        squared
            * squared
            * (f64::from(gradient[0]) * point[0]
                + f64::from(gradient[1]) * point[1]
                + f64::from(gradient[2]) * point[2])
    }

    pub fn get2(&self, x: f64, y: f64) -> f32 {
        self.get2_f64(x, y) as f32
    }

    fn get2_f64(&self, x: f64, y: f64) -> f64 {
        const SQRT_3: f64 = 1.7320508075688772;

        let x = x + self.table.offsets()[0];
        let y = y + self.table.offsets()[1];

        let skew = 0.5 * (SQRT_3 - 1.0);
        let unskew = (3.0 - SQRT_3) / 6.0;

        let shift = (x + y) * skew;
        let i = (x + shift).floor() as i32;
        let j = (y + shift).floor() as i32;

        let shift = f64::from(i.wrapping_add(j)) * unskew;
        let dx = x - (f64::from(i) - shift);
        let dy = y - (f64::from(j) - shift);

        let (a, b) = if dx > dy { (1, 0) } else { (0, 1) };

        let i = i & 255;
        let j = j & 255;

        let gradient = |a, b| (self.table.permute(i + a + self.table.permute(j + b)) % 12) as usize;

        let first = Self::corner(gradient(0, 0), [dx, dy, 0.0], 0.5);

        let second = Self::corner(
            gradient(a, b),
            [dx - f64::from(a) + unskew, dy - f64::from(b) + unskew, 0.0],
            0.5,
        );

        let third = Self::corner(
            gradient(1, 1),
            [dx - 1.0 + 2.0 * unskew, dy - 1.0 + 2.0 * unskew, 0.0],
            0.5,
        );

        70.0 * (first + second + third)
    }

    pub fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        self.get3_f64(x, y, z) as f32
    }

    fn get3_f64(&self, x: f64, y: f64, z: f64) -> f64 {
        let shifted = [
            x + self.table.offsets()[0],
            y + self.table.offsets()[1],
            z + self.table.offsets()[2],
        ];

        let skew = (shifted[0] + shifted[1] + shifted[2]) / 3.0;
        let cell = shifted.map(|value| (value + skew).floor() as i32);

        let unskew = f64::from(cell[0].wrapping_add(cell[1]).wrapping_add(cell[2])) / 6.0;

        let point: [f64; 3] = std::array::from_fn(|i| shifted[i] - (f64::from(cell[i]) - unskew));

        let [x, y, z] = point;

        let (a, b) = if x >= y {
            if y >= z {
                ([1, 0, 0], [1, 1, 0])
            } else if x >= z {
                ([1, 0, 0], [1, 0, 1])
            } else {
                ([0, 0, 1], [1, 0, 1])
            }
        } else if y < z {
            ([0, 0, 1], [0, 1, 1])
        } else if x < z {
            ([0, 1, 0], [0, 1, 1])
        } else {
            ([0, 1, 0], [1, 1, 0])
        };

        let cell = cell.map(|value| value & 255);

        let gradient = |delta: [i32; 3]| {
            (self.table.permute(
                cell[0]
                    + delta[0]
                    + self
                        .table
                        .permute(cell[1] + delta[1] + self.table.permute(cell[2] + delta[2])),
            ) % 12) as usize
        };

        let first = Self::corner(gradient([0; 3]), point, 0.6);

        let second = Self::corner(
            gradient(a),
            std::array::from_fn(|i| point[i] - f64::from(a[i]) + 1.0 / 6.0),
            0.6,
        );

        let third = Self::corner(
            gradient(b),
            std::array::from_fn(|i| point[i] - f64::from(b[i]) + 1.0 / 3.0),
            0.6,
        );

        let fourth = Self::corner(gradient([1; 3]), point.map(|value| value - 0.5), 0.6);

        32.0 * (first + second + third + fourth)
    }
}
