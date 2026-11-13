#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Scenario liveness for the duplicate-input and TTL-expiry coverage.
//!
//! The golden frame test protects the default 9-frame lifecycle. These tests
//! pin the two new presets to the reducer semantics they are meant to expose:
//! same-revision resends must not advance the revision, and stale facts must
//! degrade to `Unknown(Stale)` before node health turns offline.

use serde_json::Value;
use std::process::Command;

fn run_simulator(arguments: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ailive-gun-spirit-sim"))
        .args(arguments)
        .output()
        .expect("simulator must start");
    assert!(output.status.success(), "simulator exited unsuccessfully");
    String::from_utf8(output.stdout).expect("simulator output must be UTF-8")
}

fn parse_frames(raw: &str) -> Vec<Value> {
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("each simulator line must be JSON"))
        .collect()
}

fn fact_value(frame: &Value, fact: &str) -> (String, String) {
    let fact = &frame["perception_state"][fact];
    if fact["status"] == "known" {
        (
            "known".to_owned(),
            fact["value"]
                .as_str()
                .expect("known fact must carry a string value")
                .to_owned(),
        )
    } else {
        (
            "unknown".to_owned(),
            fact["reason"]
                .as_str()
                .expect("unknown fact must carry a reason")
                .to_owned(),
        )
    }
}

#[test]
fn explicit_default_scenario_matches_no_argument_output() {
    let implicit = run_simulator(&[]);
    let explicit = run_simulator(&["--scenario", "default"]);
    assert_eq!(implicit, explicit);
}

#[test]
fn repeat_scenario_resends_without_advancing_revision() {
    let frames = parse_frames(&run_simulator(&["--scenario", "repeat"]));
    assert_eq!(frames.len(), 11);

    // baseline, held -> first semantic advance
    assert_eq!(frames[0]["command"], "baseline");
    assert_eq!(frames[1]["command"], "held");
    assert_eq!(frames[1]["disposition"], "advanced");
    assert_eq!(frames[1]["events"][0], "carrier.held_entered");
    let held_revision = frames[1]["revision"].as_u64().expect("revision is u64");

    // A fresh observation carrying the same value, the exact same observation
    // delivered again, and an idle tick are all same-revision resends.
    for (offset, frame) in frames[2..=4].iter().enumerate() {
        let index = offset + 2;
        assert_eq!(frame["disposition"], "resend", "frame {index}");
        assert_eq!(frame["events"], serde_json::json!([]));
        assert_eq!(
            frame["revision"].as_u64(),
            Some(held_revision),
            "frame {index} must keep revision {held_revision}"
        );
    }
    assert_eq!(frames[2]["command"], "held");
    assert_eq!(frames[3]["command"], "duplicate");
    assert_eq!(frames[4]["command"], "tick");

    // Duplicate input must not poison the next real edge.
    assert_eq!(frames[5]["command"], "ready");
    assert_eq!(frames[5]["disposition"], "advanced");
    assert_eq!(frames[5]["events"][0], "carrier.ready_entered");
    assert!(frames[5]["revision"].as_u64().unwrap() > held_revision);

    // A duplicate of the ready edge stays on the same revision.
    assert_eq!(frames[6]["command"], "duplicate");
    assert_eq!(frames[6]["disposition"], "resend");
    assert_eq!(frames[6]["revision"], frames[5]["revision"]);

    // A repeated aux-control value is a refresh, and its duplicate is ignored.
    assert_eq!(frames[7]["command"], "control-on");
    assert_eq!(frames[7]["disposition"], "advanced");
    assert_eq!(frames[8]["command"], "control-on");
    assert_eq!(frames[8]["disposition"], "resend");
    assert_eq!(frames[8]["revision"], frames[7]["revision"]);
    assert_eq!(frames[9]["command"], "duplicate");
    assert_eq!(frames[9]["disposition"], "resend");
    assert_eq!(frames[9]["revision"], frames[7]["revision"]);

    assert_eq!(frames[10]["command"], "control-off");
    assert_eq!(frames[10]["disposition"], "advanced");
    assert_eq!(frames[10]["events"][0], "control.primary.released");
}

#[test]
fn stale_scenario_expires_facts_before_nodes_go_offline() {
    let frames = parse_frames(&run_simulator(&["--scenario", "stale"]));
    assert_eq!(frames.len(), 6);
    assert_eq!(frames[2]["command"], "stale");

    // Facts cross their TTL while both nodes are still inside the offline TTL.
    for fact in [
        "carrier_state",
        "front_grip",
        "rear_grip",
        "motion",
        "pose",
        "primary_control",
    ] {
        assert_eq!(
            fact_value(&frames[2], fact),
            ("unknown".to_owned(), "stale".to_owned())
        );
    }
    for health in frames[2]["perception_state"]["node_health"]
        .as_array()
        .expect("node_health is an array")
    {
        assert_eq!(health["state"], "healthy");
    }

    // A fresh observation recovers the carrier without inventing the other facts.
    assert_eq!(frames[3]["command"], "held");
    assert_eq!(frames[3]["disposition"], "advanced");
    assert_eq!(
        fact_value(&frames[3], "carrier_state"),
        ("known".to_owned(), "held".to_owned())
    );
    assert_eq!(
        fact_value(&frames[3], "front_grip"),
        ("unknown".to_owned(), "stale".to_owned())
    );

    // The recovered observation itself is duplicated once and ignored.
    assert_eq!(frames[4]["command"], "duplicate");
    assert_eq!(frames[4]["disposition"], "resend");
    assert_eq!(frames[4]["revision"], frames[3]["revision"]);

    // Crossing the node-offline TTL reports NodeOffline, not Stale.
    assert_eq!(frames[5]["command"], "unknown");
    for fact in ["carrier_state", "front_grip", "rear_grip"] {
        assert_eq!(
            fact_value(&frames[5], fact),
            ("unknown".to_owned(), "node_offline".to_owned())
        );
    }
    for health in frames[5]["perception_state"]["node_health"]
        .as_array()
        .expect("node_health is an array")
    {
        assert_eq!(health["state"], "offline");
    }
}

#[test]
fn simulator_rejects_unknown_scenario() {
    let output = Command::new(env!("CARGO_BIN_EXE_ailive-gun-spirit-sim"))
        .args(["--scenario", "expired"])
        .output()
        .expect("simulator must start");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("AILIVE_GUN_SPIRIT_SIM_INVALID"));
}
