use log::info;

use serde::{Deserialize, Serialize};
use victory_data_store::{database::Datastore, topics::TopicKey};
use victory_time_rs::Timepoint;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "uav_state")]
pub struct UAVState {
    pub position: Pose,
    pub velocity: Pose,
    pub acceleration: Pose,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "pose")]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

impl Default for Pose {
    fn default() -> Pose {
        Pose {
            x: 0.0,
            y: 1.0,
            z: 2.0,
            roll: 3.0,
            pitch: 4.0,
            yaw: 5.0,
        }
    }
}

impl Default for UAVState {
    fn default() -> UAVState {
        UAVState {
            position: Pose::default(),
            velocity: Pose::default(),
            acceleration: Pose::default(),
        }
    }
}

fn main() {
    env_logger::init();

    let uav_state = UAVState::default();
    let mut data_store = Datastore::new();

    let topic = TopicKey::from_str("main");

    for i in 0..1000 {
        let time = Timepoint::new_secs(i as f64);
        data_store
            .add_struct(&topic, time, uav_state.clone())
            .expect("Failed to add struct to data store");
    }

    let last_value: UAVState = data_store
        .get_struct(&topic)
        .expect("Failed to get latest struct from data store");

    info!("Last value: {:?}", last_value);
    info!("Saved Keys: {:#?}", data_store.get_all_display_names());

    /*
    [2024-10-14T08:50:38Z INFO  victory_data_store] Saved Keys: {
        "main/uav_state",
        "main/uav_state/acceleration/pose",
        "main/uav_state/acceleration/pose/pitch",
        "main/uav_state/acceleration/pose/roll",
        "main/uav_state/acceleration/pose/x",
        "main/uav_state/acceleration/pose/y",
        "main/uav_state/acceleration/pose/yaw",
        "main/uav_state/acceleration/pose/z",
        "main/uav_state/position/pose",
        "main/uav_state/position/pose/pitch",
        "main/uav_state/position/pose/roll",
        "main/uav_state/position/pose/x",
        "main/uav_state/position/pose/y",
        "main/uav_state/position/pose/yaw",
        "main/uav_state/position/pose/z",
        "main/uav_state/velocity/pose",
        "main/uav_state/velocity/pose/pitch",
        "main/uav_state/velocity/pose/roll",
        "main/uav_state/velocity/pose/x",
        "main/uav_state/velocity/pose/y",
        "main/uav_state/velocity/pose/yaw",
        "main/uav_state/velocity/pose/z",
    } */
}
