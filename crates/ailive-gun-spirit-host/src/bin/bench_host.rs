//! PC 台架程序（M2）：验证板 ↔ 电脑的帧协议会话。
//!
//! 两种模式：
//! - `--simulate`：不依赖硬件，用合成帧跑完整会话判据（握手、快照、事件去重、
//!   revision 倒退、坏帧、重复投递）。**这是 M2 的验收手段**。
//! - `--tcp <addr>`：连真实板子（M3 用），按帧解析并汇报统计。
//!
//! 退出码：0 = 全部判据通过；1 = 有判据失败或连接出错。

// 台架工具：夹具编码失败说明测试自身写错，用 expect 直接暴露最清晰。
#![allow(clippy::expect_used)]

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::ExitCode;

use ailive_gun_spirit_host::bench::{Rejection, Session};
use ailive_gun_spirit_protocol::event::WireEvent;
use ailive_gun_spirit_protocol::payload::{Expression, Hello};
use ailive_gun_spirit_protocol::state::{WireFact, WireNodeHealth, WireState};
use ailive_gun_spirit_protocol::{decode, encode, MessageType};

// 单一定义源：契约版本只从库里的常量取，避免二进制与库各维护一份而漂移。
use ailive_gun_spirit_host::bench::{
    EXPECTED_CONTRACT_MAJOR as CONTRACT_MAJOR, EXPECTED_CONTRACT_MINOR as CONTRACT_MINOR,
};
const BOARD_BOOT: &str = "board-boot-1";

fn frame(message_type: MessageType, payload_bytes: &[u8]) -> Vec<u8> {
    let mut buffer = vec![0u8; ailive_gun_spirit_protocol::MAX_FRAME];
    let written = encode(&mut buffer, message_type, 0, payload_bytes).expect("encode frame");
    buffer.truncate(written);
    buffer
}

fn hello_frame() -> Vec<u8> {
    let hello = Hello {
        contract_major: CONTRACT_MAJOR,
        contract_minor: CONTRACT_MINOR,
        boot_id: BOARD_BOOT,
        capabilities: 0b1_1111,
    };
    let mut payload_bytes = [0u8; 160];
    let written = hello.encode(&mut payload_bytes).expect("encode hello");
    frame(MessageType::Hello, &payload_bytes[..written])
}

fn state_frame(revision: u64) -> Vec<u8> {
    let known = |value: u8| WireFact::Known {
        value_code: value,
        observed_at_ms: revision * 10,
        fresh_until_ms: revision * 10 + 1_000,
        confidence_milli: 900,
    };
    let state = WireState {
        contract_major: CONTRACT_MAJOR,
        contract_minor: CONTRACT_MINOR,
        producer_boot_id: String::from(BOARD_BOOT),
        revision,
        produced_at_ms: revision * 10,
        carrier_state: known(1),
        front_grip: known(0),
        rear_grip: known(0),
        motion: WireFact::Unknown {
            reason_code: 2,
            since_revision: revision,
        },
        pose: known(0),
        primary_control: known(1),
        node_health: vec![WireNodeHealth {
            node_uid: String::from("node-front"),
            state_code: 0,
        }],
    };
    let mut payload_bytes = [0u8; 512];
    let written = state.encode(&mut payload_bytes).expect("encode state");
    frame(MessageType::State, &payload_bytes[..written])
}

fn event_frame(event_id: &str) -> Vec<u8> {
    let event = WireEvent {
        contract_major: CONTRACT_MAJOR,
        contract_minor: CONTRACT_MINOR,
        event_id: String::from(event_id),
        producer_boot_id: String::from(BOARD_BOOT),
        occurred_at_ms: 100,
        caused_by_revision: 1,
        kind_code: 0,
    };
    let mut payload_bytes = [0u8; 320];
    let written = event.encode(&mut payload_bytes).expect("encode event");
    frame(MessageType::Event, &payload_bytes[..written])
}

