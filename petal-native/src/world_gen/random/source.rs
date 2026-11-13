
pub trait RandomSource {
    fn next_u64(&mut self) -> u64;

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_i32(&mut self) -> i32 {
        self.next_u32() as i32
    }

    fn next_i32_bound(&mut self, bound: u32) -> u32 {
        assert!(bound > 0);

        let mut bits = self.next_u32() >> 1;
        let mut value = bits % bound;

        while bits.wrapping_sub(value).wrapping_add(bound - 1) >= (1 << 31) {
            bits = self.next_u32() >> 1;
            value = bits % bound;
        }

        value
    }

    fn next_f64(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u32() >> 8) as f32) * (1.0 / ((1u32 << 24) as f32))
    }
}

pub struct LegacyRandomSource {
    seed: u64,
}

impl LegacyRandomSource {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1 << 48) - 1;

    pub fn new(seed: i64) -> Self {
        Self {
            seed: ((seed as u64) ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next_bits(&mut self, bits: u32) -> u32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;

        (self.seed >> (48 - bits)) as u32
    }

    fn next_f64(&mut self) -> f64 {
        let high = (self.next_bits(26) as u64) << 27;
        let low = self.next_bits(27) as u64;

        (high + low) as f64 * (1.0 / ((1u64 << 53) as f64))
    }
}

impl RandomSource for LegacyRandomSource {
    fn next_u64(&mut self) -> u64 {
        let high = (self.next_bits(32) as u64) << 32;
        let low = self.next_bits(32) as i32 as i64 as u64;

        high.wrapping_add(low)
    }

    fn next_u32(&mut self) -> u32 {
        self.next_bits(32)
    }

    fn next_i32_bound(&mut self, bound: u32) -> u32 {
        assert!(bound > 0);

        if bound.is_power_of_two() {
            return ((bound as u64 * self.next_bits(31) as u64) >> 31) as u32;
        }

        loop {
            let bits = self.next_bits(31);
            let value = bits % bound;

            if bits.wrapping_sub(value).wrapping_add(bound - 1) < (1 << 31) {
                return value;
            }
        }
    }

    fn next_f64(&mut self) -> f64 {
        LegacyRandomSource::next_f64(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{LegacyRandomSource, RandomSource};

    #[test]
    fn legacy_next_f64_matches_java_random() {
        let mut random = LegacyRandomSource::new(0);

        let expected: [f64; 3] = [
            0.730967787376657,
            0.24053641567148587,
            0.6374174253501083,
        ];

        expected.into_iter().for_each(|expected| {
            assert_eq!(random.next_f64().to_bits(), expected.to_bits());
        });
    }
}
