//! 场景（地图）加载器：读取 `SCENARIO_FORMAT.md` 定义的 `.scn.json`，
//! 做**与门禁脚本同等的校验**，并输出可读摘要。
//!
//! 设计纪律（与规范一致）：
//! - `schema_version` 必须为 1；
//! - **不猜测未知字段**：所有结构都 `deny_unknown_fields`；
//! - 枚举（玩法/图元/路径/掩体）走白名单，未知值直接拒绝；
//! - 路径只允许**安全相对路径**（禁绝对路径、`..`、URL）。
//!
//! 本模块**只解析与校验，不做渲染**；渲染与玩法属于靶场（主机侧）。

use serde::Deserialize;

/// 一条校验问题。按"**位置 → 实际值 → 期望 → 原因 → 怎么改**"组织，
/// 让地图作者不必读源码就能自己修好。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// 稳定代码（`arena.size` 等），便于测试断言与文档引用。
    pub code: &'static str,
    /// 字段路径，例如 `targets[0].count`。
    pub path: String,
    /// 实际写下的值。
    pub actual: String,
    /// 期望的范围/白名单。
    pub expected: String,
    /// 为什么有这条限制（安全 / 确定性 / 性能）。
    pub reason: String,
    /// 具体怎么改。
    pub hint: String,
}

impl core::fmt::Display for Issue {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "[{code}] {path} = {actual}\n       期望: {expected}\n       原因: {reason}\n       修改: {hint}",
            code = self.code,
            path = self.path,
            actual = self.actual,
            expected = self.expected,
            reason = self.reason,
            hint = self.hint
        )
    }
}

fn issue(
    code: &'static str,
    path: impl Into<String>,
    actual: impl Into<String>,
    expected: impl Into<String>,
    reason: impl Into<String>,
    hint: impl Into<String>,
) -> Issue {
    Issue {
        code,
        path: path.into(),
        actual: actual.into(),
        expected: expected.into(),
        reason: reason.into(),
        hint: hint.into(),
    }
}

/// 场景加载错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioError {
    /// JSON 解析失败（字段类型错误、未知字段等）。
    Parse(String),
    /// 通过解析但违反规范。
    Invalid(Vec<Issue>),
}

impl core::fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(message) => {
                writeln!(formatter, "JSON 解析失败：")?;
                write!(formatter, "  {message}")
            }
            Self::Invalid(issues) => {
                writeln!(
                    formatter,
                    "场景校验未通过，共 {} 个问题（下面每条都给出位置、期望与修改建议）：",
                    issues.len()
                )?;
                for (index, item) in issues.iter().enumerate() {
                    writeln!(formatter, "  {}. {item}", index + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ScenarioError {}

/// 校验上限（与规范一致）。
pub const MAX_TARGETS: u32 = 64;
pub const MAX_DURATION_S: f64 = 600.0;
pub const MAX_FILE_BYTES: usize = 256 * 1024;

/// 玩法类型白名单。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ModeKind {
    TimeAttack,
    Wave,
    Precision,
    Reaction,
}

/// 内置图元白名单。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Primitive {
    Drone,
    Dummy,
    Plate,
    Box,
    Wall,
}

impl Primitive {
    /// 线格式名称（snake_case），用于摘要与日志。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Drone => "drone",
            Self::Dummy => "dummy",
            Self::Plate => "plate",
            Self::Box => "box",
            Self::Wall => "wall",
        }
    }
}

