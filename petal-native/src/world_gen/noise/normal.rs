use crate::world_gen::noise::{NoiseStack, PerlinNoise};
use std::sync::Arc;

const MAX_OCTAVE_COUNT: usize = 32;

const TARGET_DEVIATION: f64 = 1.0 / 3.0;
const LAYER_DEVIATION: f64 = 0.270_224_783_124_521_1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Normalization {
    Enabled,
    Disabled,
    Legacy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OctaveInfo {
    pub index: i32,
    pub frequency: f64,
    pub amplitude: f64,
}

impl OctaveInfo {
    pub fn seed(&self) -> String {
        format!("octave_{}", self.index)
    }
}

#[derive(Clone, Debug)]
pub struct NormalNoise {
    base_octave: i32,
    base_amplitude: f64,
    octave_count: usize,
    normalization: Normalization,
    modifiers: Vec<f64>,
    octaves: Vec<OctaveInfo>,
    normalization_factor: f64,
    range: [f32; 2],
}

impl NormalNoise {
    pub fn builder() -> NormalNoiseBuilder {
        NormalNoiseBuilder::default()
    }

    pub fn octaves(&self) -> &[OctaveInfo] {
        &self.octaves
    }

    pub fn normalization_factor(&self) -> f64 {
        self.normalization_factor
    }

    pub fn range(&self) -> [f32; 2] {
        self.range
    }

    pub fn base_octave(&self) -> i32 {
        self.base_octave
    }

    pub fn base_amplitude(&self) -> f64 {
        self.base_amplitude
    }

    pub fn octave_count(&self) -> usize {
        self.octave_count
    }

    pub fn normalization(&self) -> Normalization {
        self.normalization
    }

    pub fn create_with(
        &self,
        mut create_perlin: impl FnMut(usize, &str) -> PerlinNoise,
    ) -> NoiseStack {
        const SECOND_FREQUENCY: f64 = 1.018_126_888_217_522_7;

        let mut builder = NoiseStack::builder();

        for octave in &self.octaves {
            let name = octave.seed();
            let amplitude = (self.normalization_factor * octave.amplitude) as f32;

            let first = create_perlin(0, &name);
            let second = create_perlin(1, &name);

            builder.add(Arc::new(first), octave.frequency, amplitude);

            builder.add(
                Arc::new(second),
                octave.frequency * SECOND_FREQUENCY,
                amplitude,
            );
        }

        builder.build()
    }
}

#[derive(Clone, Debug)]
pub struct NormalNoiseBuilder {
    base_octave: i32,
    base_amplitude: f64,
    octave_count: usize,
    normalization: Normalization,
    modifiers: Vec<f64>,
}

impl Default for NormalNoiseBuilder {
    fn default() -> Self {
        Self {
            base_octave: 0,
            base_amplitude: 1.0,
            octave_count: 1,
            normalization: Normalization::Enabled,
            modifiers: Vec::new(),
        }
    }
}

impl NormalNoiseBuilder {
    pub fn set_base_octave(&mut self, octave: i32) -> &mut Self {
        self.base_octave = octave;
        self
    }

    pub fn set_base_amplitude(&mut self, amplitude: f64) -> &mut Self {
        self.base_amplitude = amplitude;
        self
    }

    pub fn set_octave_count(&mut self, count: usize) -> &mut Self {
        self.octave_count = count;
        self
    }

    pub fn set_normalization(&mut self, normalization: Normalization) -> &mut Self {
        self.normalization = normalization;
        self
    }

    pub fn set_amplitude_modifier(
        &mut self,
        index: usize,
        value: f64,
    ) -> Result<&mut Self, &'static str> {
        if index >= self.octave_count {
            return Err("amplitude modifier index is outside octave range");
        }

        if self.modifiers.is_empty() {
            self.modifiers.resize(self.octave_count, 1.0);
        }

        self.modifiers[index] = value;
        Ok(self)
    }

    pub fn build(&self) -> Result<NormalNoise, &'static str> {
        if self.octave_count == 0 {
            return Err("octave count must be at least 1");
        }

        if self.octave_count > MAX_OCTAVE_COUNT {
            return Err("octave count exceeds maximum of 32");
        }

        if !self.base_amplitude.is_finite() || self.base_amplitude < 0.0 {
            return Err("base amplitude must be finite and non-negative");
        }

        if !self.modifiers.is_empty() && self.modifiers.len() != self.octave_count {
            return Err("amplitude modifier count does not match octave count");
        }

        if self.modifiers.iter().any(|value| !value.is_finite()) {
            return Err("amplitude modifiers must be finite");
        }

        let octaves = build_octaves(
            self.base_octave,
            self.base_amplitude,
            self.octave_count,
            self.normalization != Normalization::Disabled,
            &self.modifiers,
        );

        let target_amplitude: f64 = octaves.iter().map(|octave| octave.amplitude.abs()).sum();

        let mut normalization_factor = compute_normalization_factor(target_amplitude, &octaves);

        let mut normalized_amplitude = target_amplitude;

        if self.normalization == Normalization::Legacy && normalization_factor != 0.0 {
            let legacy_factor = compute_legacy_normalization_factor(
                self.base_amplitude,
                self.octave_count,
                &self.modifiers,
            );

            normalized_amplitude *= legacy_factor / normalization_factor;
            normalization_factor = legacy_factor;
        }

        let bound = (normalized_amplitude * TARGET_DEVIATION * 6.0) as f32;

        Ok(NormalNoise {
            base_octave: self.base_octave,
            base_amplitude: self.base_amplitude,
            octave_count: self.octave_count,
            normalization: self.normalization,
            modifiers: self.modifiers.clone(),
            octaves,
            normalization_factor,
            range: [-bound, bound],
        })
    }
}

