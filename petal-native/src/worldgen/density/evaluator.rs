use super::{DensityNode, DensityOp};

#[derive(Debug)]
pub enum DensityError {
    EmptyGraph,
    InvalidRoot,
    InvalidNode,
    InvalidInput,
}

fn gradient(y: f64, node: &DensityNode) -> f64 {
    let from_y = node.p0;
    let to_y = node.p1;

    if from_y == to_y {
        return node.p2;
    }

    let t = ((y - from_y) / (to_y - from_y)).clamp(0.0, 1.0);
    node.p2 + (node.p3 - node.p2) * t
}

fn squeeze(value: f64) -> f64 {
    let clamped = value.clamp(-1.0, 1.0);
    clamped / 2.0 - clamped.powi(3) / 24.0
}

pub fn evaluate(
    nodes: &[DensityNode],
    root: usize,
    x: f64,
    y: f64,
    z: f64,
    values: &mut Vec<f64>,
) -> Result<f64, DensityError> {
    if nodes.is_empty() {
        return Err(DensityError::EmptyGraph);
    }

    if root >= nodes.len() {
        return Err(DensityError::InvalidRoot);
    }

    values.clear();
    values.reserve(nodes.len().saturating_sub(values.capacity()));

    for (index, node) in nodes.iter().enumerate() {
        let op = match node.op {
            0 => DensityOp::Constant,
            1 => DensityOp::YClampedGradient,
            2 => DensityOp::Add,
            3 => DensityOp::Mul,
            4 => DensityOp::Min,
            5 => DensityOp::Max,
            6 => DensityOp::Abs,
            7 => DensityOp::Square,
            8 => DensityOp::Cube,
            9 => DensityOp::HalfNegative,
            10 => DensityOp::QuarterNegative,
            11 => DensityOp::Squeeze,
            12 => DensityOp::Clamp,
            _ => return Err(DensityError::InvalidNode),
        };

        let input = |input_index: u32| -> Result<f64, DensityError> {
            let input_index = input_index as usize;

            if input_index >= index {
                return Err(DensityError::InvalidInput);
            }

            Ok(values[input_index])
        };

        let value = match op {
            DensityOp::Constant => node.p0,

            DensityOp::YClampedGradient => gradient(y, node),

            DensityOp::Add => input(node.a)? + input(node.b)?,
            DensityOp::Mul => input(node.a)? * input(node.b)?,
            DensityOp::Min => input(node.a)?.min(input(node.b)?),
            DensityOp::Max => input(node.a)?.max(input(node.b)?),

            DensityOp::Abs => input(node.a)?.abs(),
            DensityOp::Square => input(node.a)?.powi(2),
            DensityOp::Cube => input(node.a)?.powi(3),

            DensityOp::HalfNegative => {
                let value = input(node.a)?;
                if value < 0.0 { value * 0.5 } else { value }
            }

            DensityOp::QuarterNegative => {
                let value = input(node.a)?;
                if value < 0.0 { value * 0.25 } else { value }
            }

            DensityOp::Squeeze => squeeze(input(node.a)?),

            DensityOp::Clamp => {
                input(node.a)?.clamp(node.p0, node.p1)
            }
        };

        values.push(value);
    }

    let _ = (x, z);

    Ok(values[root])
}

pub fn sample_volume(
    nodes: &[DensityNode],
    root: usize,
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    size_x: usize,
    size_y: usize,
    size_z: usize,
    step_x: i32,
    step_y: i32,
    step_z: i32,
    output: &mut [f32],
) -> Result<(), DensityError> {
    let expected_size = size_x
        .checked_mul(size_y)
        .and_then(|v| v.checked_mul(size_z))
        .ok_or(DensityError::InvalidInput)?;

    if output.len() < expected_size {
        return Err(DensityError::InvalidInput);
    }

    let mut values = Vec::with_capacity(nodes.len());

    for z in 0..size_z {
        for x in 0..size_x {
            for y in 0..size_y {
                let world_x = origin_x + x as i32 * step_x;
                let world_y = origin_y + y as i32 * step_y;
                let world_z = origin_z + z as i32 * step_z;

                let index = (z * size_x + x) * size_y + y;

                output[index] = evaluate(
                    nodes,
                    root,
                    world_x as f64,
                    world_y as f64,
                    world_z as f64,
                    &mut values,
                )
                    .map_err(|_| DensityError::InvalidNode)? as f32;
            }
        }
    }

    Ok(())
}