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

/// 场景加载错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioError {
    /// JSON 解析失败（字段类型错误、未知字段等）。
    Parse(String),
    /// 通过解析但违反规范。
    Invalid(Vec<String>),
}

impl core::fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(message) => write!(formatter, "scenario parse error: {message}"),
            Self::Invalid(issues) => {
                write!(formatter, "scenario invalid: {}", issues.join("; "))
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
            return Err(ScenarioError::Invalid(vec![format!(
                "file too large: {} bytes (max {MAX_FILE_BYTES})",
                text.len()
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
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.schema_version != 1 {
            issues.push(format!(
                "schema_version must be 1, got {}",
                self.schema_version
            ));
        }
        if self.id.trim().is_empty() {
            issues.push(String::from("id must not be empty"));
        }
        if self.name.trim().is_empty() {
            issues.push(String::from("name must not be empty"));
        }
        if self.arena.width <= 0.0
            || !self.arena.width.is_finite()
            || self.arena.height <= 0.0
            || !self.arena.height.is_finite()
        {
            issues.push(format!(
                "arena must be positive, got {} x {}",
                self.arena.width, self.arena.height
            ));
        }
        if self.mode.duration_s <= 0.0
            || !self.mode.duration_s.is_finite()
            || self.mode.duration_s > MAX_DURATION_S
        {
            issues.push(format!(
                "duration_s must be in (0, {MAX_DURATION_S}], got {}",
                self.mode.duration_s
            ));
        }
        if self.mode.kind == ModeKind::Wave && self.mode.waves.is_empty() {
            issues.push(String::from("wave mode requires at least one wave"));
        }
        let mut previous_at = f64::NEG_INFINITY;
        for (index, wave) in self.mode.waves.iter().enumerate() {
            if wave.at_s < previous_at {
                issues.push(format!("wave {index}: at_s must be non-decreasing"));
            }
            previous_at = wave.at_s;
            if wave.targets == 0 || wave.targets > MAX_TARGETS {
                issues.push(format!(
                    "wave {index}: targets must be in 1..={MAX_TARGETS}, got {}",
                    wave.targets
                ));
            }
        }
        for (index, cover) in self.cover.iter().enumerate() {
            match cover {
                Cover::Box { w, h, .. } | Cover::Wall { w, h, .. } => {
                    if *w <= 0.0 || !w.is_finite() || *h <= 0.0 || !h.is_finite() {
                        issues.push(format!("cover {index}: w/h must be positive"));
                    }
                }
                Cover::Circle { r, .. } => {
                    if *r <= 0.0 || !r.is_finite() {
                        issues.push(format!("cover {index}: r must be positive"));
                    }
                }
            }
        }
        if self.targets.is_empty() {
            issues.push(String::from("at least one target spec is required"));
        }
        let mut total_targets = 0u32;
        for (index, target) in self.targets.iter().enumerate() {
            if target.count == 0 || target.count > MAX_TARGETS {
                issues.push(format!(
                    "target {index}: count must be in 1..={MAX_TARGETS}, got {}",
                    target.count
                ));
            }
            total_targets = total_targets.saturating_add(target.count);
        }
        if total_targets > MAX_TARGETS {
            issues.push(format!(
                "total targets must be <= {MAX_TARGETS}, got {total_targets}"
            ));
        }
        if let Some(scoring) = &self.scoring {
            if !scoring.precision_bonus_radius_m.is_finite()
                || scoring.precision_bonus_radius_m < 0.0
            {
                issues.push(String::from(
                    "scoring.precision_bonus_radius_m must be finite and >= 0",
                ));
            }
        }
        for (index, cue) in self.audio.iter().enumerate() {
            if !is_safe_relative_path(&cue.asset) {
                issues.push(format!("audio {index}: unsafe asset path {:?}", cue.asset));
            }
        }
        if let Some(presentation) = &self.presentation {
            if !(0.0..=1.0).contains(&presentation.character_visibility) {
                issues.push(format!(
                    "presentation.character_visibility must be in [0,1], got {}",
                    presentation.character_visibility
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
                assert!(issues.iter().any(|issue| issue.contains("schema_version")));
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
                assert!(issues
                    .iter()
                    .any(|issue| issue.contains("unsafe asset path")));
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
                assert!(issues.iter().any(|issue| issue.contains("total targets")));
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
                assert!(issues.iter().any(|issue| issue.contains("arena")));
                assert!(issues.iter().any(|issue| issue.contains("duration_s")));
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
                assert!(issues
                    .iter()
                    .any(|issue| issue.contains("at least one wave")));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
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