/// 把一帧的某一字节翻掉，模拟链路腐蚀（CRC 会抓到）。
fn corrupt(mut bytes: Vec<u8>) -> Vec<u8> {
    let index = bytes.len() / 2;
    bytes[index] ^= 0xFF;
    bytes
}

/// 根据头部声明计算整帧长度；magic 或长度非法时返回 `None`（调用方滑动一字节）。
fn header_frame_len(buffer: &[u8]) -> Option<usize> {
    use ailive_gun_spirit_protocol::{HEADER_LEN, MAGIC, MAX_PAYLOAD};
    if buffer.len() < HEADER_LEN || buffer[0..2] != MAGIC {
        return None;
    }
    let length = usize::from(u16::from_le_bytes([buffer[5], buffer[6]]));
    if length > MAX_PAYLOAD {
        return None;
    }
    Some(HEADER_LEN + length + 2)
}
/// 喂一段字节流给会话，按帧增量解析。
fn feed(session: &mut Session, buffer: &[u8]) -> usize {
    let mut offset = 0;
    let mut handled = 0;
    while offset < buffer.len() {
        match decode(&buffer[offset..]) {
            Ok((frame_view, consumed)) => {
                offset += consumed;
                handled += 1;
                let outcome = match frame_view.message_type {
                    MessageType::Hello => Hello::decode(frame_view.payload)
                        .map_err(Rejection::from)
                        .and_then(|hello| session.on_hello(&hello)),
                    MessageType::State => WireState::decode(frame_view.payload)
                        .map_err(Rejection::from)
                        .and_then(|state| session.on_state(state)),
                    MessageType::Event => WireEvent::decode(frame_view.payload)
                        .map_err(Rejection::from)
                        .and_then(|event| {
                            let result = session.on_event(&event).map(|_| ());
                            if matches!(result, Ok(())) {
                                println!("  event kind={} id={}", event.kind_code, event.event_id);
                            }
                            result
                        }),
                    _ => Ok(()),
                };
                if let Err(reason) = outcome {
                    println!("  reject: {reason}");
                }
            }
            Err(error) => {
                use ailive_gun_spirit_protocol::FrameError;
                match error {
                    // 坏 magic：逐字节滑动重同步，不计入坏帧统计。
                    FrameError::BadMagic => offset += 1,
                    // 数据不足：等更多字节。
                    FrameError::NeedMoreData => break,
                    // 其余错误：按头部声明的长度跳过整帧，避免把同一坏帧反复计数。
                    other => {
                        session.note_malformed();
                        println!("  frame-error: {other}");
                        let skip = header_frame_len(&buffer[offset..]).unwrap_or(1);
                        offset += skip;
                    }
                }
            }
        }
    }
    handled
}

fn run_simulation() -> bool {
    println!("bench: simulate");
    let mut session = Session::new();

    let mut stream = Vec::new();
    stream.extend(hello_frame());
    stream.extend(state_frame(1));
    stream.extend(state_frame(2));
    stream.extend(event_frame("evt-1"));
    stream.extend(event_frame("evt-1")); // 链路重传：必须被去重
    stream.extend(event_frame("evt-2"));
    stream.extend(state_frame(1)); // revision 倒退：必须被拒
    stream.extend(corrupt(event_frame("evt-3"))); // 坏 CRC：丢弃且不崩

    let handled = feed(&mut session, &stream);
    let stats = session.stats().clone();
    println!(
        "  frames={handled} hello={} states={} states_rejected={} events_new={} events_dup={} malformed={}",
        stats.hellos, stats.states, stats.states_rejected, stats.events_new, stats.events_duplicate,
        stats.events_rejected
    );

    let mut ok = true;
    ok &= stats.hellos == 1;
    ok &= stats.states == 2;
    ok &= stats.states_rejected == 1;
    ok &= stats.events_new == 2;
    ok &= stats.events_duplicate == 1;
    ok &= stats.events_rejected == 1;
    ok &= session.snapshot().map(|state| state.revision) == Some(2);

    if ok {
        println!("bench: PASS");
    } else {
        println!("bench: FAIL");
    }
    ok
}

