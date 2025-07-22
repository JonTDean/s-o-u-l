//! Liquid State Machine (spiking neural network reservoir) stub.
use rand::Rng;

/// A simple spiking network with integrate-and-fire neurons.
pub struct LiquidStateMachine {
    pub neurons: usize,
    pub connections: Vec<(usize, usize, f32)>, // (source, target, weight)
    pub potential: Vec<f32>,   // membrane potentials
}

impl LiquidStateMachine {
    pub fn new(neurons: usize, connection_prob: f32) -> Self {
        let mut rng = rand::rng();
        let mut connections = Vec::new();
        for i in 0..neurons {
            for j in 0..neurons {
                if i != j && rng.random::<f32>() < connection_prob {
                    // random weight between 0 and 1
                    connections.push((i, j, rng.random::<f32>()));
                }
            }
        }
        LiquidStateMachine {
            neurons,
            connections,
            potential: vec![0.0; neurons],
        }
    }

    /// Steps the network given external inputs (which neurons receive a spike input).
    /// Returns a vector of booleans indicating which neurons spiked this step.
    pub fn step(&mut self, inputs: &[bool]) -> Vec<bool> {
        let threshold = 1.0;
        let mut spiked = vec![false; self.neurons];
        // Apply external inputs as instant potential boosts
        for (i, &stim) in inputs.iter().enumerate() {
            if stim && i < self.neurons {
                self.potential[i] += 1.0;
            }
        }
        // Determine which neurons fire
        for i in 0..self.neurons {
            if self.potential[i] > threshold {
                spiked[i] = true;
                self.potential[i] = 0.0; // reset after spike
            }
        }
        // Propagate spikes to connected neurons
        for &(src, tgt, weight) in &self.connections {
            if src < self.neurons && tgt < self.neurons && spiked[src] {
                self.potential[tgt] += weight;
            }
        }
        // Leak potential (decay for those not spiked this step)
        for i in 0..self.neurons {
            if !spiked[i] {
                self.potential[i] *= 0.9;
            }
        }
        spiked
    }
}
