#![allow(clippy::expect_used, clippy::unwrap_used)]

//! 硬约束2（双节点形态零语义变更）的可执行门。
//!
//! 与 `simulator_cli.rs` 的9帧断言相比，本测试比较**整帧的每一个字段**——
//! 断言没写到的字段也在保护范围内。
//!
//! 唯一豁免是 `contract_version`：契约抬版本（v0.2 → v0.3）属于**声明**变更，
//! 不是语义变更；节点模型的泛化不该顺带改动任何物理事实。
//!
//! 该字段在帧里出现于三处嵌套路径（其中两处在数组元素内）：
//!   · `perception_state.contract_version`
//!   · `device_events[].contract_version`
//!   · `role_bridge.sensor_turn.events[].contract_version`
//! 因此归一化按**深度递归**做，不写死路径——否则将来新增一处会被静默漏过。

use serde_json::Value;
use std::process::Command;

const GOLDEN: &str = include_str!("golden/default_lifecycle.jsonl");
const CONTRACT_VERSION_KEY: &str = "contract_version";

/// 把任意深度的 `contract_version` 换成哨兵值，让比较与版本号无关。
fn normalize(value: &mut Value) {
    match value {
        Value::Object(entries) => {
            for (key, child) in entries.iter_mut() {
                if key == CONTRACT_VERSION_KEY {
                    *child = Value::String("<normalized>".to_owned());
                } else {
                    normalize(child);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(normalize),
        _ => {}
    }
}

fn parse_frames(raw: &str) -> Vec<Value> {
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("每行必须是合法JSON"))
        .collect()
}

/// 返回第一处差异的路径与两侧取值；完全一致时返回 `None`。
///
/// 直接对整帧 `assert_eq!` 也能失败，但会把两整帧 JSON 打进日志，
/// 而真正要看的信息（哪个字段）淹没其中。
fn first_difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    match (expected, actual) {
        (Value::Object(expected_entries), Value::Object(actual_entries)) => {
            for key in expected_entries.keys() {
                let child = format!("{path}.{key}");
                match actual_entries.get(key) {
                    Some(actual_child) => {
                        if let Some(found) =
                            first_difference(&expected_entries[key], actual_child, &child)
                        {
                            return Some(found);
                        }
                    }
                    None => return Some(format!("{child}: 期望存在，实际缺失")),
                }
            }
            for key in actual_entries.keys() {
                if !expected_entries.contains_key(key) {
                    return Some(format!("{path}.{key}: 实际多出此字段"));
                }
            }
            None
        }
        (Value::Array(expected_items), Value::Array(actual_items)) => {
            if expected_items.len() != actual_items.len() {
                return Some(format!(
                    "{path}: 数组长度 期望{} 实际{}",
                    expected_items.len(),
                    actual_items.len()
                ));
            }
            for (index, (expected_item, actual_item)) in
                expected_items.iter().zip(actual_items).enumerate()
            {
                let child = format!("{path}[{index}]");
                if let Some(found) = first_difference(expected_item, actual_item, &child) {
                    return Some(found);
                }
            }
            None
        }
        (expected_scalar, actual_scalar) => {
            if expected_scalar == actual_scalar {
                None
            } else {
                Some(format!(
                    "{path}: 期望 {expected_scalar}，实际 {actual_scalar}"
                ))
            }
        }
    }
}

#[test]
fn simulator_frames_keep_every_field_apart_from_contract_version() {
    let expected_frames = parse_frames(GOLDEN);
    let output = Command::new(env!("CARGO_BIN_EXE_ailive-gun-spirit-sim"))
        .output()
        .expect("simulator必须能启动");
    assert!(output.status.success(), "simulator退出码非0");
    let actual_frames =
        parse_frames(&String::from_utf8(output.stdout).expect("simulator输出必须是UTF-8"));

    assert_eq!(
        actual_frames.len(),
        expected_frames.len(),
        "帧数从 {} 变成了 {}",
        expected_frames.len(),
        actual_frames.len()
    );

    for (index, (expected_frame, actual_frame)) in
        expected_frames.iter().zip(&actual_frames).enumerate()
    {
        let mut expected_frame = expected_frame.clone();
        let mut actual_frame = actual_frame.clone();
        normalize(&mut expected_frame);
        normalize(&mut actual_frame);

        if let Some(difference) = first_difference(&expected_frame, &actual_frame, "frame") {
            panic!("第 {index} 帧除 contract_version 外发生了语义变更 → {difference}");
        }
    }
}
