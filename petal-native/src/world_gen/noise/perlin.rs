use super::gradient::{GRADIENTS, GradientTable, wrap};
use crate::world_gen::random::RandomSource;

const PERLIN_GRADIENTS: [[f32; 3]; 16] = [
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

    fn coordinates(&self, point: [f64; 3]) -> ([i32; 3], [f64; 3]) {
        let offsets = self.table.offsets();

        let shifted = std::array::from_fn(|i| wrap(point[i]) + offsets[i]);

        let cell = shifted.map(|value| value.floor() as i32);

        let relative = std::array::from_fn(|i| shifted[i] - f64::from(cell[i]));

        (cell, relative)
    }

    fn gradients(&self, cell: [i32; 3]) -> [[f32; 3]; 8] {
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
}

#[inline]
fn smoothstep(value: f32) -> f32 {
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
fn lerp3(fade: [f32; 3], values: [f32; 8]) -> f32 {
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

#[derive(Clone, Debug)]
pub struct SmearedPerlinNoise {
    base: PerlinNoise,
    fudge_y_scale: f64,
}

impl SmearedPerlinNoise {
    pub fn new(random: &mut impl RandomSource, fudge_y_scale: f64) -> Self {
        Self {
            base: PerlinNoise::new(random),
            fudge_y_scale,
        }
    }

    pub fn get2(&self, x: f64, z: f64) -> f32 {
        self.get3(wrap(x), 0.0, wrap(z))
    }

    pub fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        let (cell, relative) = self.base.coordinates([x, y, z]);

        let fudge = compute_fudge_y(y, relative[1], self.fudge_y_scale);

        let point = [
            relative[0] as f32,
            (relative[1] - fudge) as f32,
            relative[2] as f32,
        ];

        let gradients = self.base.gradients(cell);

        let values = std::array::from_fn(|i| {
            let gradient = gradients[i];

            gradient[0] * (point[0] - (i & 1) as f32)
                + gradient[1] * (point[1] - ((i >> 1) & 1) as f32)
                + gradient[2] * (point[2] - ((i >> 2) & 1) as f32)
        });

        lerp3(
            [
                smoothstep(point[0]),
                smoothstep(relative[1] as f32),
                smoothstep(point[2]),
            ],
            values,
        )
    }

    pub fn table(&self) -> &GradientTable {
        self.base.table()
    }
}

fn compute_fudge_y(input_y: f64, relative_y: f64, scale: f64) -> f64 {
    let limited_y = if input_y >= 0.0 && input_y < relative_y {
        input_y
    } else {
        relative_y
    };

    ((limited_y / scale + f64::from(1e-7_f32)).floor()) * scale
}
