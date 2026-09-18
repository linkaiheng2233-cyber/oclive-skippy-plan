#![allow(dead_code)]
// oclive-cli生成的Monolith七焊接键顺序烟测；正式宿主仍走OCLive headless API。

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
