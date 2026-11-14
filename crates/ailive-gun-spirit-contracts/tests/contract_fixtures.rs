#![allow(clippy::expect_used, clippy::unwrap_used)]

use ailive_gun_spirit_contracts::{
    generated_schema_documents, AngularRateSample, ContractValidate, DeviceEvent, PerceptionState,
    SensorObservation,
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
        "../../../schemas/fixtures/v0.4/valid/sensor-grip-front.json"
    ));
    decode_valid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.4/valid/sensor-grip-rear.json"
    ));
    decode_valid::<AngularRateSample>(include_str!(
        "../../../schemas/fixtures/v0.4/valid/angular-rate-sample.json"
    ));
    decode_valid::<PerceptionState>(include_str!(
        "../../../schemas/fixtures/v0.4/valid/perception-ready.json"
    ));
    decode_valid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.4/valid/device-held-entered.json"
    ));
    // v0.4 新增的两个值：弹匣就位事件、预供弹主控。
    decode_valid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.4/valid/device-magazine-seated.json"
    ));
    decode_valid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.4/valid/sensor-prefeed-lever.json"
    ));
}

/// 单板形态的**契约层**证明：同一个 `node_uid` 的两条 `GripContact`
/// 仅靠载荷内的判别字段就能分开。这正是 v0.3 抬版本要换来的东西，v0.4 保持不变。
#[test]
fn one_node_can_carry_both_grip_bindings() {
    let front: SensorObservation = serde_json::from_str(include_str!(
        "../../../schemas/fixtures/v0.4/valid/sensor-grip-front.json"
    ))
    .expect("front grip fixture must deserialize");
    let rear: SensorObservation = serde_json::from_str(include_str!(
        "../../../schemas/fixtures/v0.4/valid/sensor-grip-rear.json"
    ))
    .expect("rear grip fixture must deserialize");

    assert_eq!(front.node_uid, rear.node_uid, "两个样例必须来自同一节点");
    assert_ne!(
        front.observation, rear.observation,
        "判别不同，载荷必须不同"
    );
}

#[test]
fn invalid_fixtures_are_rejected() {
    decode_invalid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/sensor-strength-overflow.json"
    ));
    decode_invalid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/sensor-unknown-variant.json"
    ));
    decode_invalid::<SensorObservation>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/sensor-empty-node-uid.json"
    ));
    decode_invalid::<PerceptionState>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/perception-too-many-sources.json"
    ));
    decode_invalid::<PerceptionState>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/perception-unknown-carries-value.json"
    ));
    decode_invalid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/device-forbidden-fired-event.json"
    ));
    decode_invalid::<DeviceEvent>(include_str!(
        "../../../schemas/fixtures/v0.4/invalid/device-baseline-revision.json"
    ));
}

/// 判别字段的两类非法输入必须在**反序列化**阶段就被挡住。
///
/// 这里不复用 `decode_invalid`（它只要求"两条路里有一条失败"）——
/// 缺字段/未知变体是**形状**错误，必须由 `deny_unknown_fields` 与必填字段
/// 在进语义校验之前拦下；若它们溜到语义层才被拒，说明类型本身没兜住。
#[test]
fn grip_binding_shape_errors_fail_at_deserialization() {
    assert!(
        serde_json::from_str::<SensorObservation>(include_str!(
            "../../../schemas/fixtures/v0.4/invalid/sensor-missing-grip-binding.json"
        ))
        .is_err(),
        "缺少 grip 判别字段的 GripContact 必须反序列化失败"
    );
    assert!(
        serde_json::from_str::<SensorObservation>(include_str!(
            "../../../schemas/fixtures/v0.4/invalid/sensor-unknown-grip-binding.json"
        ))
        .is_err(),
        "未定义的 grip 取值必须反序列化失败"
    );
}

/// v0.2 的消息在 v0.3/v0.4 下必须**明确被拒**，而不是"解析通过但语义悄悄变了"。
/// 这是把 `ContractVersion::V0_2` 变体删掉而不是保留的直接理由。
#[test]
fn v0_2_messages_are_rejected_under_current_version() {
    let legacy: &str = include_str!("../../../schemas/fixtures/v0.2/valid/sensor-grip.json");
    assert!(
        serde_json::from_str::<SensorObservation>(legacy).is_err(),
        "标着 0.2 的消息不得在 v0.3 类型下解析成功"
    );
}
