use super::{DensityBuffer, DensityVolume};

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    Abs,
    Square,
    Cube,
    Sqrt,
    HalfNegative,
    QuarterNegative,
    Reciprocal,
    Negate,
    Squeeze,
}

#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
}

#[derive(Clone, Debug)]
pub enum DensityFunction {
    Constant(f32),

    Unary {
        op: UnaryOp,
        input: Box<Self>,
    },

    Binary {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
    },

    Clamp {
        input: Box<Self>,
        min: f32,
        max: f32,
    },

    Lerp {
        alpha: Box<Self>,
        first: Box<Self>,
        second: Box<Self>,
    },
}

impl DensityFunction {
    pub fn sample(&self, x: i32, y: i32, z: i32) -> f32 {
        match self {
            Self::Constant(value) => *value,

            Self::Unary { op, input } => {
                let value = input.sample(x, y, z);

                match op {
                    UnaryOp::Abs => value.abs(),
                    UnaryOp::Square => value * value,
                    UnaryOp::Cube => value * value * value,
                    UnaryOp::Sqrt => value.max(0.0).sqrt(),
                    UnaryOp::HalfNegative => {
                        if value < 0.0 {
                            value * 0.5
                        } else {
                            value
                        }
                    }
                    UnaryOp::QuarterNegative => {
                        if value < 0.0 {
                            value * 0.25
                        } else {
                            value
                        }
                    }
                    UnaryOp::Reciprocal => 1.0 / value,
                    UnaryOp::Negate => -value,
                    UnaryOp::Squeeze => {
                        let value = value.clamp(-1.0, 1.0);
                        value / 2.0 - value.powi(3) / 24.0
                    }
                }
            }

            Self::Binary { op, left, right } => {
                let a = left.sample(x, y, z);
                let b = right.sample(x, y, z);

                match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Sub => a - b,
                    BinaryOp::Mul => a * b,
                    BinaryOp::Div => a / b,
                    BinaryOp::Min => a.min(b),
                    BinaryOp::Max => a.max(b),
                }
            }

            Self::Clamp { input, min, max } => input.sample(x, y, z).clamp(*min, *max),

            Self::Lerp {
                alpha,
                first,
                second,
            } => {
                let alpha = alpha.sample(x, y, z);
                let first = first.sample(x, y, z);
                let second = second.sample(x, y, z);

                first + alpha * (second - first)
            }
        }
    }

    pub fn sample_volume(&self, volume: &DensityVolume, output: &mut DensityBuffer) {
        let required = volume.len();

        if output.values.len() != required {
            output.resize(required);
        }

        for z in 0..volume.size_z {
            let block_z = volume.block_z(z);

            for x in 0..volume.size_x {
                let block_x = volume.block_x(x);
                let base_index = volume.index(x, 0, z);

                for y in 0..volume.size_y {
                    let block_y = volume.block_y(y);

                    output.set(base_index + y, self.sample(block_x, block_y, block_z));
                }
            }
        }
    }
}
