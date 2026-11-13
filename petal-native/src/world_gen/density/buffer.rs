#[derive(Clone, Debug)]
pub struct DensityBuffer {
    pub values: Vec<f32>,
}

impl DensityBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            values: vec![0.0; size],
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> f32 {
        self.values[index]
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: f32) {
        self.values[index] = value;
    }

    pub fn fill(&mut self, value: f32) {
        self.values.fill(value);
    }

    pub fn resize(&mut self, size: usize) {
        self.values.resize(size, 0.0);
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.values
    }
}
