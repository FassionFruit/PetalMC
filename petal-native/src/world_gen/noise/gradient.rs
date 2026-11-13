use crate::world_gen::random::RandomSource;

pub const GRADIENTS: [[i8; 3]; 16] = [
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

#[derive(Clone, Debug)]
pub struct GradientTable {
    pub offset_x: f64,
    pub offset_y: f64,
    pub offset_z: f64,
    pub permutations: [u8; 256],
}

impl GradientTable {
    pub fn new(random: &mut impl RandomSource, noise_offset_scale: f64) -> Self {
        let offset_x = random.next_f64() * noise_offset_scale;
        let offset_y = random.next_f64() * noise_offset_scale;
        let offset_z = random.next_f64() * noise_offset_scale;

        let mut permutations = std::array::from_fn(|i| i as u8);

        for i in 0..256 {
            let j = (i + random.next_i32_bound(256 - i)) as usize;
            permutations.swap(i as usize, j);
        }

        Self {
            offset_x,
            offset_y,
            offset_z,
            permutations,
        }
    }

    #[inline]
    pub fn permute(&self, index: i32) -> i32 {
        self.permutations[(index & 255) as usize] as i32
    }

    #[inline]
    pub fn permute_to_gradient(&self, index: i32) -> [i8; 3] {
        GRADIENTS[(self.permute(index) & 15) as usize]
    }

    #[inline]
    pub fn offsets(&self) -> [f64; 3] {
        [self.offset_x, self.offset_y, self.offset_z]
    }
}

#[inline]
pub fn grad_dot(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    let gradient = GRADIENTS[(hash & 15) as usize];

    gradient[0] as f32 * x + gradient[1] as f32 * y + gradient[2] as f32 * z
}

#[inline]
pub fn wrap(value: f64) -> f64 {
    const ROUND_OFF: f64 = 33_554_432.0;
    const HALF_ROUND_OFF: f64 = f64::from_bits(16_777_216.0_f64.to_bits() - 1);

    if value >= -HALF_ROUND_OFF && value < HALF_ROUND_OFF {
        return value;
    }

    value - (value / ROUND_OFF + 0.5).floor() * ROUND_OFF
}
