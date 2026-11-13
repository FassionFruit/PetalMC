use super::gradient::{GradientTable, wrap};
use crate::world_gen::density::DensityVolume;
use crate::world_gen::noise::{GRADIENTS, Noise};
use crate::world_gen::random::RandomSource;

pub(crate) const PERLIN_GRADIENTS: [[f32; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, -1.0],
];

#[derive(Clone, Debug)]
pub struct PerlinNoise {
    table: GradientTable,
}

impl PerlinNoise {
    pub fn new(random: &mut impl RandomSource) -> Self {
        Self {
            table: GradientTable::new(random, 256.0),
        }
    }

    pub fn table(&self) -> &GradientTable {
        &self.table
    }

    pub fn get2(&self, x: f64, z: f64) -> f32 {
        self.get3(wrap(x), 0.0, wrap(z))
    }

    pub fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        let (cell, relative) = self.coordinates([x, y, z]);
        let point = relative.map(|value| value as f32);

        let gradients = self.gradients(cell);

        let values = std::array::from_fn(|i| {
            let gradient = gradients[i];

            gradient[0] * (point[0] - (i & 1) as f32)
                + gradient[1] * (point[1] - ((i >> 1) & 1) as f32)
                + gradient[2] * (point[2] - ((i >> 2) & 1) as f32)
        });

        lerp3(
            [
                smoothstep(point[0]),
                smoothstep(point[1]),
                smoothstep(point[2]),
            ],
            values,
        )
    }

    pub(crate) fn coordinates(&self, point: [f64; 3]) -> ([i32; 3], [f64; 3]) {
        let offsets = self.table.offsets();

        let shifted = std::array::from_fn(|i| wrap(point[i]) + offsets[i]);

        let cell = shifted.map(|value| value.floor() as i32);

        let relative = std::array::from_fn(|i| shifted[i] - f64::from(cell[i]));

        (cell, relative)
    }

    pub(crate) fn gradients(&self, cell: [i32; 3]) -> [[f32; 3]; 8] {
        let p = self.table.permute(cell[0]);
        let q = self.table.permute(cell[0].wrapping_add(1));

        let lower = self.table.permute(p.wrapping_add(cell[1]));
        let upper = self.table.permute(p.wrapping_add(cell[1]).wrapping_add(1));

        let lower_x = self.table.permute(q.wrapping_add(cell[1]));
        let upper_x = self.table.permute(q.wrapping_add(cell[1]).wrapping_add(1));

        let rows = [
            lower, lower_x, upper, upper_x, lower, lower_x, upper, upper_x,
        ];

        std::array::from_fn(|i| {
            let index = self
                .table
                .permute(rows[i].wrapping_add(cell[2]).wrapping_add((i / 4) as i32))
                & 15;

            PERLIN_GRADIENTS[index as usize]
        })
    }

    pub fn add_to_volume(
        &self,
        buffer: &mut [f32],
        size_x: usize,
        size_y: usize,
        size_z: usize,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        assert_eq!(buffer.len(), size_x * size_y * size_z);

        let mut cached_xz = [0.0_f32; 8];
        let mut cached_y = [0.0_f32; 8];

        let mut index = 0;

        for iz in 0..size_z {
            let z = wrap(f64::from(min_z + iz as i32) * xz_scale) + self.table.offsets()[2];

            let floor_z = z.floor() as i32;
            let relative_z = (z - f64::from(floor_z)) as f32;
            let alpha_z = smoothstep(relative_z);

            for ix in 0..size_x {
                let x = wrap(f64::from(min_x + ix as i32) * xz_scale) + self.table.offsets()[0];

                let floor_x = x.floor() as i32;
                let relative_x = (x - f64::from(floor_x)) as f32;
                let alpha_x = smoothstep(relative_x);

                let x0 = self.table.permute(floor_x);
                let x1 = self.table.permute(floor_x + 1);

                let mut last_floor_y = i32::MIN;

                for iy in 0..size_y {
                    let y = wrap(f64::from(min_y + iy as i32) * y_scale) + self.table.offsets()[1];

                    let floor_y = y.floor() as i32;
                    let relative_y = (y - f64::from(floor_y)) as f32;
                    let alpha_y = smoothstep(relative_y);

                    if last_floor_y != floor_y {
                        let xy00 = self.table.permute(x0 + floor_y);
                        let xy01 = self.table.permute(x0 + floor_y + 1);
                        let xy10 = self.table.permute(x1 + floor_y);
                        let xy11 = self.table.permute(x1 + floor_y + 1);

                        let indices = [
                            xy00 + floor_z,
                            xy10 + floor_z,
                            xy01 + floor_z,
                            xy11 + floor_z,
                            xy00 + floor_z + 1,
                            xy10 + floor_z + 1,
                            xy01 + floor_z + 1,
                            xy11 + floor_z + 1,
                        ];

                        for i in 0..8 {
                            let gradient =
                                PERLIN_GRADIENTS[(self.table.permute(indices[i]) & 15) as usize];

                            let dx = relative_x - (i & 1) as f32;
                            let dz = relative_z - ((i >> 2) & 1) as f32;

                            cached_xz[i] = gradient[0] * dx + gradient[2] * dz;
                            cached_y[i] = gradient[1];
                        }

                        last_floor_y = floor_y;
                    }

                    let values = [
                        cached_xz[0] + cached_y[0] * relative_y,
                        cached_xz[1] + cached_y[1] * relative_y,
                        cached_xz[2] + cached_y[2] * (relative_y - 1.0),
                        cached_xz[3] + cached_y[3] * (relative_y - 1.0),
                        cached_xz[4] + cached_y[4] * relative_y,
                        cached_xz[5] + cached_y[5] * relative_y,
                        cached_xz[6] + cached_y[6] * (relative_y - 1.0),
                        cached_xz[7] + cached_y[7] * (relative_y - 1.0),
                    ];

                    buffer[index] += amplitude * lerp3([alpha_x, alpha_y, alpha_z], values);

                    index += 1;
                }
            }
        }
    }
}

