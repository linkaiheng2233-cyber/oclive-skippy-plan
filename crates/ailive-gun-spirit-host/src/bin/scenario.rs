//! 场景加载器命令行：解析 `.scn.json`、校验、打印摘要。
//!
//! 本工具**只做解析与校验**，不渲染、不跑玩法——渲染属于靶场（主机侧）。
//! 它存在的意义是让"地图可分享"这条链路先跑通：
//! 任何人拿到一个场景文件，都能立刻知道它是否合法、里面有什么。
//!
//! 用法：
//!   ailive-gun-spirit-scenario <path/to/file.scn.json>
//!   ailive-gun-spirit-scenario --quiet <path>      # 只输出 PASS/FAIL

use std::path::PathBuf;
use std::process::ExitCode;

use ailive_gun_spirit_host::scenario::Scenario;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut quiet = false;
    let mut path: Option<PathBuf> = None;
    for argument in args.by_ref() {
        match argument.as_str() {
            "--quiet" | "-q" => quiet = true,
            "--help" | "-h" => {
                println!("usage: ailive-gun-spirit-scenario [--quiet] <scenario.scn.json>");
                return ExitCode::SUCCESS;
            }
            other => path = Some(PathBuf::from(other)),
        }
    }
    let Some(path) = path else {
        eprintln!("usage: ailive-gun-spirit-scenario [--quiet] <scenario.scn.json>");
        return ExitCode::from(2);
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("scenario: cannot read {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
    };

    match Scenario::from_json(&text) {
        Ok(scenario) => {
            if quiet {
                println!("scenario: PASS ({})", path.display());
            } else {
                println!("scenario: {}", path.display());
                for line in scenario.summary_lines() {
                    println!("{line}");
                }
                println!("scenario: PASS");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("scenario: FAIL ({})", path.display());
            eprintln!("  {error}");
            ExitCode::FAILURE
        }
    }
}
