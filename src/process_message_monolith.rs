#![allow(dead_code)]
// 此文件由 oclive-cli 根据 monolith.toml 生成。
// 请勿手改焊接逻辑；修改 monolith.toml 后请运行 `oclive build`（或重新 `oclive init`）再生成。
//
// 蓝图 v2/v3：见项目根 docs/BLUEPRINT_V2_POINTER.md（主仓 creator-docs/role-pack/）。
// 桌面宿主主路径仍以 oclivenewnew 内 `process_message` 为准；本文件仅演示 Monolith 七焊接键静态调用顺序。

/// Monolith 入口：演示七焊接键调用顺序（已焊接 → 静态 `oclive_monolith_builtin`；未焊接 → trait 占位）。
pub fn run_monolith_pipeline_demo() {
    oclive_monolith_builtin::ensure_linked();
    slot_memory::run();
    slot_emotion::run();
    slot_event::run();
    slot_prompt::run();
    slot_llm::run();
    slot_agent::run();
    slot_complex_emotion::run();
    println!("monolith: pipeline completed");
}

mod slot_memory {
    pub fn run() {
        oclive_monolith_builtin::memory::invoke();
    }
}

mod slot_emotion {
    pub fn run() {
        oclive_monolith_builtin::emotion::invoke();
    }
}

mod slot_event {
    pub fn run() {
        oclive_monolith_builtin::event::invoke();
    }
}

mod slot_prompt {
    pub fn run() {
        oclive_monolith_builtin::prompt::invoke();
    }
}

mod slot_llm {
    pub fn run() {
        oclive_monolith_builtin::llm::invoke();
    }
}

mod slot_agent {
    pub fn run() {
        oclive_monolith_builtin::agent::invoke();
    }
}

mod slot_complex_emotion {
    pub fn run() {
        oclive_monolith_builtin::complex_emotion::invoke();
    }
}
