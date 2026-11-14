//! 发行版（distro）最小运行器：读取 `distros/` 下的发行版包，校验它，
//! 并按**反应表**把设备事件映射成表情。
//!
//! 本模块**只做**三件事：
//! 1. 解析 `manifest.json` / `expressions.json` / `reactions.json` / `presets.json`；
//! 2. 校验运行时真正会用到的一致性（版本、表情引用、事件名、阈值范围）；
//! 3. 给出"某事件发生时该显示什么表情"的判定（含冷却与优先级）。
//!
//! 它**不负责**渲染、不负责安装/回滚、不读取场景（场景属于靶场）。
//! 文件级完整性（sha256/字节数/路径穿越）由 `scripts/check-distro.ps1` 在门禁里守。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// 发行版加载/校验错误。
#[derive(Debug)]
pub enum DistroError {
    /// 文件缺失、不可读或不是合法 JSON。
    Io { path: PathBuf, message: String },
    /// JSON 形状不合法（未知字段、类型错误）。
    Shape { path: PathBuf, message: String },
    /// 通过解析但违反语义约束。
    Invalid(Vec<String>),
}

impl core::fmt::Display for DistroError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Io { path, message } => write!(formatter, "{}: {message}", path.display()),
            Self::Shape { path, message } => {
                write!(formatter, "{}: 形状不合法：{message}", path.display())
            }
            Self::Invalid(issues) => {
                writeln!(formatter, "发行版校验未通过，共 {} 个问题：", issues.len())?;
                for (index, issue) in issues.iter().enumerate() {
                    writeln!(formatter, "  {}. {issue}", index + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for DistroError {}

/// 固件固定支持的 7 个表情 id。发行版必须完整覆盖，否则固件侧会出现"没有对应图"。
pub const FIRMWARE_EXPRESSIONS: [&str; 7] = [
    "neutral", "happy", "excited", "shy", "confused", "sad", "angry",
];

/// 事件名 → 线格式 kind code（与 `ailive-gun-spirit-protocol::event` 常量一致）。
#[must_use]
pub fn kind_code_for_event(name: &str) -> Option<u8> {
    Some(match name {
        "carrier.held_entered" => 0,
        "carrier.held_exited" => 1,
        "carrier.ready_entered" => 2,
        "carrier.ready_exited" => 3,
        "control.primary.engaged" => 4,
        "control.primary.released" => 5,
        "magazine.seated" => 6,
        _ => return None,
    })
}

/// kind code → 事件名（用于日志与摘要）。
#[must_use]
pub fn event_name_for_kind(code: u8) -> Option<&'static str> {
    Some(match code {
        0 => "carrier.held_entered",
        1 => "carrier.held_exited",
        2 => "carrier.ready_entered",
        3 => "carrier.ready_exited",
        4 => "control.primary.engaged",
        5 => "control.primary.released",
        6 => "magazine.seated",
        _ => return None,
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    author: Option<String>,
    distro_version: String,
    role_ref: RoleRef,
    requires_firmware: String,
    requires_contract: String,
    #[serde(default)]
    limits: Option<Limits>,
    #[serde(default)]
    files: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Limits {
    max_total_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoleRef {
    id: String,
    version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionDocument {
    schema_version: u32,
    vocabulary: Vec<ExpressionEntry>,
    default: String,
    #[serde(default)]
    manual_hold_s: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionEntry {
    id: String,
    label: String,
    asset: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReactionDocument {
    schema_version: u32,
    reactions: Vec<Reaction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reaction {
    event: String,
    expression: String,
    cooldown_s: f64,
    priority: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PresetDocument {
    schema_version: u32,
    presets: Presets,
    #[serde(default)]
    offline: Option<OfflinePresets>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Presets {
    expression_min_dwell_s: f64,
    perception_stable_window_s: f64,
    pose_raised_threshold_deg: f64,
    grip_strength_threshold_milli: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OfflinePresets {
    idle_expression: String,
    touch_expression: String,
    #[serde(default)]
    auto_rotate: bool,
}

/// 一次反应命中的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedReaction {
    /// 应切换到的表情 id。
    pub expression: String,
    /// 命中的规则事件名（便于解释"为什么是这个表情"）。
    pub event: String,
    /// 该规则的优先级。
    pub priority: u8,
}

/// 运行时状态：冷却计时。
#[derive(Debug)]
struct CooldownState {
    /// 事件名 → 上次触发时间（毫秒）。
    last_fired_ms: BTreeMap<String, u64>,
}

/// 加载后的发行版。
#[derive(Debug)]
pub struct Distro {
    manifest_id: String,
    manifest_name: String,
    manifest_author: Option<String>,
    distro_version: String,
    requires_firmware: String,
    max_total_bytes: Option<u64>,
    file_count: usize,
    role_id: String,
    role_version: String,
    requires_contract: String,
    expressions: BTreeMap<String, (String, String)>,
    default_expression: String,
    manual_hold_ms: u64,
    reactions: Vec<Reaction>,
    min_dwell_ms: u64,
    offline_idle: Option<String>,
    offline_touch: Option<String>,
    offline_auto_rotate: bool,
    cooldown: CooldownState,
    /// 当前表情（由最近一次命中或默认值决定）。
    current: String,
    /// 当前表情的锁定截止时间（手动/最短驻留）。
    hold_until_ms: u64,
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, DistroError> {
    let text = std::fs::read_to_string(path).map_err(|error| DistroError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|error| DistroError::Shape {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

impl Distro {
    /// 从目录加载并校验一个发行版。
    pub fn load(directory: &Path) -> Result<Self, DistroError> {
        let manifest: Manifest = read_json(&directory.join("manifest.json"))?;
        let expressions: ExpressionDocument = read_json(&directory.join("expressions.json"))?;
        let reactions: ReactionDocument = read_json(&directory.join("reactions.json"))?;
        let presets: PresetDocument = read_json(&directory.join("presets.json"))?;

        let mut issues = Vec::new();
        if manifest.schema_version != 1 {
            issues.push(format!(
                "manifest.schema_version 应为 1，实际 {}",
                manifest.schema_version
            ));
        }
        if expressions.schema_version != 1 {
            issues.push(format!(
                "expressions.schema_version 应为 1，实际 {}",
                expressions.schema_version
            ));
        }
        if reactions.schema_version != 1 {
            issues.push(format!(
                "reactions.schema_version 应为 1，实际 {}",
                reactions.schema_version
            ));
        }
        if presets.schema_version != 1 {
            issues.push(format!(
                "presets.schema_version 应为 1，实际 {}",
                presets.schema_version
            ));
        }

        // 表情词表：唯一 + 覆盖固件 7 个 id + 默认值存在。
        let mut expression_map = BTreeMap::new();
        for entry in &expressions.vocabulary {
            if expression_map
                .insert(entry.id.clone(), (entry.label.clone(), entry.asset.clone()))
                .is_some()
            {
                issues.push(format!("expressions.vocabulary 重复 id：{}", entry.id));
            }
        }
        for required in FIRMWARE_EXPRESSIONS {
            if !expression_map.contains_key(required) {
                issues.push(format!(
                    "expressions.vocabulary 缺少固件固定 id：{required}（设备端没有对应图）"
                ));
            }
        }
        if !expression_map.contains_key(&expressions.default) {
            issues.push(format!(
                "expressions.default = {} 不在词表中",
                expressions.default
            ));
        }

        // 反应表：事件名合法、表情存在、冷却与优先级范围合理。
        for (index, reaction) in reactions.reactions.iter().enumerate() {
            if kind_code_for_event(&reaction.event).is_none() {
                issues.push(format!(
                    "reactions[{index}].event = {} 不是已知事件（见协议事件表）",
                    reaction.event
                ));
            }
            if !expression_map.contains_key(&reaction.expression) {
                issues.push(format!(
                    "reactions[{index}].expression = {} 不在表情词表中",
                    reaction.expression
                ));
            }
            if !(0.0..=60.0).contains(&reaction.cooldown_s) {
                issues.push(format!(
                    "reactions[{index}].cooldown_s = {} 应在 [0, 60]",
                    reaction.cooldown_s
                ));
            }
            if !(1..=5).contains(&reaction.priority) {
                issues.push(format!(
                    "reactions[{index}].priority = {} 应在 [1, 5]",
                    reaction.priority
                ));
            }
        }

        // 预设：范围检查（与 check-distro.ps1 同口径，但运行时也要挡）。
        if !(0.1..=5.0).contains(&presets.presets.expression_min_dwell_s) {
            issues.push(format!(
                "presets.expression_min_dwell_s = {} 应在 [0.1, 5]",
                presets.presets.expression_min_dwell_s
            ));
        }
        if !(0.0..=2.0).contains(&presets.presets.perception_stable_window_s) {
            issues.push(format!(
                "presets.perception_stable_window_s = {} 应在 [0, 2]",
                presets.presets.perception_stable_window_s
            ));
        }
        if !(1.0..=45.0).contains(&presets.presets.pose_raised_threshold_deg) {
            issues.push(format!(
                "presets.pose_raised_threshold_deg = {} 应在 [1, 45]",
                presets.presets.pose_raised_threshold_deg
            ));
        }
        if !(0..=1000).contains(&presets.presets.grip_strength_threshold_milli) {
            issues.push(format!(
                "presets.grip_strength_threshold_milli = {} 应在 [0, 1000]",
                presets.presets.grip_strength_threshold_milli
            ));
        }
        if let Some(offline) = &presets.offline {
            if !expression_map.contains_key(&offline.idle_expression) {
                issues.push(format!(
                    "presets.offline.idle_expression = {} 不在表情词表中",
                    offline.idle_expression
                ));
            }
            if !expression_map.contains_key(&offline.touch_expression) {
                issues.push(format!(
                    "presets.offline.touch_expression = {} 不在表情词表中",
                    offline.touch_expression
                ));
            }
        }

        if !issues.is_empty() {
            return Err(DistroError::Invalid(issues));
        }

        let manual_hold_ms = (expressions.manual_hold_s.clamp(0.0, 30.0) * 1000.0) as u64;
        let min_dwell_ms =
            (presets.presets.expression_min_dwell_s.clamp(0.0, 30.0) * 1000.0) as u64;
        let default_expression = expressions.default.clone();
        Ok(Self {
            manifest_id: manifest.id,
            manifest_name: manifest.name,
            manifest_author: manifest.author,
            distro_version: manifest.distro_version,
            requires_firmware: manifest.requires_firmware,
            max_total_bytes: manifest
                .limits
                .as_ref()
                .map(|limits| limits.max_total_bytes),
            file_count: manifest.files.len(),
            role_id: manifest.role_ref.id,
            role_version: manifest.role_ref.version,
            requires_contract: manifest.requires_contract,
            expressions: expression_map,
            default_expression: default_expression.clone(),
            manual_hold_ms,
            reactions: reactions.reactions,
            min_dwell_ms,
            offline_idle: presets
                .offline
                .as_ref()
                .map(|offline| offline.idle_expression.clone()),
            offline_touch: presets
                .offline
                .as_ref()
                .map(|offline| offline.touch_expression.clone()),
            offline_auto_rotate: presets
                .offline
                .as_ref()
                .map(|offline| offline.auto_rotate)
                .unwrap_or(false),
            cooldown: CooldownState {
                last_fired_ms: BTreeMap::new(),
            },
            current: default_expression,
            hold_until_ms: 0,
        })
    }

    /// 发行版标识（manifest.id）。
    #[must_use]
    pub fn id(&self) -> &str {
        &self.manifest_id
    }

    /// 人类可读名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.manifest_name
    }

    /// 发行版版本。
    #[must_use]
    pub fn version(&self) -> &str {
        &self.distro_version
    }

    /// 作者（可选）。
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.manifest_author.as_deref()
    }

    /// 需求固件版本范围。
    #[must_use]
    pub fn requires_firmware(&self) -> &str {
        &self.requires_firmware
    }

    /// 声明的总大小上限（字节，可选）。
    #[must_use]
    pub fn max_total_bytes(&self) -> Option<u64> {
        self.max_total_bytes
    }

    /// manifest 里登记的文件数。
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.file_count
    }

    /// 离线模式是否自动轮换表情。
    #[must_use]
    pub fn offline_auto_rotate(&self) -> bool {
        self.offline_auto_rotate
    }

    /// 引用的角色（id, 版本范围）。
    #[must_use]
    pub fn role_ref(&self) -> (&str, &str) {
        (&self.role_id, &self.role_version)
    }

    /// 需要的感知契约版本。
    #[must_use]
    pub fn requires_contract(&self) -> &str {
        &self.requires_contract
    }

    /// 表情词表大小。
    #[must_use]
    pub fn expression_count(&self) -> usize {
        self.expressions.len()
    }

    /// 当前表情。
    #[must_use]
    pub fn current_expression(&self) -> &str {
        &self.current
    }

    /// 离线空闲表情（未配置则为默认）。
    #[must_use]
    pub fn offline_idle_expression(&self) -> &str {
        self.offline_idle
            .as_deref()
            .unwrap_or(&self.default_expression)
    }

    /// 离线触摸表情。
    #[must_use]
    pub fn offline_touch_expression(&self) -> &str {
        self.offline_touch
            .as_deref()
            .unwrap_or(&self.default_expression)
    }

    /// 表情的最短驻留时间（毫秒）。
    #[must_use]
    pub fn min_dwell_ms(&self) -> u64 {
        self.min_dwell_ms
    }

    /// 手动选择表情时的保持时间（毫秒）。
    #[must_use]
    pub fn manual_hold_ms(&self) -> u64 {
        self.manual_hold_ms
    }

    /// 处理一次设备事件；命中且未被冷却/驻留拦住时返回应切换的表情。
    ///
    /// 规则：同一事件受 `cooldown_s` 限制；当前表情在最短驻留期内不被打断
    /// （除非新规则优先级更高）。
    pub fn on_event(&mut self, kind_code: u8, now_ms: u64) -> Option<AppliedReaction> {
        let event_name = event_name_for_kind(kind_code)?;
        let reaction = self
            .reactions
            .iter()
            .filter(|candidate| candidate.event == event_name)
            .max_by_key(|candidate| candidate.priority)?;

        let last = self.cooldown.last_fired_ms.get(event_name).copied();
        let cooldown_ms = (reaction.cooldown_s.clamp(0.0, 60.0) * 1000.0) as u64;
        if let Some(last_ms) = last {
            if now_ms.saturating_sub(last_ms) < cooldown_ms {
                return None;
            }
        }

        // 最短驻留：低优先级事件不得打断当前表情。
        let current_priority = self
            .reactions
            .iter()
            .filter(|candidate| candidate.event == event_name)
            .map(|candidate| candidate.priority)
            .max()
            .unwrap_or(0);
        if now_ms < self.hold_until_ms && current_priority < reaction.priority {
            return None;
        }

        self.cooldown
            .last_fired_ms
            .insert(event_name.to_owned(), now_ms);
        self.current.clone_from(&reaction.expression);
        self.hold_until_ms = now_ms.saturating_add(self.min_dwell_ms);
        Some(AppliedReaction {
            expression: reaction.expression.clone(),
            event: event_name.to_owned(),
            priority: reaction.priority,
        })
    }

    /// 手动指定表情（触摸/主机下发），返回是否被接受。
    pub fn apply_manual(&mut self, expression: &str, now_ms: u64) -> bool {
        if !self.expressions.contains_key(expression) {
            return false;
        }
        self.current = expression.to_owned();
        self.hold_until_ms = now_ms.saturating_add(self.manual_hold_ms);
        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    fn example_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("distros")
            .join("example-kit")
    }

    #[test]
    fn loads_repository_example_distro() {
        let distro = Distro::load(&example_dir()).expect("example distro must load");
        assert_eq!(distro.id(), "mumu-range-kit");
        assert_eq!(distro.version(), "0.1.0");
        assert_eq!(distro.requires_contract(), "0.3");
        assert_eq!(distro.role_ref().0, "mumu");
        assert_eq!(distro.expression_count(), 7);
        assert_eq!(distro.current_expression(), "neutral");
        assert_eq!(distro.offline_idle_expression(), "neutral");
        assert_eq!(distro.offline_touch_expression(), "happy");
    }

    #[test]
    fn maps_known_events_and_rejects_unknown_codes() {
        assert_eq!(kind_code_for_event("magazine.seated"), Some(6));
        assert_eq!(kind_code_for_event("control.primary.engaged"), Some(4));
        assert_eq!(kind_code_for_event("carrier.held_entered"), Some(0));
        assert_eq!(kind_code_for_event("nope"), None);
        assert_eq!(event_name_for_kind(6), Some("magazine.seated"));
        assert_eq!(event_name_for_kind(9), None);

        let mut distro = Distro::load(&example_dir()).unwrap();
        assert!(distro.on_event(9, 0).is_none());
    }

    #[test]
    fn applies_reaction_and_respects_cooldown() {
        let mut distro = Distro::load(&example_dir()).unwrap();
        // control.primary.engaged -> excited, cooldown 0.3 s
        let applied = distro.on_event(4, 1_000).expect("first press must react");
        assert_eq!(applied.expression, "excited");
        assert_eq!(applied.event, "control.primary.engaged");
        assert_eq!(distro.current_expression(), "excited");

        // 冷却期内同事件不再触发
        assert!(distro.on_event(4, 1_100).is_none());
        // 冷却结束后恢复
        assert!(distro.on_event(4, 1_400).is_some());
    }

    #[test]
    fn reaction_picks_highest_priority_rule() {
        let mut distro = Distro::load(&example_dir()).unwrap();
        // carrier.held_entered 规则 priority 3，cooldown 3 s
        let applied = distro.on_event(0, 5_000).expect("held_entered must react");
        assert_eq!(applied.expression, "happy");
        assert_eq!(applied.priority, 3);
    }

    #[test]
    fn manual_expression_must_exist() {
        let mut distro = Distro::load(&example_dir()).unwrap();
        assert!(distro.apply_manual("happy", 0));
        assert_eq!(distro.current_expression(), "happy");
        assert!(!distro.apply_manual("does-not-exist", 0));
        assert_eq!(distro.current_expression(), "happy");
    }

    #[test]
    fn invalid_reaction_expression_is_reported() {
        // 用临时目录构造"反应表引用了不存在的表情"，验证校验确实拦住。
        let directory = std::env::temp_dir().join("gs-distro-invalid-test");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let copy = |name: &str| {
            std::fs::copy(example_dir().join(name), directory.join(name)).expect("copy");
        };
        copy("manifest.json");
        copy("expressions.json");
        copy("presets.json");
        let reactions = std::fs::read_to_string(example_dir().join("reactions.json")).unwrap();
        let broken = reactions.replacen(
            "\"expression\": \"happy\"",
            "\"expression\": \"nonexistent\"",
            1,
        );
        std::fs::write(directory.join("reactions.json"), broken).unwrap();

        let error = Distro::load(&directory).expect_err("must reject");
        match error {
            DistroError::Invalid(issues) => {
                assert!(issues.iter().any(|issue| issue.contains("不在表情词表中")));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&directory);
    }
}
