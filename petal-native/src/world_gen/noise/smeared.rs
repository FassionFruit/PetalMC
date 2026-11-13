use crate::world_gen::density::DensityVolume;
use crate::world_gen::noise::{wrap, PerlinNoise, GradientTable, GRADIENTS};
use crate::world_gen::random::RandomSource;

#[derive(Clone, Debug)]
pub struct SmearedPerlinNoise {
    base: PerlinNoise,
    pub(crate) fudge_y_scale: f64,
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

        crate::world_gen::noise::perlin::lerp3(
            [
                crate::world_gen::noise::perlin::smoothstep(point[0]),
                crate::world_gen::noise::perlin::smoothstep(relative[1] as f32),
                crate::world_gen::noise::perlin::smoothstep(point[2]),
            ],
            values,
        )
    }

    pub fn table(&self) -> &GradientTable {
        self.base.table()
    }

    pub fn compute_fudge_y(&self, original_y: f64, relative_y: f64) -> f64 {
        let fudge_limit = if original_y >= 0.0 && original_y < relative_y {
            original_y
        } else {
            relative_y
        };

        (fudge_limit / self.fudge_y_scale + 1.0e-7_f32 as f64).floor()
            * self.fudge_y_scale
    }

    pub fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        use super::gradient::wrap;
        use super::perlin::{lerp3, smoothstep, PERLIN_GRADIENTS};

        let table = self.base.table();
        let offsets = table.offsets();

        let mut cached_xz = [0.0_f32; 8];
        let mut cached_y = [0.0_f32; 8];

        let mut index = 0;

        for index_z in 0..volume.size_z {
            let z = wrap(f64::from(volume.block_z(index_z)) * xz_scale) + offsets[2];
            let floor_z = z.floor() as i32;
            let relative_z = (z - f64::from(floor_z)) as f32;
            let alpha_z = smoothstep(relative_z);

            for index_x in 0..volume.size_x {
                let x = wrap(f64::from(volume.block_x(index_x)) * xz_scale) + offsets[0];
                let floor_x = x.floor() as i32;
                let relative_x = (x - f64::from(floor_x)) as f32;

                let x0 = table.permute(floor_x);
                let x1 = table.permute(floor_x + 1);

                let alpha_x = smoothstep(relative_x);
                let mut last_floor_y = i32::MIN;

                for index_y in 0..volume.size_y {
                    let original_y = f64::from(volume.block_y(index_y)) * y_scale;
                    let y = wrap(original_y) + offsets[1];

                    let floor_y = y.floor() as i32;
                    let relative_y = y - f64::from(floor_y);
                    let relative_y_f32 = relative_y as f32;
                    let alpha_y = smoothstep(relative_y_f32);

                    if last_floor_y != floor_y {
                        let xy00 = table.permute(x0 + floor_y);
                        let xy01 = table.permute(x0 + floor_y + 1);
                        let xy10 = table.permute(x1 + floor_y);
                        let xy11 = table.permute(x1 + floor_y + 1);

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
                            let gradient_index =
                                (table.permute(indices[i]) & 15) as usize;

                            let gradient = PERLIN_GRADIENTS[gradient_index];

                            let dx = relative_x - (i & 1) as f32;
                            let dz = relative_z - ((i >> 2) & 1) as f32;

                            cached_xz[i] = gradient[0] * dx + gradient[2] * dz;
                            cached_y[i] = gradient[1];
                        }

                        last_floor_y = floor_y;
                    }

                    let fudged_relative_y = (
                        relative_y
                            - self.compute_fudge_y(original_y, relative_y)
                    ) as f32;

                    let values = std::array::from_fn(|i| {
                        cached_xz[i]
                            + cached_y[i]
                            * (fudged_relative_y - ((i >> 1) & 1) as f32)
                    });

                    let value = lerp3(
                        [alpha_x, alpha_y, alpha_z],
                        values,
                    );

                    buffer[index] += amplitude * value;
                    index += 1;
                }
            }
        }
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
