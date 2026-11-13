use crate::world_gen::random::RandomSource;

use super::PerlinNoise;

const INPUT_FACTOR: f64 = 1.0181268882175227;

pub struct NormalNoise {
    first: PerlinNoise,
    second: PerlinNoise,
    value_factor: f64,
    max_value: f64,
}

impl NormalNoise {
    pub fn new_legacy(
        random: &mut impl RandomSource,
        first_octave: i32,
        amplitudes: Vec<f64>,
    ) -> Self {
        let first = PerlinNoise::new_legacy(random, first_octave, amplitudes.clone());

        let second = PerlinNoise::new_legacy(random, first_octave, amplitudes.clone());

        Self::from_perlin(first, second, &amplitudes)
    }

    pub fn from_perlin(first: PerlinNoise, second: PerlinNoise, amplitudes: &[f64]) -> Self {
        let active_octaves: Vec<usize> = amplitudes
            .iter()
            .enumerate()
            .filter_map(|(index, amplitude)| (*amplitude != 0.0).then_some(index))
            .collect();

        let octave_span = match (active_octaves.first(), active_octaves.last()) {
            (Some(min), Some(max)) => max - min,
            _ => 0,
        };

        let expected_deviation = 0.1 * (1.0 + 1.0 / (octave_span as f64 + 1.0));

        let value_factor = 1.0 / 6.0 / expected_deviation;

        let max_value = (first.max_value() + second.max_value()) * value_factor;

        Self {
            first,
            second,
            value_factor,
            max_value,
        }
    }

    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        let first = self.first.get_value(x, y, z);

        let second = self
            .second
            .get_value(x * INPUT_FACTOR, y * INPUT_FACTOR, z * INPUT_FACTOR);

        (first + second) * self.value_factor
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn first(&self) -> &PerlinNoise {
        &self.first
    }

    pub fn second(&self) -> &PerlinNoise {
        &self.second
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_gen::random::LegacyRandomSource;

    #[test]
    fn normal_noise_is_deterministic() {
        let mut random_a = LegacyRandomSource::new(1234);
        let mut random_b = LegacyRandomSource::new(1234);

        let noise_a = NormalNoise::new_legacy(&mut random_a, -2, vec![1.0, 0.5, 0.25]);

        let noise_b = NormalNoise::new_legacy(&mut random_b, -2, vec![1.0, 0.5, 0.25]);

        assert_eq!(
            noise_a.get_value(10.0, 20.0, 30.0),
            noise_b.get_value(10.0, 20.0, 30.0)
        );
    }

    #[test]
    fn normal_noise_is_finite() {
        let mut random = LegacyRandomSource::new(5678);

        let noise = NormalNoise::new_legacy(&mut random, -2, vec![1.0, 0.5, 0.25]);

        for x in -10..10 {
            let value = noise.get_value(x as f64, x as f64 * 2.0, x as f64 * 3.0);

            assert!(value.is_finite());
        }
    }
}
