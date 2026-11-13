use std::slice;
use crate::worldgen::density::{sample_volume, DensityNode};

const SUCCESS: i32 = 0;
const NULL_POINTER: i32 = 1;
const INVALID_ARGUMENT: i32 = 2;
const EVALUATION_FAILED: i32 = 3;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn petal_density_sample(
    nodes: *const DensityNode,
    node_count: usize,
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

    output: *mut f32,
    output_len: usize,
) -> i32 {
    if nodes.is_null() || output.is_null() {
        return NULL_POINTER;
    }

    if node_count == 0 || root >= node_count {
        return INVALID_ARGUMENT;
    }

    let Some(volume_size) = size_x
        .checked_mul(size_y)
        .and_then(|v| v.checked_mul(size_z))
    else {
        return INVALID_ARGUMENT;
    };

    if output_len < volume_size {
        return INVALID_ARGUMENT;
    }

    let nodes = slice::from_raw_parts(nodes, node_count);
    let output = slice::from_raw_parts_mut(output, output_len);

    match sample_volume(
        nodes,
        root,
        origin_x,
        origin_y,
        origin_z,
        size_x,
        size_y,
        size_z,
        step_x,
        step_y,
        step_z,
        output,
    ) {
        Ok(()) => SUCCESS,
        Err(_) => EVALUATION_FAILED,
    }
}