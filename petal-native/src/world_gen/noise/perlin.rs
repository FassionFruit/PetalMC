use crate::world_gen::random::RandomSource;

use super::ImprovedNoise;

pub struct PerlinNoise {
    first_octave: i32,
    amplitudes: Vec<f64>,
    noise_levels: Vec<Option<ImprovedNoise>>,
    lowest_freq_input_factor: f64,
    lowest_freq_value_factor: f64,
    max_value: f64,
}

impl PerlinNoise {
    pub fn new_legacy(
        random: &mut impl RandomSource,
        first_octave: i32,
        amplitudes: Vec<f64>,
    ) -> Self {
        let count = amplitudes.len();

        let mut noise_levels: Vec<Option<ImprovedNoise>> =
            std::iter::repeat_with(|| None).take(count).collect();

        for index in (0..count).rev() {
            if amplitudes[index] != 0.0 {
                noise_levels[index] = Some(ImprovedNoise::new(random));
            } else {
                Self::skip_octave(random);
            }
        }

        Self::from_levels(first_octave, amplitudes, noise_levels)
    }

    pub fn from_levels(
        first_octave: i32,
        amplitudes: Vec<f64>,
        noise_levels: Vec<Option<ImprovedNoise>>,
    ) -> Self {
        assert_eq!(amplitudes.len(), noise_levels.len());

        let count = amplitudes.len();

        let lowest_freq_input_factor = 2.0_f64.powi(first_octave);
        let lowest_freq_value_factor =
            2.0_f64.powi(count as i32 - 1) / (2.0_f64.powi(count as i32) - 1.0);

        let max_value = amplitudes
            .iter()
            .map(|amplitude| amplitude.abs())
            .sum::<f64>()
            * lowest_freq_value_factor;

        Self {
            first_octave,
            amplitudes,
            noise_levels,
            lowest_freq_input_factor,
            lowest_freq_value_factor,
            max_value,
        }
    }

    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample(x, y, z, 0.0, 0.0, false)
    }

    pub fn sample(
        &self,
        x: f64,
        y: f64,
        z: f64,
        y_scale: f64,
        y_max: f64,
        use_origin: bool,
    ) -> f64 {
        let mut input_factor = self.lowest_freq_input_factor;
        let mut value_factor = self.lowest_freq_value_factor;
        let mut result = 0.0;

        for (index, noise) in self.noise_levels.iter().enumerate() {
            if let Some(noise) = noise {
                let sample_x = Self::wrap(x * input_factor);
                let sample_y = Self::wrap(y * input_factor);
                let sample_z = Self::wrap(z * input_factor);

                let value = if use_origin {
                    noise.noise_with_y_scale(
                        sample_x,
                        sample_y,
                        sample_z,
                        y_scale * input_factor,
                        y_max * input_factor,
                    )
                } else {
                    noise.noise(sample_x, sample_y, sample_z)
                };

                result += self.amplitudes[index] * value * value_factor;
            }

            input_factor *= 2.0;
            value_factor *= 0.5;
        }

        result
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn first_octave(&self) -> i32 {
        self.first_octave
    }

    pub fn amplitudes(&self) -> &[f64] {
        &self.amplitudes
    }

    pub fn get_octave_noise(&self, octave: i32) -> Option<&ImprovedNoise> {
        let index = octave - self.first_octave;

        if index < 0 {
            return None;
        }

        self.noise_levels.get(index as usize)?.as_ref()
    }

    pub fn wrap(value: f64) -> f64 {
        let wrapped = value - (value / 33_554_432.0 + 0.5).floor() * 33_554_432.0;
        wrapped
    }

    fn skip_octave(random: &mut impl RandomSource) {
        for _ in 0..262 {
            random.next_u32();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_gen::random::LegacyRandomSource;

    #[test]
    fn perlin_noise_is_deterministic() {
        let amplitudes = vec![1.0, 0.5, 0.25];

        let mut random_a = LegacyRandomSource::new(1234);
        let mut random_b = LegacyRandomSource::new(1234);

        let noise_a = PerlinNoise::new_legacy(&mut random_a, -2, amplitudes.clone());

        let noise_b = PerlinNoise::new_legacy(&mut random_b, -2, amplitudes);

        assert_eq!(
            noise_a.get_value(10.0, 20.0, 30.0),
            noise_b.get_value(10.0, 20.0, 30.0)
        );
    }

    #[test]
    fn perlin_noise_is_finite() {
        let mut random = LegacyRandomSource::new(5678);

        let noise = PerlinNoise::new_legacy(&mut random, -2, vec![1.0, 0.5, 0.25]);

        for x in -10..10 {
            let value = noise.get_value(x as f64, x as f64 * 2.0, x as f64 * 3.0);

            assert!(value.is_finite());
        }
    }
}
