//! 感知内核 → ESP32-S3 固件的链接垫片。
//!
//! 提供三样 no_std 静态库**必须由最终二进制**提供的东西：
//! 1. `#[panic_handler]`：板上没有 unwinding，直接停机并把控制权交给复位逻辑；
//! 2. `#[global_allocator]`：包一层 newlib 的 `malloc/free`，与 C 侧共用同一个堆；
//! 3. C ABI：固件侧只需要这些 `extern "C"` 符号即可驱动内核。
//!
//! 本 crate 刻意位于 workspace 之外：工作区 `forbid(unsafe_code)`，而
//! `unsafe impl GlobalAlloc`、`extern "C"` 与全局 reducer 的可变静态在这里不可避免。
//!
//! **线程模型**：全局 reducer 只在固件主循环里被调用（单线程），因此用 `static mut`
//! 加访问函数即可；如果将来引入多任务，必须改为互斥保护。

#![no_std]

extern crate alloc;

use core::cell::UnsafeCell;
use core::fmt::Write as _;

use alloc::string::String;
use ailive_gun_spirit_contracts::{
    BootId, Capability, CarrierState, Contact, ContractVersion, Fact, GripBinding, MotionClass,
    NodeUid, ObservationId, PerceptionState, Permille, Pose, SensorObservation,
    SensorObservationKind, UnknownReason,
};
use ailive_gun_spirit_perception_core::{
    CapabilitySet, ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    PerceptionReducer,
};
use ailive_gun_spirit_protocol::state::{
    WireFact, WireNodeHealth, WireState, CARRIER_HELD, CARRIER_READY, CARRIER_STANDBY, CONTACT_ENGAGED,
    CONTACT_RELEASED, HEALTH_DEGRADED, HEALTH_HEALTHY, HEALTH_INCOMPATIBLE_PROTOCOL, HEALTH_OFFLINE,
    HEALTH_PROTOCOL_FAULT, MOTION_IDLE, MOTION_MOVING, POSE_LOWERED, POSE_RAISED,
    REASON_NEVER_OBSERVED, REASON_NODE_OFFLINE, REASON_SENSOR_FAULT, REASON_STALE,
    REASON_UNCALIBRATED,
};

/// ABI 版本：固件侧用它确认头文件与库一致。
#[no_mangle]
pub static GS_KERNEL_ABI_VERSION: u32 = 2;

const NODE_UID: &str = "gun-node";
const HOST_BOOT: &str = "board-kernel-1";
const NODE_BOOT: &str = "gun-node-boot-1";

// ---------------------------------------------------------------------------
// panic handler
// ---------------------------------------------------------------------------

/// 板上 panic：不展开、不打日志，直接停住让看门狗复位。
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// ---------------------------------------------------------------------------
// 全局分配器（复用 newlib 的堆）
// ---------------------------------------------------------------------------

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(pointer: *mut u8);
}

struct NewlibAllocator;

// SAFETY: 仅转发到 C 运行时的 malloc/free。
unsafe impl core::alloc::GlobalAlloc for NewlibAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { malloc(layout.size()) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, _layout: core::alloc::Layout) {
        unsafe { free(pointer) }
    }
}

#[global_allocator]
static ALLOCATOR: NewlibAllocator = NewlibAllocator;

// ---------------------------------------------------------------------------
// 内核实例
// ---------------------------------------------------------------------------

struct KernelState {
    reducer: Option<PerceptionReducer>,
    sequence: u64,
}

struct KernelCell(UnsafeCell<KernelState>);

// SAFETY: 只在固件主循环（单线程）访问；见模块文档的线程模型说明。
unsafe impl Sync for KernelCell {}

static KERNEL: KernelCell = KernelCell(UnsafeCell::new(KernelState {
    reducer: None,
    sequence: 0,
}));

fn kernel() -> &'static mut KernelState {
    // SAFETY: 单线程访问，且调用期间不会重入。
    unsafe { &mut *KERNEL.0.get() }
}

fn permille(value: u16) -> Permille {
    Permille::new(value).unwrap_or(if let Ok(zero) = Permille::new(0) {
        zero
    } else {
        // Permille(0) 恒合法；此分支不可达，仅为避免 unwrap。
        panic!("permille zero rejected")
    })
}

fn build_reducer() -> Option<PerceptionReducer> {
    let node = NodeUid::try_new(NODE_UID).ok()?;
    let freshness = FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 3_500, 200).ok()?;
    let orientation =
        OrientationCalibration::try_new([1_000, 0, 0], [0, 0, 1_000], permille(500)).ok()?;
    let confidence = ConfidenceProfile::try_new(permille(900), permille(700), permille(900)).ok()?;
    let capabilities = CapabilitySet::try_new([
        Capability::FrontGrip,
        Capability::RearGrip,
        Capability::Motion,
        Capability::Orientation,
        Capability::PrimaryControl,
        Capability::ShockDiagnostic,
    ])
    .ok()?;
    let profile =
        PerceptionProfile::try_new([(node, capabilities)], freshness, Some(orientation), confidence)
            .ok()?;
    let host_boot = BootId::try_new(HOST_BOOT).ok()?;
    Some(PerceptionReducer::new(host_boot, profile, 0))
}

