#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DensityOp {
    Constant = 0,
    YClampedGradient = 1,

    Add = 2,
    Mul = 3,
    Min = 4,
    Max = 5,

    Abs = 6,
    Square = 7,
    Cube = 8,
    HalfNegative = 9,
    QuarterNegative = 10,
    Squeeze = 11,

    Clamp = 12,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DensityNode {
    pub op: u32,

    pub a: u32,
    pub b: u32,

    pub p0: f64,
    pub p1: f64,
    pub p2: f64,
    pub p3: f64,
}

impl DensityNode {
    pub fn constant(value: f64) -> Self {
        Self {
            op: DensityOp::Constant as u32,
            a: 0,
            b: 0,
            p0: value,
            p1: 0.0,
            p2: 0.0,
            p3: 0.0,
        }
    }
}