#[inline]
pub(crate) fn smoothstep(value: f32) -> f32 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

#[inline]
fn lerp2(a: f32, b: f32, values: [f32; 4]) -> f32 {
    lerp(
        b,
        lerp(a, values[0], values[1]),
        lerp(a, values[2], values[3]),
    )
}

#[inline]
pub(crate) fn lerp3(fade: [f32; 3], values: [f32; 8]) -> f32 {
    lerp(
        fade[2],
        lerp2(
            fade[0],
            fade[1],
            [values[0], values[1], values[2], values[3]],
        ),
        lerp2(
            fade[0],
            fade[1],
            [values[4], values[5], values[6], values[7]],
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_gen::random::LegacyRandomSource;

    #[test]
    fn add_to_volume_matches_get3() {
        let mut random = LegacyRandomSource::new(12345);
        let noise = PerlinNoise::new(&mut random);

        let cases = [(1.0, 1.0), (0.5, 0.25), (0.125, 0.5), (2.0, 1.5)];

        for (xz_scale, y_scale) in cases {
            let (size_x, size_y, size_z) = (7, 9, 5);
            let (min_x, min_y, min_z) = (-13, -32, 21);

            let mut actual = vec![0.0; size_x * size_y * size_z];

            noise.add_to_volume(
                &mut actual,
                size_x,
                size_y,
                size_z,
                min_x,
                min_y,
                min_z,
                xz_scale,
                y_scale,
                1.0,
            );

            let mut index = 0;

            for z in 0..size_z {
                for x in 0..size_x {
                    for y in 0..size_y {
                        let expected = noise.get3(
                            f64::from(min_x + x as i32) * xz_scale,
                            f64::from(min_y + y as i32) * y_scale,
                            f64::from(min_z + z as i32) * xz_scale,
                        );

                        assert!(
                            (actual[index] - expected).abs() < 1e-6,
                            "Mismatch at ({x}, {y}, {z}): actual={}, expected={expected}",
                            actual[index],
                        );

                        index += 1;
                    }
                }
            }
        }
    }
}