fn run_tcp(address: &str, seconds: u64) -> bool {
    println!("bench: tcp {address}");
    let mut stream = match TcpStream::connect(address) {
        Ok(stream) => stream,
        Err(error) => {
            println!("  connect failed: {error}");
            return false;
        }
    };
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(500)));
    let mut session = Session::new();
    let hello = hello_frame();
    if let Err(error) = stream.write_all(&hello) {
        println!("  write hello failed: {error}");
        return false;
    }

    let expression = Expression {
        expression: "happy",
    };
    let mut expr_payload = [0u8; 64];
    let written = expression.encode(&mut expr_payload).expect("encode expr");
    let expr = frame(MessageType::Expression, &expr_payload[..written]);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let mut buffer = [0u8; 4096];
    let mut pending: Vec<u8> = Vec::new();
    // 只在首次收到状态后发一次表情：否则"收帧→发表情→再收帧"会形成无限往返。
    let mut expression_sent = false;
    while std::time::Instant::now() < deadline {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                pending.extend_from_slice(&buffer[..count]);
                let handled = feed(&mut session, &pending);
                if handled > 0 {
                    pending.clear();
                    if !expression_sent && session.snapshot().is_some() {
                        expression_sent = true;
                        let _ = stream.write_all(&expr);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(error) => {
                println!("  read failed: {error}");
                return false;
            }
        }
    }
    let stats = session.stats();
    println!(
        "  hello={} states={} events_new={} events_dup={}",
        stats.hellos, stats.states, stats.events_new, stats.events_duplicate
    );
    if let Some(snapshot) = session.snapshot() {
        println!(
            "  snapshot revision={} carrier={:?} front={:?} rear={:?} motion={:?} pose={:?} control={:?} nodes={}",
            snapshot.revision,
            snapshot.carrier_state,
            snapshot.front_grip,
            snapshot.rear_grip,
            snapshot.motion,
            snapshot.pose,
            snapshot.primary_control,
            snapshot.node_health.len()
        );
    } else {
        println!("  snapshot: none");
    }
    stats.hellos >= 1 && stats.states >= 1
}

