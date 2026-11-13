use super::perlin::{PerlinNoise, SmearedPerlinNoise};
use crate::world_gen::noise::SimplexNoise;
use std::sync::Arc;

pub trait Noise: Send + Sync {
    fn get2(&self, x: f64, z: f64) -> f32;

    fn get3(&self, x: f64, y: f64, z: f64) -> f32;
}

impl Noise for PerlinNoise {
    fn get2(&self, x: f64, z: f64) -> f32 {
        PerlinNoise::get2(self, x, z)
    }

    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        PerlinNoise::get3(self, x, y, z)
    }
}

impl Noise for SmearedPerlinNoise {
    fn get2(&self, x: f64, z: f64) -> f32 {
        SmearedPerlinNoise::get2(self, x, z)
    }

    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        SmearedPerlinNoise::get3(self, x, y, z)
    }
}

#[derive(Clone)]
pub struct NoiseLayer {
    pub noise: Arc<dyn Noise>,
    pub frequency: f64,
    pub amplitude: f32,
}

#[derive(Clone, Default)]
pub struct NoiseStack {
    layers: Vec<NoiseLayer>,
}

impl NoiseStack {
    pub fn builder() -> NoiseStackBuilder {
        NoiseStackBuilder::default()
    }

    pub fn layers(&self) -> &[NoiseLayer] {
        &self.layers
    }

    pub fn get2(&self, x: f64, z: f64) -> f32 {
        let mut value = 0.0_f32;

        for layer in &self.layers {
            value += layer.amplitude * layer.noise.get2(x * layer.frequency, z * layer.frequency);
        }

        value
    }

    pub fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        let mut value = 0.0_f32;

        for layer in &self.layers {
            let frequency = layer.frequency;

            value += layer.amplitude
                * layer
                    .noise
                    .get3(x * frequency, y * frequency, z * frequency);
        }

        value
    }
}

impl Noise for SimplexNoise {
    fn get2(&self, x: f64, y: f64) -> f32 {
        SimplexNoise::get2(self, x, y)
    }

    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        SimplexNoise::get3(self, x, y, z)
    }
}

#[derive(Default)]
pub struct NoiseStackBuilder {
    layers: Vec<NoiseLayer>,
}

impl NoiseStackBuilder {
    pub fn add(&mut self, noise: Arc<dyn Noise>, frequency: f64, amplitude: f32) -> &mut Self {
        self.layers.push(NoiseLayer {
            noise,
            frequency,
            amplitude,
        });

        self
    }

    pub fn add_stack(&mut self, stack: &NoiseStack, frequency: f64, amplitude: f32) -> &mut Self {
        self.layers
            .extend(stack.layers.iter().map(|layer| NoiseLayer {
                noise: Arc::clone(&layer.noise),
                frequency: layer.frequency * frequency,
                amplitude: layer.amplitude * amplitude,
            }));

        self
    }

    pub fn build(&self) -> NoiseStack {
        NoiseStack {
            layers: self.layers.clone(),
        }
    }
}
