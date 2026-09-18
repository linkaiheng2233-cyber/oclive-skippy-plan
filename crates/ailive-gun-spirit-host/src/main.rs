//! A.I.Live ai枪娘器灵的纯无头集成宿主。
//!
//! 当前入口继续复用OCLive headless HTTP API；BLE、感知核心和renderer将在该crate的
//! composition root中接线，而不会反向进入OCLive或感知核心。

fn kernel_bench_iterations() -> u32 {
    std::env::var("OCLIVE_KERNEL_BENCH_ITERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|&value| value > 0)
        .unwrap_or(0)
        .min(1_000_000)
}

fn main() {
    let _log_guard = oclive_kernel_host::init_tracing();
    let bench_iterations = kernel_bench_iterations();
    if bench_iterations > 0 {
        for _ in 0..bench_iterations {
            tracing::info!(
                target: "oclive_kernel_server",
                version = oclive_kernel_runtime::RUNTIME_API_VERSION,
                "ailive-gun-spirit host kernel-linked bench smoke"
            );
        }
        return;
    }

    let args: Vec<String> = std::env::args().collect();
    if args
        .iter()
        .skip(1)
        .any(|argument| argument == "-h" || argument == "--help")
    {
        eprintln!(
            "Usage: {} [--api] [--port PORT]\nEnv: OCLIVE_API_PORT, OCLIVE_HTTP_API_MOCK_LLM, OCLIVE_ROLES_DIR, RUST_LOG",
            args[0]
        );
        return;
    }

    let cli_port = oclive_kernel_runtime::parse_api_port_arg(&args).unwrap_or_else(|error| {
        eprintln!("[OCLIVE_CLI_INVALID_ARGUMENT] {error}");
        std::process::exit(2);
    });
    let port = oclive_kernel_runtime::resolve_api_port(cli_port);
    tracing::info!(
        target: "oclive_kernel_server",
        version = oclive_kernel_runtime::RUNTIME_API_VERSION,
        port,
        "starting A.I.Live ai枪娘器灵 headless host"
    );
    oclive_kernel_host::run_api_server(port);
}
