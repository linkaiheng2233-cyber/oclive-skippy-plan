//! 发行版运行器命令行：加载 → 校验 → 按事件序列演示表情判定。
//!
//! 本工具**不渲染**、不安装，只回答一个具体问题：
//! "给定这些设备事件，按这个发行版的反应表，角色应该显示什么表情？"
//!
//! 用法：
//!   ailive-gun-spirit-distro --dir distros/example-kit
//!   ailive-gun-spirit-distro --dir distros/example-kit \
//!       --events carrier.held_entered,control.primary.engaged --interval-ms 1000

use std::path::PathBuf;
use std::process::ExitCode;

use ailive_gun_spirit_host::distro::{kind_code_for_event, Distro};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut directory = PathBuf::from("distros/example-kit");
    let mut events: Vec<String> = Vec::new();
    let mut interval_ms: u64 = 1_000;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--dir" => {
                if let Some(value) = args.next() {
                    directory = PathBuf::from(value);
                }
            }
            "--events" => {
                if let Some(value) = args.next() {
                    events = value
                        .split(',')
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(str::to_owned)
                        .collect();
                }
            }
            "--interval-ms" => {
                if let Some(value) = args.next() {
                    interval_ms = value.parse().unwrap_or(1_000);
                }
            }
            "--help" | "-h" => {
                println!(
                    "usage: ailive-gun-spirit-distro [--dir <path>] [--events a,b,c] [--interval-ms n]"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let mut distro = match Distro::load(&directory) {
        Ok(distro) => distro,
        Err(error) => {
            eprintln!("distro: FAIL ({})", directory.display());
            eprintln!("  {error}");
            return ExitCode::FAILURE;
        }
    };

    println!("distro: {}", directory.display());
    println!(
        "  id={} name={} version={}",
        distro.id(),
        distro.name(),
        distro.version()
    );
    if let Some(author) = distro.author() {
        println!("  作者: {author}");
    }
    println!(
        "  需求固件: {}   登记文件 {} 个   总大小上限 {:?} B",
        distro.requires_firmware(),
        distro.file_count(),
        distro.max_total_bytes()
    );
    let (role_id, role_version) = distro.role_ref();
    println!(
        "  角色引用: {role_id} {role_version}   需求契约: {}",
        distro.requires_contract()
    );
    println!(
        "  表情 {} 个  最短驻留 {} ms  手动保持 {} ms",
        distro.expression_count(),
        distro.min_dwell_ms(),
        distro.manual_hold_ms()
    );
    println!(
        "  离线: 空闲={} 触摸={} 自动轮换={}",
        distro.offline_idle_expression(),
        distro.offline_touch_expression(),
        distro.offline_auto_rotate()
    );

    if events.is_empty() {
        println!("  提示: 用 --events 传入事件序列可查看表情判定");
        println!("distro: PASS");
        return ExitCode::SUCCESS;
    }

    println!("  事件序列（间隔 {interval_ms} ms）：");
    let mut now_ms: u64 = 0;
    let mut applied_count = 0usize;
    for name in &events {
        let Some(code) = kind_code_for_event(name) else {
            eprintln!("  {name}: ERR 未知事件（见协议事件表）");
            return ExitCode::FAILURE;
        };
        match distro.on_event(code, now_ms) {
            Some(applied) => {
                applied_count += 1;
                println!(
                    "    t={now_ms:>6} ms  {name:<26} -> {:<8} (priority {})",
                    applied.expression, applied.priority
                );
            }
            None => {
                println!("    t={now_ms:>6} ms  {name:<26} -> （被冷却/驻留抑制）");
            }
        }
        now_ms = now_ms.saturating_add(interval_ms);
    }
    println!(
        "  结果: 当前表情 {}（{} 条规则命中）",
        distro.current_expression(),
        applied_count
    );
    println!("distro: PASS");
    ExitCode::SUCCESS
}
