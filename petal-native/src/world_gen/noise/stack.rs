use super::perlin::{PerlinNoise};
use crate::world_gen::density::DensityVolume;
use crate::world_gen::noise::SimplexNoise;
use crate::world_gen::noise::smeared::SmearedPerlinNoise;
use std::sync::Arc;

pub trait Noise: Send + Sync {
    fn get2(&self, x: f64, z: f64) -> f32;

    fn get3(&self, x: f64, y: f64, z: f64) -> f32;

    fn range(&self) -> [f32; 2];

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        assert_eq!(buffer.len(), volume.len());

        let mut index = 0;

        for z in 0..volume.size_z {
            let z = f64::from(volume.block_z(z)) * xz_scale;

            for x in 0..volume.size_x {
                let x = f64::from(volume.block_x(x)) * xz_scale;

                for y in 0..volume.size_y {
                    let y = f64::from(volume.block_y(y)) * y_scale;

                    buffer[index] += amplitude * self.get3(x, y, z);
                    index += 1;
                }
            }
        }
    }
}

impl Noise for PerlinNoise {
    fn get2(&self, x: f64, z: f64) -> f32 {
        PerlinNoise::get2(self, x, z)
    }

    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        PerlinNoise::get3(self, x, y, z)
    }

    fn range(&self) -> [f32; 2] {
        [-2.0, 2.0]
    }

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        PerlinNoise::add_to_volume(
            self,
            buffer,
            volume.size_x,
            volume.size_y,
            volume.size_z,
            volume.min_x,
            volume.min_y,
            volume.min_z,
            xz_scale,
            y_scale,
            amplitude,
        );
    }
}

impl Noise for SmearedPerlinNoise {
    fn get2(&self, x: f64, z: f64) -> f32 {
        SmearedPerlinNoise::get2(self, x, z)
    }

    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        SmearedPerlinNoise::get3(self, x, y, z)
    }

    fn range(&self) -> [f32; 2] {
        let range = self.fudge_y_scale.abs() as f32 + 2.0;
        [-range, range]
    }

    fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        SmearedPerlinNoise::add_to_volume(
            self,
            buffer,
            volume,
            xz_scale,
            y_scale,
            amplitude,
        );
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

    pub fn range(&self) -> [f32; 2] {
        self.layers.iter().fold([0.0, 0.0], |range, layer| {
            let noise_range = layer.noise.range();

            let min = noise_range[0] * layer.amplitude;
            let max = noise_range[1] * layer.amplitude;

            [range[0] + min.min(max), range[1] + min.max(max)]
        })
    }

    pub fn add_to_volume(
        &self,
        buffer: &mut [f32],
        volume: &DensityVolume,
        xz_scale: f64,
        y_scale: f64,
        amplitude: f32,
    ) {
        assert_eq!(buffer.len(), volume.len());

        for layer in &self.layers {
            layer.noise.add_to_volume(
                buffer,
                volume,
                xz_scale * layer.frequency,
                y_scale * layer.frequency,
                amplitude * layer.amplitude,
            );
        }
    }
}

impl Noise for SimplexNoise {
    fn get2(&self, x: f64, y: f64) -> f32 {
        SimplexNoise::get2(self, x, y)
    }

    fn get3(&self, x: f64, y: f64, z: f64) -> f32 {
        SimplexNoise::get3(self, x, y, z)
    }

    fn range(&self) -> [f32; 2] {
        [-2.0, 2.0]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct ConstantNoise(f32);

    impl Noise for ConstantNoise {
        fn get2(&self, _: f64, _: f64) -> f32 {
            self.0
        }

        fn get3(&self, _: f64, _: f64, _: f64) -> f32 {
            self.0
        }

        fn range(&self) -> [f32; 2] {
            [self.0, self.0]
        }
    }

    #[test]
    fn sums_layers() {
        let stack = NoiseStack::builder()
            .add(Arc::new(ConstantNoise(2.0)), 1.0, 3.0)
            .add(Arc::new(ConstantNoise(4.0)), 2.0, 0.5)
            .build();

        assert_eq!(stack.get2(1.0, 1.0), 8.0);
        assert_eq!(stack.get3(1.0, 1.0, 1.0), 8.0);
    }

    #[test]
    fn calculates_range() {
        let stack = NoiseStack::builder()
            .add(Arc::new(ConstantNoise(2.0)), 1.0, 3.0)
            .add(Arc::new(ConstantNoise(4.0)), 1.0, -0.5)
            .build();

        assert_eq!(stack.range(), [4.0, 4.0]);
    }

    #[test]
    fn flattens_nested_stack() {
        let inner = NoiseStack::builder()
            .add(Arc::new(ConstantNoise(2.0)), 2.0, 3.0)
            .build();

        let outer = NoiseStack::builder().add_stack(&inner, 4.0, 0.5).build();

        assert_eq!(outer.get2(1.0, 1.0), 3.0);
    }
}