fn next_observation(kind: SensorObservationKind, millis: u64) -> Option<SensorObservation> {
    let state = kernel();
    state.sequence = state.sequence.checked_add(1)?;
    Some(SensorObservation {
        contract_version: ContractVersion::V0_3,
        observation_id: ObservationId::try_new(format_args_string(
            format_args!("{NODE_UID}:{}", state.sequence),
        ))
        .ok()?,
        node_uid: NodeUid::try_new(NODE_UID).ok()?,
        boot_id: BootId::try_new(NODE_BOOT).ok()?,
        sequence: state.sequence,
        node_monotonic_ms: millis,
        observation: kind,
    })
}

/// 避免依赖 `format!` 宏的形式：用 `core::fmt::Write` 拼字符串。
fn format_args_string(args: core::fmt::Arguments<'_>) -> String {
    let mut output = String::new();
    let _ = output.write_fmt(args);
    output
}

fn ingest(kind: SensorObservationKind, millis: u64) -> i32 {
    let state = kernel();
    let Some(reducer) = state.reducer.as_mut() else {
        return -1;
    };
    let Some(observation) = next_observation(kind, millis) else {
        return -2;
    };
    match reducer.ingest(observation, millis) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

// ---------------------------------------------------------------------------
// C ABI
// ---------------------------------------------------------------------------

/// ABI 版本（函数形式，便于 C 侧直接调用）。
#[no_mangle]
pub extern "C" fn gs_kernel_abi_version() -> u32 {
    GS_KERNEL_ABI_VERSION
}

/// 初始化内核实例。成功返回 0。
#[no_mangle]
pub extern "C" fn gs_kernel_init() -> i32 {
    let Some(reducer) = build_reducer() else {
        return -1;
    };
    let state = kernel();
    state.reducer = Some(reducer);
    state.sequence = 0;
    0
}

/// 是否已初始化。
#[no_mangle]
pub extern "C" fn gs_kernel_ready() -> i32 {
    if kernel().reducer.is_some() {
        1
    } else {
        0
    }
}

/// 摄入握持观测。`grip`: 0=前握, 1=后握；`contact`: 0=接触, 1=释放。
#[no_mangle]
pub extern "C" fn gs_kernel_ingest_grip(
    grip: u8,
    contact: u8,
    strength_milli: u16,
    millis: u64,
) -> i32 {
    let grip = match grip {
        0 => GripBinding::Front,
        1 => GripBinding::Rear,
        _ => return -10,
    };
    let contact = match contact {
        0 => Contact::Engaged,
        1 => Contact::Released,
        _ => return -11,
    };
    ingest(
        SensorObservationKind::GripContact {
            grip,
            contact,
            strength_milli: permille(strength_milli),
        },
        millis,
    )
}

/// 摄入运动分类。`motion`: 0=静止, 1=移动。
#[no_mangle]
pub extern "C" fn gs_kernel_ingest_motion(motion: u8, millis: u64) -> i32 {
    let motion = match motion {
        0 => MotionClass::Idle,
        1 => MotionClass::Moving,
        _ => return -10,
    };
    ingest(SensorObservationKind::MotionClass { motion }, millis)
}

/// 摄入姿态估计（毫重力 + 质量）。
#[no_mangle]
pub extern "C" fn gs_kernel_ingest_orientation(
    gravity_x_mg: i32,
    gravity_y_mg: i32,
    gravity_z_mg: i32,
    quality_milli: u16,
    millis: u64,
) -> i32 {
    ingest(
        SensorObservationKind::OrientationEstimate {
            gravity_mg_x: gravity_x_mg,
            gravity_mg_y: gravity_y_mg,
            gravity_mg_z: gravity_z_mg,
            quality_milli: permille(quality_milli),
        },
        millis,
    )
}

/// 摄入主控触点。`contact`: 0=接触, 1=释放。
#[no_mangle]
pub extern "C" fn gs_kernel_ingest_control(contact: u8, millis: u64) -> i32 {
    let contact = match contact {
        0 => Contact::Engaged,
        1 => Contact::Released,
        _ => return -10,
    };
    ingest(
        SensorObservationKind::AuxControlContact {
            control_id: ailive_gun_spirit_contracts::PrimaryControlId::PrimaryTrigger,
            contact,
        },
        millis,
    )
}

/// 让内核按当前时间推进（触发 TTL 过期与载体状态重算）。
#[no_mangle]
pub extern "C" fn gs_kernel_tick(millis: u64) -> i32 {
    let state = kernel();
    let Some(reducer) = state.reducer.as_mut() else {
        return -1;
    };
    match reducer.tick(millis) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

/// 当前内核 revision（未初始化返回 0）。
#[no_mangle]
pub extern "C" fn gs_kernel_state_revision() -> u64 {
    kernel()
        .reducer
        .as_ref()
        .map(|reducer| reducer.current_state().revision)
        .unwrap_or(0)
}

/// 把内核当前快照编码成协议 `STATE` 载荷，写入调用方缓冲区。
/// 返回写入字节数；负数表示错误。
#[no_mangle]
pub extern "C" fn gs_kernel_encode_state(output: *mut u8, capacity: usize) -> isize {
    if output.is_null() || capacity == 0 {
        return -1;
    }
    let state = kernel();
    let Some(reducer) = state.reducer.as_ref() else {
        return -2;
    };
    let Some(wire) = to_wire_state(reducer.current_state()) else {
        return -3;
    };
    // SAFETY: 调用方保证缓冲区有效且长度为 capacity。
    let buffer = unsafe { core::slice::from_raw_parts_mut(output, capacity) };
    match wire.encode(buffer) {
        Ok(written) => isize::try_from(written).unwrap_or(-4),
        Err(_) => -4,
    }
}

fn contact_code(contact: Contact) -> u8 {
    match contact {
        Contact::Engaged => CONTACT_ENGAGED,
        Contact::Released => CONTACT_RELEASED,
    }
}

fn motion_code(motion: MotionClass) -> u8 {
    match motion {
        MotionClass::Idle => MOTION_IDLE,
        MotionClass::Moving => MOTION_MOVING,
    }
}

fn pose_code(pose: Pose) -> u8 {
    match pose {
        Pose::Raised => POSE_RAISED,
        Pose::Lowered => POSE_LOWERED,
    }
}

fn carrier_code(carrier: CarrierState) -> u8 {
    match carrier {
        CarrierState::Standby => CARRIER_STANDBY,
        CarrierState::Held => CARRIER_HELD,
        CarrierState::Ready => CARRIER_READY,
    }
}

fn reason_code(reason: UnknownReason) -> u8 {
    match reason {
        UnknownReason::NeverObserved => REASON_NEVER_OBSERVED,
        UnknownReason::NodeOffline => REASON_NODE_OFFLINE,
        UnknownReason::Stale => REASON_STALE,
        UnknownReason::Uncalibrated => REASON_UNCALIBRATED,
        UnknownReason::SensorFault => REASON_SENSOR_FAULT,
    }
}

/// 把契约 `Fact<T>` 映射到线格式；`source_refs` 按设计**不进入线格式**。
fn wire_fact<T, F>(fact: &Fact<T>, value_code: F) -> WireFact
where
    T: Copy,
    F: Fn(T) -> u8,
{
    match fact {
        Fact::Known {
            value,
            observed_at_monotonic_ms,
            fresh_until_monotonic_ms,
            confidence_milli,
            ..
        } => WireFact::Known {
            value_code: value_code(*value),
            observed_at_ms: *observed_at_monotonic_ms,
            fresh_until_ms: *fresh_until_monotonic_ms,
            confidence_milli: confidence_milli.get(),
        },
        Fact::Unknown {
            reason,
            since_revision,
        } => WireFact::Unknown {
            reason_code: reason_code(*reason),
            since_revision: *since_revision,
        },
    }
}

fn health_code(state: ailive_gun_spirit_contracts::NodeHealthState) -> u8 {
    use ailive_gun_spirit_contracts::NodeHealthState as State;
    match state {
        State::Healthy => HEALTH_HEALTHY,
        State::Degraded => HEALTH_DEGRADED,
        State::Offline => HEALTH_OFFLINE,
        State::IncompatibleProtocol => HEALTH_INCOMPATIBLE_PROTOCOL,
        State::ProtocolFault => HEALTH_PROTOCOL_FAULT,
    }
}

fn to_wire_state(state: &PerceptionState) -> Option<WireState> {
    let mut node_health = alloc::vec::Vec::with_capacity(state.node_health.len());
    for entry in &state.node_health {
        node_health.push(WireNodeHealth {
            node_uid: String::from(entry.node_uid.as_str()),
            state_code: health_code(entry.state),
        });
    }
    Some(WireState {
        contract_major: 0,
        contract_minor: 3,
        host_boot_id: String::from(state.host_boot_id.as_str()),
        revision: state.revision,
        produced_at_ms: state.produced_at_monotonic_ms,
        carrier_state: wire_fact(&state.carrier_state, carrier_code),
        front_grip: wire_fact(&state.front_grip, contact_code),
        rear_grip: wire_fact(&state.rear_grip, contact_code),
        motion: wire_fact(&state.motion, motion_code),
        pose: wire_fact(&state.pose, pose_code),
        primary_control: wire_fact(&state.primary_control, contact_code),
        node_health,
    })
}

/// 自检：跑一遍"前握接触 → 后握接触"，返回推进后的 revision；失败返回负数。
#[no_mangle]
pub extern "C" fn gs_kernel_self_test() -> i32 {
    if gs_kernel_init() != 0 {
        return -1;
    }
    if gs_kernel_ingest_grip(0, 0, 700, 0) != 0 {
        return -2;
    }
    if gs_kernel_ingest_grip(1, 0, 650, 10) != 0 {
        return -3;
    }
    i32::try_from(gs_kernel_state_revision()).unwrap_or(-4)
}

