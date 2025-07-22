//! Echo State Network (ESN) reservoir implementation (recurrent neural network with fixed weights).

use rand::Rng;
use num_traits::sign::Signed;

pub struct EchoStateNetwork {
    pub state_size: usize,
    pub input_size: usize,
    pub output_size: usize,
    pub state: Vec<f32>,
    pub w: Vec<Vec<f32>>,      // recurrent weight matrix (state_size x state_size)
    pub w_in: Vec<Vec<f32>>,   // input weight matrix (state_size x input_size)
    pub w_out: Vec<Vec<f32>>,  // output weight matrix (output_size x state_size)
}

impl EchoStateNetwork {
    /// Creates a new Echo State Network with random internal weights.
    /// `spectral_radius` should be less than 1.0 for echo state property.
    pub fn new(state_size: usize, input_size: usize, output_size: usize, spectral_radius: f32) -> Self {
        let mut rng = rand::rng();
        // Initialize random weights
        let mut w = vec![vec![0.0; state_size]; state_size];
        for i in 0..state_size {
            for j in 0..state_size {
                w[i][j] = rng.random_range(-1.0..1.0);
            }
        }
        // Scale W to have spectral radius approx given value (simple scaling of magnitude)
        // (In practice, would compute eigenvalues, but here just normalize by max row sum for simplicity)
        let max_row_sum = w.iter().map(|row| row.iter().map(|v| v.abs()).sum::<f32>()).fold(0.0, f32::max);
        if max_row_sum > 0.0 {
            let scale = spectral_radius / max_row_sum;
            for i in 0..state_size {
                for j in 0..state_size {
                    w[i][j] *= scale;
                }
            }
        }
        let mut w_in = vec![vec![0.0; input_size]; state_size];
        for i in 0..state_size {
            for j in 0..input_size {
                w_in[i][j] = rng.random_range(-1.0..1.0) * 0.5;
            }
        }
        let w_out = vec![vec![0.0; state_size]; output_size];
        EchoStateNetwork {
            state_size,
            input_size,
            output_size,
            state: vec![0.0; state_size],
            w,
            w_in,
            w_out,
        }
    }

    /// Advances the reservoir one time-step with the given input vector.
    /// Returns the network's output (current readout).
    pub fn update(&mut self, input: &[f32]) -> Vec<f32> {
        // Calculate new state = tanh(W_in * input + W * prev_state)
        let mut new_state = vec![0.0; self.state_size];
        for i in 0..self.state_size {
            let mut sum = 0.0;
            // input contribution
            for j in 0..self.input_size {
                if j < input.len() {
                    sum += self.w_in[i][j] * input[j];
                }
            }
            // recurrent contribution
            for j in 0..self.state_size {
                sum += self.w[i][j] * self.state[j];
            }
            new_state[i] = sum.tanh();
        }
        self.state = new_state;
        // Compute output = W_out * state (linear readout)
        let mut output = vec![0.0; self.output_size];
        for k in 0..self.output_size {
            let mut sum = 0.0;
            for j in 0..self.state_size {
                sum += self.w_out[k][j] * self.state[j];
            }
            output[k] = sum;
        }
        output
    }

    /// Sets the output weights (e.g., after training).
    pub fn set_output_weights(&mut self, weights: Vec<Vec<f32>>) {
        self.w_out = weights;
    }
}
