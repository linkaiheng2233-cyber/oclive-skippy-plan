#![allow(clippy::expect_used, clippy::unwrap_used)]

use ailive_gun_spirit_contracts::{
    generated_schema_documents, ContractValidate, DeviceEvent, PerceptionState, SensorObservation,
};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

fn decode_valid<T>(json: &str)
where
    T: DeserializeOwned + ContractValidate,
{
    let value: T = serde_json::from_str(json).expect("valid fixture must deserialize");
    value.validate().expect("valid fixture must pass semantics");
}

fn decode_invalid<T>(json: &str)
where
    T: DeserializeOwned + ContractValidate,
{
    if let Ok(value) = serde_json::from_str::<T>(json) {
        assert!(
            value.validate().is_err(),
            "invalid fixture unexpectedly passed semantic validation"
        );
    }
}

#[test]
fn committed_schemas_match_rust_types() {
    let schema_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    for schema in generated_schema_documents().expect("schemas must generate") {
        let path = schema_root.join(schema.file_name);
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert_eq!(
            committed,
            schema.json,
            "{} drifted; run the generate_schemas example",
            path.display()
        );
    }
}

#[test]
fn valid_fixtures_decode_and_validate() {
    decode_valid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.2/valid/sensor-grip.json"
    ));
    decode_valid::<PerceptionState>(include_str!(
        "../../../schemas/fixtures/v0.2/valid/perception-ready.json"
    ));
    decode_valid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.2/valid/device-held-entered.json"
    ));
}

#[test]
fn invalid_fixtures_are_rejected() {
    decode_invalid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/sensor-strength-overflow.json"
    ));
    decode_invalid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/sensor-unknown-variant.json"
    ));
    decode_invalid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/sensor-empty-node-uid.json"
    ));
    decode_invalid::<PerceptionState>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/perception-too-many-sources.json"
    ));
    decode_invalid::<PerceptionState>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/perception-unknown-carries-value.json"
    ));
    decode_invalid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/device-forbidden-fired-event.json"
    ));
    decode_invalid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.2/invalid/device-baseline-revision.json"
    ));
}