/// 模拟设备：监听端口，按协议回应 HELLO/STATE/EVENT/EXPR。
///
/// 存在的意义：**在没有板子的情况下验证整条 TCP 链路**（M2 的收尾），
/// 以及给 M3 上板联调提供一个已知正确的对端做对照。
fn run_serve(listener_address: &str, seconds: u64) -> bool {
    use std::net::TcpListener;
    println!("bench: serve {listener_address}");
    let listener = match TcpListener::bind(listener_address) {
        Ok(listener) => listener,
        Err(error) => {
            println!("  bind failed: {error}");
            return false;
        }
    };
    let _ = listener.set_nonblocking(true);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let mut served = 0u32;
    while std::time::Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                served += 1;
                println!("  client connected: {peer}");
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(500)));
                let mut buffer = [0u8; 4096];
                let mut pending: Vec<u8> = Vec::new();
                let mut revision = 0u64;
                let session_deadline =
                    std::time::Instant::now() + std::time::Duration::from_secs(5);
                while std::time::Instant::now() < session_deadline {
                    match stream.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(count) => {
                            pending.extend_from_slice(&buffer[..count]);
                            let mut offset = 0;
                            while offset < pending.len() {
                                match decode(&pending[offset..]) {
                                    Ok((frame_view, consumed)) => {
                                        offset += consumed;
                                        match frame_view.message_type {
                                            MessageType::Hello => {
                                                let _ = stream.write_all(&hello_frame());
                                                revision += 1;
                                                let _ = stream.write_all(&state_frame(revision));
                                                let _ = stream.write_all(&event_frame("evt-sim-1"));
                                            }
                                            MessageType::Expression => {
                                                let mut ack = [0u8; 4];
                                                ack.copy_from_slice(&0u32.to_le_bytes());
                                                let frame_bytes = frame(MessageType::Ack, &ack);
                                                let _ = stream.write_all(&frame_bytes);
                                                revision += 1;
                                                let _ = stream.write_all(&state_frame(revision));
                                            }
                                            _ => {}
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                            pending.drain(..offset);
                        }
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                || error.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(_) => break,
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(error) => {
                println!("  accept failed: {error}");
                return false;
            }
        }
    }
    println!("  served {served} connection(s)");
    served > 0
}
/// UDP 广播发现：向 `255.255.255.255:47901` 发 `GS-DISCOVER`，
/// 等板子回 `GS-HERE <ip>:<port>`，再用返回地址建立 TCP 会话。
fn run_discover(seconds: u64, target: Option<&str>) -> bool {
    use std::net::UdpSocket;
    println!("bench: discover");
    // 绑定固定端口接收板子的主动广播（板子在自己的子网里广播，必然到得了）。
    // 若端口被占用则退回临时端口 + 主动请求。
    let socket = match UdpSocket::bind("0.0.0.0:47901") {
        Ok(socket) => socket,
        Err(_) => match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => socket,
            Err(error) => {
                println!("  bind failed: {error}");
                return false;
            }
        },
    };
    if let Err(error) = socket.set_broadcast(true) {
        println!("  set_broadcast failed: {error}");
        return false;
    }
    let _ = socket.set_read_timeout(Some(std::time::Duration::from_millis(800)));
    // 未指定目标时向广播地址请求；指定了就用单播（跨网卡场景的兜底）。
    let request_target = match target {
        Some(address) => format!("{address}:47901"),
        None => String::from("255.255.255.255:47901"),
    };
    println!("  request target: {request_target}");
    if let Err(error) = socket.send_to(b"GS-DISCOVER", &request_target) {
        println!("  broadcast failed: {error}");
        return false;
    }
    let mut buffer = [0u8; 256];
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    while std::time::Instant::now() < deadline {
        match socket.recv_from(&mut buffer) {
            Ok((count, peer)) => {
                let text = String::from_utf8_lossy(&buffer[..count]).trim().to_owned();
                println!("  reply from {peer}: {text}");
                if let Some(rest) = text.strip_prefix("GS-HERE ") {
                    return run_tcp(rest.trim(), 8);
                }
                // 不是应答就再请求一次，继续等。
                let _ = socket.send_to(b"GS-DISCOVER", &request_target);
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                let _ = socket.send_to(b"GS-DISCOVER", &request_target);
            }
            Err(error) => {
                println!("  recv failed: {error}");
                return false;
            }
        }
    }
    println!("  no device answered");
    false
}
fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let ok = match args.first().map(String::as_str) {
        Some("--simulate") | None => run_simulation(),
        Some("--tcp") => {
            let address = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:47900".to_owned());
            let seconds = args
                .get(2)
                .and_then(|value| value.parse().ok())
                .unwrap_or(10);
            run_tcp(&address, seconds)
        }
        Some("--discover") => {
            let seconds = args
                .get(1)
                .and_then(|value| value.parse().ok())
                .unwrap_or(10);
            let target = args.get(2).map(String::as_str);
            run_discover(seconds, target)
        }
        Some("--serve") => {
            let address = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:47900".to_owned());
            let seconds = args
                .get(2)
                .and_then(|value| value.parse().ok())
                .unwrap_or(10);
            run_serve(&address, seconds)
        }
        Some(other) => {
            eprintln!("unknown mode: {other}");
            eprintln!("usage: ailive-gun-spirit-bench [--simulate | --discover [s] | --serve <addr> [s] | --tcp <addr> [s]]");
            return ExitCode::from(2);
        }
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
