pub mod buffer;
pub mod evaluator;
pub mod function;
pub mod graph;
pub mod volume;

pub use buffer::*;
pub use evaluator::*;
pub use function::*;
pub use graph::*;
pub use volume::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_addition() {
        let nodes = [
            DensityNode::constant(10.0),
            DensityNode::constant(20.0),
            DensityNode {
                op: DensityOp::Add as u32,
                a: 0,
                b: 1,
                p0: 0.0,
                p1: 0.0,
                p2: 0.0,
                p3: 0.0,
            },
        ];

        let mut values = Vec::new();

        let result = evaluator::evaluate(&nodes, 2, 0.0, 0.0, 0.0, &mut values).unwrap();

        assert_eq!(result, 30.0);
    }

    #[test]
    fn samples_volume_in_z_x_y_order() {
        let nodes = [DensityNode {
            op: DensityOp::YClampedGradient as u32,
            a: 0,
            b: 0,
            p0: 0.0,
            p1: 2.0,
            p2: 0.0,
            p3: 2.0,
        }];

        let mut output = [0.0; 8];

        evaluator::sample_volume(&nodes, 0, 0, 0, 0, 2, 2, 2, 1, 1, 1, &mut output).unwrap();

        assert_eq!(output, [0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,]);
    }
}
