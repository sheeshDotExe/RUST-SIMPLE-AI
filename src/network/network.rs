use core::fmt::Debug;
use rand::{rngs::ThreadRng, Rng};

pub const E: f64 = 2.71828182845904523536028747135266250f64;

pub const WEIGHT_MUTATION_RATE: f64 = 0.2;
pub const BIAS_MUTATION_RATE: f64 = 0.2;

pub const MIN_WEIGHT_MUTATION: f64 = -0.5;
pub const MAX_WEIGHT_MUTATION: f64 = 0.5;

pub const MIN_BIAS_MUTATION: f64 = -10.0;
pub const MAX_BIAS_MUTATION: f64 = 10.0;

fn sigmoid(x: f64) -> f64 {
    return 1.0 / (1.0 + E.powf(x));
}

#[derive(Debug, Clone)]
pub struct Layer {
    nodes: Vec<f64>,
    weights: Vec<Vec<f64>>,
}

#[derive(Debug, Clone)]
pub struct Network {
    layers: Vec<Layer>,
}

impl Layer {
    fn new(nodes: usize, weights: usize, rng: &mut ThreadRng) -> Self {
        let mut layer = Self {
            nodes: Vec::with_capacity(nodes),
            weights: Vec::with_capacity(nodes),
        };

        for _bias_index in 0..nodes {
            let value = rng.gen_range(-20.0..=20.0);
            layer.nodes.push(value);
        }

        for node_index in 0..nodes {
            layer.weights.push(Vec::with_capacity(weights));
            for _weight_index in 0..weights {
                let value = rng.gen_range(-2.0..=2.0);
                layer.weights[node_index].push(value);
            }
        }

        return layer;
    }

    fn apply_layer(
        input_values: Vec<f64>,
        prev_layer: &Layer,
        output_values: Vec<f64>,
    ) -> Vec<f64> {
        let mut output_values = output_values;
        for node_index in 0..input_values.len() {
            for (i, weight) in prev_layer.weights[node_index].iter().enumerate() {
                output_values[i] = sigmoid(output_values[i] + input_values[node_index] * weight);
            }
        }
        return output_values;
    }
}

impl Network {
    pub fn new(dimensions: Vec<usize>, rng: &mut ThreadRng) -> Self {
        let mut network = Self {
            layers: Vec::with_capacity(dimensions.len()),
        };

        assert!(dimensions.len() >= 2, "Invalid network size");

        for i in 0..dimensions.len() {
            if i == dimensions.len() - 1 {
                // output layer
                let layer = Layer::new(dimensions[i], 0, rng);
                network.layers.push(layer);
                break;
            }
            // input layer or hidden layer
            let layer = Layer::new(dimensions[i], dimensions[i + 1], rng);
            network.layers.push(layer);
        }

        return network;
    }

    pub fn mutate(&mut self, rng: &mut ThreadRng) {
        for layer in self.layers.iter_mut() {
            for node in layer.nodes.iter_mut() {
                if rng.gen_range(0.0..=1.0) < BIAS_MUTATION_RATE {
                    *node += rng.gen_range(MIN_BIAS_MUTATION..=MAX_BIAS_MUTATION);
                }
            }

            for weights in layer.weights.iter_mut() {
                for weight in weights.iter_mut() {
                    if rng.gen_range(0.0..=1.0) < WEIGHT_MUTATION_RATE {
                        *weight += rng.gen_range(MIN_WEIGHT_MUTATION..=MAX_WEIGHT_MUTATION);
                    }
                }
            }
        }
    }

    pub fn inherit_from(parent: &Network, rng: &mut ThreadRng) -> Self {
        let mut new_network = parent.clone();
        new_network.mutate(rng);
        return new_network;
    }

    pub fn feed_forward(&self, input_values: Vec<f64>) -> Vec<f64> {
        let mut input_values = input_values;
        for index in 0..self.layers.len() {
            if index < 1 {
                continue;
            }
            let prev_layer = &self.layers[index - 1];
            let layer = &self.layers[index];
            input_values = Layer::apply_layer(input_values, prev_layer, layer.nodes.clone());
        }

        return input_values;
    }
}
