use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BigStateVector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // pub w: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BigStatePose {
    pub position: BigStateVector,
    pub orientation: BigStateVector,
    pub linear_velocity: BigStateVector,
    pub angular_velocity: BigStateVector,
    pub linear_acceleration: BigStateVector,
    pub angular_acceleration: BigStateVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigState {
    pub pose: BigStatePose,
    pub trajectory: HashMap<String, BigStatePose>,
}

impl Default for BigState {
    fn default() -> Self {
        Self::new()
    }
}

impl BigState {
    pub fn new() -> BigState {
        let mut trajectory = HashMap::new();
        for i in 0..10 {
            trajectory.insert(i.to_string(), BigStatePose::default());
        }
        BigState {
            pose: BigStatePose::default(),
            trajectory,
        }
    }
}