fn build_octaves(
    base_octave: i32,
    base_amplitude: f64,
    octave_count: usize,
    normalize: bool,
    modifiers: &[f64],
) -> Vec<OctaveInfo> {
    let mut frequency = 2.0_f64.powi(base_octave);
    let mut amplitude = base_amplitude;

    if normalize {
        amplitude *=
            2.0_f64.powi((octave_count - 1) as i32) / (2.0_f64.powi(octave_count as i32) - 1.0);
    }

    let mut octaves = Vec::with_capacity(octave_count);

    for index in 0..octave_count {
        let modifier = if modifiers.is_empty() {
            1.0
        } else {
            modifiers[index]
        };

        if modifier != 0.0 {
            octaves.push(OctaveInfo {
                index: base_octave.wrapping_add(index as i32),
                frequency,
                amplitude: amplitude * modifier,
            });
        }

        frequency *= 2.0;
        amplitude *= 0.5;
    }

    octaves
}

fn compute_normalization_factor(target_amplitude: f64, octaves: &[OctaveInfo]) -> f64 {
    let mut variance = 0.0;

    for octave in octaves {
        let deviation = LAYER_DEVIATION * octave.amplitude.abs();
        variance += deviation * deviation;
    }

    let input_deviation = variance.sqrt();

    if input_deviation == 0.0 {
        return 0.0;
    }

    let input_sum_deviation = input_deviation * 2.0_f64.sqrt();
    let target_deviation = target_amplitude * TARGET_DEVIATION;

    target_deviation / input_sum_deviation
}

fn compute_legacy_normalization_factor(
    base_amplitude: f64,
    octave_count: usize,
    modifiers: &[f64],
) -> f64 {
    let mut min_octave = usize::MAX;
    let mut max_octave = 0;

    for index in 0..octave_count {
        let modifier = if modifiers.is_empty() {
            1.0
        } else {
            modifiers[index]
        };

        if modifier == 0.0 {
            continue;
        }

        min_octave = min_octave.min(index);
        max_octave = max_octave.max(index);
    }

    if min_octave == usize::MAX {
        return 0.0;
    }

    let octave_span = max_octave - min_octave;

    let expected_deviation = 0.1 * (1.0 + 1.0 / (octave_span as f64 + 1.0));

    base_amplitude * 0.5 * TARGET_DEVIATION / expected_deviation
}
