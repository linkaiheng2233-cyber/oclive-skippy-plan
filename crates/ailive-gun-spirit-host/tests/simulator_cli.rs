#![allow(clippy::expect_used, clippy::unwrap_used)]

use serde_json::Value;
use std::process::Command;

#[test]
fn default_simulator_replays_expected_lifecycle() {
    let output = Command::new(env!("CARGO_BIN_EXE_ailive-gun-spirit-sim"))
        .output()
        .expect("simulator must start");
    assert!(output.status.success());

    let frames: Vec<Value> = String::from_utf8(output.stdout)
        .expect("simulator output must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("each simulator line must be JSON"))
        .collect();
    assert_eq!(frames.len(), 9);

    assert_eq!(frames[0]["disposition"], "baseline");
    assert_eq!(frames[0]["observation_count"], 5);
    assert_eq!(
        frames[0]["perception_state"]["carrier_state"]["value"],
        "standby"
    );
    assert_eq!(frames[1]["events"][0], "carrier.held_entered");
    assert_eq!(
        frames[1]["device_events"][0]["kind"],
        "carrier.held_entered"
    );
    assert_eq!(frames[1]["role_bridge"]["source"], "desktop_fixture");
    assert_eq!(frames[1]["role_bridge"]["oclive_dispatched"], false);
    assert_eq!(frames[1]["role_bridge"]["sensor_turn"]["origin"], "sensor");
    assert_eq!(
        frames[1]["screen_view_model"]["primary_layer"]["kind"],
        "role"
    );
    assert_eq!(frames[2]["events"][0], "carrier.ready_entered");
    assert_eq!(frames[3]["events"][0], "control.primary.engaged");
    assert_eq!(frames[4]["events"][0], "control.primary.released");
    assert_eq!(frames[5]["command"], "reboot");
    assert_eq!(frames[5]["disposition"], "baseline");
    assert_eq!(frames[5]["events"], serde_json::json!([]));
    assert_eq!(
        frames[5]["perception_state"]["carrier_state"]["value"],
        "ready"
    );
    assert_eq!(frames[6]["events"][0], "carrier.ready_exited");
    assert_eq!(frames[7]["events"][0], "carrier.held_exited");
    assert_eq!(frames[8]["events"], serde_json::json!([]));
    assert_eq!(
        frames[8]["perception_state"]["front_grip"]["reason"],
        "node_offline"
    );
    assert_eq!(
        frames[8]["screen_view_model"]["primary_layer"]["kind"],
        "system"
    );
}

#[test]
fn simulator_rejects_unknown_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_ailive-gun-spirit-sim"))
        .arg("weapon-fired")
        .output()
        .expect("simulator must start");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("AILIVE_GUN_SPIRIT_SIM_INVALID"));
}

#[test]
fn simulator_repeats_one_hundred_lifecycles_without_stuck_state() {
    const LIFECYCLE: &[&str] = &[
        "baseline",
        "held",
        "ready",
        "control-on",
        "control-off",
        "reboot",
        "lower",
        "standby",
        "unknown",
    ];
    let mut command = Command::new(env!("CARGO_BIN_EXE_ailive-gun-spirit-sim"));
    for _ in 0..100 {
        command.args(LIFECYCLE);
    }
    let output = command.output().expect("simulator must start");
    assert!(output.status.success());

    let frames: Vec<Value> = String::from_utf8(output.stdout)
        .expect("simulator output must be UTF-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("each simulator line must be JSON"))
        .collect();
    assert_eq!(frames.len(), LIFECYCLE.len() * 100);
    for cycle in 0..100 {
        let first = cycle * LIFECYCLE.len();
        assert_eq!(frames[first]["disposition"], "baseline");
        assert_eq!(
            frames[first + 7]["perception_state"]["carrier_state"]["value"],
            "standby"
        );
        assert_eq!(
            frames[first + 8]["screen_view_model"]["primary_layer"]["kind"],
            "system"
        );
    }
}
