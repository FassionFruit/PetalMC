use super::source::RandomSource;

const GOLDEN_RATIO_64: u64 = 0x9E3779B97F4A7C15;
const SILVER_RATIO_64: u64 = 0x6A09E667F3BCC909;

pub struct XoroshiroRandomSource {
    low: u64,
    high: u64,
}

impl XoroshiroRandomSource {
    pub fn new(seed: i64) -> Self {
        let low = seed as u64 ^ SILVER_RATIO_64;
        let high = low.wrapping_add(GOLDEN_RATIO_64);

        Self {
            low: mix_stafford13(low),
            high: mix_stafford13(high),
        }
    }

    pub fn from_state(low: u64, high: u64) -> Self {
        assert!(low != 0 || high != 0);

        Self { low, high }
    }

    fn next_long(&mut self) -> u64 {
        let s0 = self.low;
        let mut s1 = self.high;

        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);

        s1 ^= s0;

        self.low = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.high = s1.rotate_left(28);

        result
    }
}

impl RandomSource for XoroshiroRandomSource {
    fn next_u64(&mut self) -> u64 {
        self.next_long()
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_long() >> 32) as u32
    }
}

pub fn mix_stafford13(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);

    value ^ (value >> 31)
}
