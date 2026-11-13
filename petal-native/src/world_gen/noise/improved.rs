use crate::world_gen::random::RandomSource;

const GRADIENTS: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

pub struct ImprovedNoise {
    permutation: [u8; 256],
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
}

impl ImprovedNoise {
    pub fn new(random: &mut impl RandomSource) -> Self {
        let xo = random.next_f64() * 256.0;
        let yo = random.next_f64() * 256.0;
        let zo = random.next_f64() * 256.0;

        let mut permutation = [0u8; 256];

        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }

        for index in 0..256 {
            let offset = random.next_i32_bound((256 - index) as u32) as usize;
            permutation.swap(index, index + offset);
        }

        Self {
            permutation,
            xo,
            yo,
            zo,
        }
    }

    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        self.noise_with_y_scale(x, y, z, 0.0, 0.0)
    }

    pub fn noise_with_y_scale(&self, x: f64, y: f64, z: f64, y_scale: f64, y_max: f64) -> f64 {
        let x = x + self.xo;
        let y = y + self.yo;
        let z = z + self.zo;

        let floor_x = floor(x);
        let floor_y = floor(y);
        let floor_z = floor(z);

        let local_x = x - floor_x as f64;
        let local_y = y - floor_y as f64;
        let local_z = z - floor_z as f64;

        let adjusted_y = if y_scale != 0.0 {
            let clamped = if y_max >= 0.0 && y_max < local_y {
                y_max
            } else {
                local_y
            };

            floor(clamped / y_scale + 1.0e-7) as f64 * y_scale
        } else {
            0.0
        };

        self.sample_and_lerp(
            floor_x,
            floor_y,
            floor_z,
            local_x,
            local_y - adjusted_y,
            local_z,
            local_y,
        )
    }

    fn permutation(&self, index: i32) -> usize {
        self.permutation[(index & 255) as usize] as usize
    }

    fn sample_and_lerp(
        &self,
        x: i32,
        y: i32,
        z: i32,
        local_x: f64,
        local_y: f64,
        local_z: f64,
        interpolation_y: f64,
    ) -> f64 {
        let p_x = self.permutation(x);
        let p_x1 = self.permutation(x + 1);

        let p_xy = self.permutation(p_x as i32 + y);
        let p_xy1 = self.permutation(p_x as i32 + y + 1);
        let p_x1y = self.permutation(p_x1 as i32 + y);
        let p_x1y1 = self.permutation(p_x1 as i32 + y + 1);

        let n000 = self.gradient(p_xy as i32 + z, local_x, local_y, local_z);
        let n100 = self.gradient(p_x1y as i32 + z, local_x - 1.0, local_y, local_z);
        let n010 = self.gradient(p_xy1 as i32 + z, local_x, local_y - 1.0, local_z);
        let n110 = self.gradient(p_x1y1 as i32 + z, local_x - 1.0, local_y - 1.0, local_z);

        let n001 = self.gradient(p_xy as i32 + z + 1, local_x, local_y, local_z - 1.0);
        let n101 = self.gradient(p_x1y as i32 + z + 1, local_x - 1.0, local_y, local_z - 1.0);
        let n011 = self.gradient(p_xy1 as i32 + z + 1, local_x, local_y - 1.0, local_z - 1.0);
        let n111 = self.gradient(
            p_x1y1 as i32 + z + 1,
            local_x - 1.0,
            local_y - 1.0,
            local_z - 1.0,
        );

        let smooth_x = smoothstep(local_x);
        let smooth_y = smoothstep(interpolation_y);
        let smooth_z = smoothstep(local_z);

        lerp3(
            smooth_x, smooth_y, smooth_z, n000, n100, n010, n110, n001, n101, n011, n111,
        )
    }

    fn gradient(&self, hash: i32, x: f64, y: f64, z: f64) -> f64 {
        let gradient = GRADIENTS[(hash & 15) as usize];

        gradient[0] as f64 * x + gradient[1] as f64 * y + gradient[2] as f64 * z
    }
}

fn floor(value: f64) -> i32 {
    value.floor() as i32
}

fn smoothstep(value: f64) -> f64 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

fn lerp3(
    x: f64,
    y: f64,
    z: f64,
    n000: f64,
    n100: f64,
    n010: f64,
    n110: f64,
    n001: f64,
    n101: f64,
    n011: f64,
    n111: f64,
) -> f64 {
    let x00 = lerp(x, n000, n100);
    let x10 = lerp(x, n010, n110);
    let x01 = lerp(x, n001, n101);
    let x11 = lerp(x, n011, n111);

    let y0 = lerp(y, x00, x10);
    let y1 = lerp(y, x01, x11);

    lerp(z, y0, y1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_gen::random::LegacyRandomSource;

    #[test]
    fn noise_is_deterministic() {
        let mut random_a = LegacyRandomSource::new(12345);
        let mut random_b = LegacyRandomSource::new(12345);

        let noise_a = ImprovedNoise::new(&mut random_a);
        let noise_b = ImprovedNoise::new(&mut random_b);

        let a = noise_a.noise(12.5, 64.0, -32.25);
        let b = noise_b.noise(12.5, 64.0, -32.25);

        assert_eq!(a, b);
    }

    #[test]
    fn noise_changes_with_coordinates() {
        let mut random = LegacyRandomSource::new(12345);
        let noise = ImprovedNoise::new(&mut random);

        let a = noise.noise(0.0, 0.0, 0.0);
        let b = noise.noise(10.0, 20.0, 30.0);

        assert_ne!(a, b);
    }

    #[test]
    fn noise_output_is_finite() {
        let mut random = LegacyRandomSource::new(98765);
        let noise = ImprovedNoise::new(&mut random);

        for x in -16..16 {
            for y in -16..16 {
                for z in -16..16 {
                    let value = noise.noise(x as f64 * 0.25, y as f64 * 0.25, z as f64 * 0.25);

                    assert!(value.is_finite());
                }
            }
        }
    }
}