impl ModeKind {
    /// 线格式名称（snake_case）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TimeAttack => "time_attack",
            Self::Wave => "wave",
            Self::Precision => "precision",
            Self::Reaction => "reaction",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arena {
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wave {
    pub at_s: f64,
    pub targets: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mode {
    pub kind: ModeKind,
    pub duration_s: f64,
    #[serde(default)]
    pub waves: Vec<Wave>,
}

/// 掩体：按 `shape` 判别。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case", deny_unknown_fields)]
pub enum Cover {
    Box {
        x: f64,
        y: f64,
        z: f64,
        w: f64,
        h: f64,
    },
    Circle {
        x: f64,
        y: f64,
        r: f64,
    },
    Wall {
        x: f64,
        y: f64,
        z: f64,
        w: f64,
        h: f64,
    },
}

/// 目标运动路径：按 `kind` 判别。
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetPath {
    Sine {
        amplitude: f64,
        speed: f64,
        #[serde(default)]
        phase_s: f64,
    },
    Linear {
        speed: f64,
        #[serde(default)]
        dx: f64,
    },
    Circle {
        radius: f64,
        speed: f64,
    },
    Static,
}

impl TargetPath {
    /// 人类可读名称，用于摘要输出。
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Sine {
                amplitude, speed, ..
            } => format!("sine(amp={amplitude}, speed={speed})"),
            Self::Linear { speed, .. } => format!("linear(speed={speed})"),
            Self::Circle { radius, speed } => format!("circle(r={radius}, speed={speed})"),
            Self::Static => String::from("static"),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSpec {
    pub template: Primitive,
    #[serde(default = "default_count")]
    pub count: u32,
    #[serde(default = "default_hp")]
    pub hp: u32,
    pub path: TargetPath,
    pub spawn: Spawn,
}

fn default_count() -> u32 {
    1
}

fn default_hp() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scoring {
    pub hit_points: i32,
    #[serde(default)]
    pub miss_penalty: i32,
    #[serde(default)]
    pub precision_bonus_radius_m: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioCue {
    pub id: String,
    pub asset: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    #[serde(default = "default_anchor")]
    pub character_anchor: String,
    #[serde(default = "default_visibility")]
    pub character_visibility: f64,
    #[serde(default)]
    pub tint: Option<String>,
}

fn default_anchor() -> String {
    String::from("left")
}

fn default_visibility() -> f64 {
    0.8
}

/// 场景根结构。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub seed: u64,
    pub arena: Arena,
    pub mode: Mode,
    #[serde(default)]
    pub cover: Vec<Cover>,
    pub targets: Vec<TargetSpec>,
    #[serde(default)]
    pub scoring: Option<Scoring>,
    #[serde(default)]
    pub audio: Vec<AudioCue>,
    #[serde(default)]
    pub presentation: Option<Presentation>,
}

/// 只允许安全相对路径：无盘符、无前导分隔符、无 `..`、无 URL。
#[must_use]
pub fn is_safe_relative_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    if path.contains("..") {
        return false;
    }
    if path.contains("://") {
        return false;
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return false;
    }
    true
}

impl Scenario {
    /// 从 JSON 文本加载并校验。
    pub fn from_json(text: &str) -> Result<Self, ScenarioError> {
        if text.len() > MAX_FILE_BYTES {
            return Err(ScenarioError::Invalid(vec![issue(
                "file.size",
                "(文件)",
                format!("{} 字节", text.len()),
                format!("<= {MAX_FILE_BYTES} 字节"),
                "场景是纯数据文件，超大文件通常意味着资源内联，不便分享与校验",
                "把大资源移到 assets/ 并在 manifest 里登记，或用路径引用",
            )]));
        }
        let scenario: Self =
            serde_json::from_str(text).map_err(|error| ScenarioError::Parse(error.to_string()))?;
        let issues = scenario.validate();
        if issues.is_empty() {
            Ok(scenario)
        } else {
            Err(ScenarioError::Invalid(issues))
        }
    }

    /// 规范校验；返回所有问题（不是遇到第一个就返回），便于一次修完。
    #[must_use]
    pub fn validate(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if self.schema_version != 1 {
            issues.push(issue(
                "schema.version",
                "schema_version",
                self.schema_version.to_string(),
                "1",
                "格式版本决定字段含义；跨版本猜测字段会静默读错数据",
                "把 schema_version 改成 1；若需要新格式，先升级本仓库的场景规范",
            ));
        }
        if self.id.trim().is_empty() {
            issues.push(issue(
                "meta.id",
                "id",
                "(空)".to_owned(),
                "非空字符串，建议 kebab-case，如 basic-range",
                "id 用于分享、存档与回执中的唯一标识",
                "填写一个描述性的 id，例如 \"my-range-01\"",
            ));
        }
        if self.name.trim().is_empty() {
            issues.push(issue(
                "meta.name",
                "name",
                "(空)".to_owned(),
                "非空字符串",
                "名称会显示在场景列表与加载回执里",
                "填写人类可读名称，例如 \"我的靶场\"",
            ));
        }
        if self.arena.width <= 0.0
            || !self.arena.width.is_finite()
            || self.arena.height <= 0.0
            || !self.arena.height.is_finite()
        {
            issues.push(issue(
                "arena.size",
                "arena",
                format!("width={} height={}", self.arena.width, self.arena.height),
                "width/height 均为有限正数（米）",
                "场地尺寸参与命中判定与渲染坐标换算，非正数会让坐标系退化",
                "给出正数，例如 width=12, height=6",
            ));
        }
        if self.mode.duration_s <= 0.0
            || !self.mode.duration_s.is_finite()
            || self.mode.duration_s > MAX_DURATION_S
        {
            issues.push(issue(
                "mode.duration",
                "mode.duration_s",
                self.mode.duration_s.to_string(),
                format!("(0, {MAX_DURATION_S}] 秒"),
                "单局时长上限用于避免无限循环和多局并发的资源失控",
                format!("把 duration_s 改到 {MAX_DURATION_S} 以内"),
            ));
        }
        if self.mode.kind == ModeKind::Wave && self.mode.waves.is_empty() {
            issues.push(issue(
                "mode.waves",
                "mode.waves",
                "[]".to_owned(),
                "至少 1 个波次对象 { at_s, targets }",
                "wave 模式下没有波次就不会生成任何目标，玩家会看到空场",
                "添加波次，例如 [{ \"at_s\": 0, \"targets\": 3 }]",
            ));
        }
        let mut previous_at = f64::NEG_INFINITY;
        for (index, wave) in self.mode.waves.iter().enumerate() {
            if wave.at_s < previous_at {
                issues.push(issue(
                    "mode.waves.order",
                    format!("mode.waves[{index}].at_s"),
                    wave.at_s.to_string(),
                    "不小于前一波次的 at_s（非递减）",
                    "波次顺序决定生成节奏，逆序在固定 seed 下也会产生歧义",
                    format!("把该波次的 at_s 调到不小于 {previous_at}"),
                ));
            }
            previous_at = wave.at_s;
            if wave.targets == 0 || wave.targets > MAX_TARGETS {
                issues.push(issue(
                    "mode.waves.targets",
                    format!("mode.waves[{index}].targets"),
                    wave.targets.to_string(),
                    format!("1..={MAX_TARGETS}"),
                    format!("单波目标数上限 {MAX_TARGETS}，超出会超出渲染与判定预算"),
                    "把该波次的 targets 调进允许范围",
                ));
            }
        }
        for (index, cover) in self.cover.iter().enumerate() {
            match cover {
                Cover::Box { w, h, .. } | Cover::Wall { w, h, .. } => {
                    if *w <= 0.0 || !w.is_finite() || *h <= 0.0 || !h.is_finite() {
                        issues.push(issue(
                            "cover.size",
                            format!("cover[{index}]"),
                            format!("w={w} h={h}"),
                            "w/h 均为有限正数",
                            "非正尺寸的掩体在几何求交时非法",
                            "给出正数宽高，例如 w=1.2, h=1.0",
                        ));
                    }
                }
                Cover::Circle { r, .. } => {
                    if *r <= 0.0 || !r.is_finite() {
                        issues.push(issue(
                            "cover.radius",
                            format!("cover[{index}].r"),
                            r.to_string(),
                            "有限正数",
                            "非正半径的圆形掩体无法求交",
                            "给出正数半径，例如 r=0.6",
                        ));
                    }
                }
            }
        }
        if self.targets.is_empty() {
            issues.push(issue(
                "targets.empty",
                "targets",
                "[]".to_owned(),
                "至少 1 个目标描述（template/count/path/spawn）",
                "没有目标就没有可射击对象，场景无法进行",
                "添加一个目标描述，例如 drone + sine 路径",
            ));
        }
        let mut total_targets = 0u32;
        for (index, target) in self.targets.iter().enumerate() {
            if target.count == 0 || target.count > MAX_TARGETS {
                issues.push(issue(
                    "targets.count",
                    format!("targets[{index}].count"),
                    target.count.to_string(),
                    format!("1..={MAX_TARGETS}"),
                    format!("单条目标描述的实例上限 {MAX_TARGETS}"),
                    "把 count 调进允许范围",
                ));
            }
            total_targets = total_targets.saturating_add(target.count);
        }
        if total_targets > MAX_TARGETS {
            issues.push(issue(
                "targets.total",
                "targets",
                total_targets.to_string(),
                format!("<= {MAX_TARGETS}"),
                format!("全部目标实例合计上限 {MAX_TARGETS}，保证判定与渲染预算可控"),
                "减少某几条描述的 count，或拆成多个场景",
            ));
        }
        if let Some(scoring) = &self.scoring {
            if !scoring.precision_bonus_radius_m.is_finite()
                || scoring.precision_bonus_radius_m < 0.0
            {
                issues.push(issue(
                    "scoring.precision_radius",
                    "scoring.precision_bonus_radius_m",
                    scoring.precision_bonus_radius_m.to_string(),
                    "有限且 >= 0 的米数",
                    "该半径用于判定精度命中是否加分",
                    "给出 0 或正数，例如 0.15",
                ));
            }
        }
        for (index, cue) in self.audio.iter().enumerate() {
            if !is_safe_relative_path(&cue.asset) {
                issues.push(issue(
                    "audio.path",
                    format!("audio[{index}].asset"),
                    format!("{:?}", cue.asset),
                    "安全相对路径，如 sfx/hit.wav（禁绝对路径、..、URL、盘符）",
                    "场景文件会被分享给他人，必须防止读取包外文件（路径穿越）",
                    "改为包内相对路径，不要出现 .. 或盘符",
                ));
            }
        }
        if let Some(presentation) = &self.presentation {
            if !(0.0..=1.0).contains(&presentation.character_visibility) {
                issues.push(issue(
                    "presentation.visibility",
                    "presentation.character_visibility",
                    presentation.character_visibility.to_string(),
                    "[0, 1]",
                    "可见度是渲染透明度，超出范围没有物理意义",
                    "给出 0 到 1 之间的小数，例如 0.8",
                ));
            }
        }
        issues
    }

    /// 目标实例总数（各 spec 的 `count` 之和）。
    #[must_use]
    pub fn total_target_instances(&self) -> u32 {
        self.targets.iter().map(|target| target.count).sum()
    }

    /// 人类可读摘要（一行一条）。
    #[must_use]
    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!(
            "scenario: {} ({}) schema=v{} seed={}",
            self.id, self.name, self.schema_version, self.seed
        ));
        lines.push(format!(
            "  arena: {} x {} (background={})",
            self.arena.width,
            self.arena.height,
            self.arena.background.as_deref().unwrap_or("none")
        ));
        let waves: Vec<String> = self
            .mode
            .waves
            .iter()
            .map(|wave| format!("{}s:{}", wave.at_s, wave.targets))
            .collect();
        lines.push(format!(
            "  mode: {} {}s waves={}",
            self.mode.kind.label(),
            self.mode.duration_s,
            if waves.is_empty() {
                String::from("none")
            } else {
                waves.join(",")
            }
        ));
        let mut counters = [0usize; 3];
        for cover in &self.cover {
            match cover {
                Cover::Box { .. } => counters[0] += 1,
                Cover::Circle { .. } => counters[1] += 1,
                Cover::Wall { .. } => counters[2] += 1,
            }
        }
        lines.push(format!(
            "  cover: {} (box {}, circle {}, wall {})",
            self.cover.len(),
            counters[0],
            counters[1],
            counters[2]
        ));
        lines.push(format!(
            "  targets: {} instances from {} specs ({} total cap {})",
            self.total_target_instances(),
            self.targets.len(),
            self.total_target_instances(),
            MAX_TARGETS
        ));
        for (index, target) in self.targets.iter().enumerate() {
            lines.push(format!(
                "    spec {index}: {} x{} hp{} path={} spawn=({:.1},{:.1},{:.1})",
                target.template.label(),
                target.count,
                target.hp,
                target.path.label(),
                target.spawn.x,
                target.spawn.y,
                target.spawn.z
            ));
        }
        if let Some(scoring) = &self.scoring {
            lines.push(format!(
                "  scoring: hit {:+} miss {:+} precision<= {:.2} m",
                scoring.hit_points, scoring.miss_penalty, scoring.precision_bonus_radius_m
            ));
        }
        lines.push(format!("  audio: {} cue(s)", self.audio.len()));
        if let Some(presentation) = &self.presentation {
            lines.push(format!(
                "  presentation: character={} visibility={:.2}",
                presentation.character_anchor, presentation.character_visibility
            ));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;
    use std::path::PathBuf;

    fn example_path() -> PathBuf {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .join("..")
            .join("..")
            .join("distros")
            .join("example-kit")
            .join("scenarios")
            .join("basic-range.scn.json")
    }

    fn example_json() -> String {
        std::fs::read_to_string(example_path()).expect("read example scenario")
    }

    #[test]
    fn loads_repository_example() {
        let scenario = Scenario::from_json(&example_json()).expect("example should load");
        assert_eq!(scenario.schema_version, 1);
        assert_eq!(scenario.mode.kind, ModeKind::Wave);
        assert_eq!(scenario.mode.waves.len(), 3);
        assert_eq!(scenario.total_target_instances(), 5);
        assert!(scenario.validate().is_empty());
        let summary = scenario.summary_lines().join("\n");
        assert!(summary.contains("drone"));
        assert!(summary.contains("wave"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["mystery_field"] = serde_json::json!(1);
        let error = Scenario::from_json(&value.to_string()).expect_err("unknown field must fail");
        assert!(matches!(error, ScenarioError::Parse(_)), "got {error:?}");
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["schema_version"] = serde_json::json!(2);
        let error = Scenario::from_json(&value.to_string()).expect_err("schema 2 must fail");
        match error {
            ScenarioError::Invalid(issues) => {
                assert!(issues.iter().any(|issue| issue.code == "schema.version"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unsafe_audio_path() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["audio"][0]["asset"] = serde_json::json!("../../etc/passwd");
        let error = Scenario::from_json(&value.to_string()).expect_err("unsafe path must fail");
        match error {
            ScenarioError::Invalid(issues) => {
                assert!(issues.iter().any(|issue| issue.code == "audio.path"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_primitive_and_mode() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["targets"][0]["template"] = serde_json::json!("tank");
        let error =
            Scenario::from_json(&value.to_string()).expect_err("unknown template must fail");
        assert!(matches!(error, ScenarioError::Parse(_)), "got {error:?}");

        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["mode"]["kind"] = serde_json::json!("ranked");
        let error = Scenario::from_json(&value.to_string()).expect_err("unknown mode must fail");
        assert!(matches!(error, ScenarioError::Parse(_)), "got {error:?}");
    }

    #[test]
    fn rejects_over_limit_targets() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["targets"][0]["count"] = serde_json::json!(40);
        value["targets"][1]["count"] = serde_json::json!(40);
        let error = Scenario::from_json(&value.to_string()).expect_err("over limit must fail");
        match error {
            ScenarioError::Invalid(issues) => {
                assert!(issues.iter().any(|issue| issue.code == "targets.total"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_negative_arena_and_long_duration() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["arena"]["width"] = serde_json::json!(-1.0);
        value["mode"]["duration_s"] = serde_json::json!(900.0);
        let error = Scenario::from_json(&value.to_string()).expect_err("must fail");
        match error {
            ScenarioError::Invalid(issues) => {
                assert!(issues.iter().any(|issue| issue.code == "arena.size"));
                assert!(issues.iter().any(|issue| issue.code == "mode.duration"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn wave_mode_requires_waves() {
        let mut value: serde_json::Value = serde_json::from_str(&example_json()).unwrap();
        value["mode"]["waves"] = serde_json::json!([]);
        let error = Scenario::from_json(&value.to_string()).expect_err("must fail");
        match error {
            ScenarioError::Invalid(issues) => {
                assert!(issues.iter().any(|issue| issue.code == "mode.waves"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn diagnostics_state_location_expectation_reason_and_fix() {
        // 报错质量要求：位置、实际值、期望、原因、怎么改——五段齐全，且一次报全。
        let scenario = Scenario {
            schema_version: 2,
            id: String::from("x"),
            name: String::from("x"),
            author: None,
            seed: 0,
            arena: Arena {
                width: 1.0,
                height: 1.0,
                background: None,
            },
            mode: Mode {
                kind: ModeKind::Wave,
                duration_s: 900.0,
                waves: Vec::new(),
            },
            cover: Vec::new(),
            targets: vec![TargetSpec {
                template: Primitive::Drone,
                count: 1,
                hp: 1,
                path: TargetPath::Static,
                spawn: Spawn {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            }],
            scoring: None,
            audio: vec![AudioCue {
                id: String::from("hit"),
                asset: String::from("../escape.wav"),
            }],
            presentation: None,
        };
        let issues = scenario.validate();
        assert!(issues.len() >= 4, "expected several issues, got {issues:?}");
        let rendered = issues
            .iter()
            .map(|issue| issue.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        for required in ["期望:", "原因:", "修改:"] {
            assert!(
                rendered.contains(required),
                "missing {required} in {rendered}"
            );
        }
        for issue in &issues {
            assert!(!issue.path.is_empty(), "empty path: {issue:?}");
            assert!(!issue.actual.is_empty(), "empty actual: {issue:?}");
            assert!(!issue.hint.is_empty(), "empty hint: {issue:?}");
        }
        let codes: Vec<&str> = issues.iter().map(|issue| issue.code).collect();
        assert!(codes.contains(&"schema.version"));
        assert!(codes.contains(&"mode.duration"));
        assert!(codes.contains(&"mode.waves"));
        assert!(codes.contains(&"audio.path"));
    }

    #[test]
    fn safe_relative_path_rules() {
        assert!(is_safe_relative_path("sfx/hit.wav"));
        assert!(!is_safe_relative_path("/abs/path"));
        assert!(!is_safe_relative_path("../escape"));
        assert!(!is_safe_relative_path("C:/windows"));
        assert!(!is_safe_relative_path("https://example.invalid/a.wav"));
        assert!(!is_safe_relative_path(""));
    }
}
