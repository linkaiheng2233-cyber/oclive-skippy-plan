//! Monolith smoke入口；与标准Host入口分离。

mod process_message_monolith;

fn kernel_bench_iterations() -> u32 {
    std::env::var("OCLIVE_KERNEL_BENCH_ITERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|&value| value > 0)
        .unwrap_or(1)
        .min(1_000_000)
}

fn init_kernel_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn main() {
    init_kernel_tracing();
    let iterations = kernel_bench_iterations();
    tracing::info!("ailive-gun-spirit-host — monolith smoke build");
    for _ in 0..iterations {
        process_message_monolith::run_monolith_pipeline_demo();
    }
}
