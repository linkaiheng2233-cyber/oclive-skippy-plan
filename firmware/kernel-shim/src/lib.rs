//! 感知内核 → ESP32-S3 固件的链接垫片（V1 链接层验证）。
//!
//! 提供三样 no_std 静态库**必须由最终二进制**提供的东西：
//! 1. `#[panic_handler]`：板上没有 unwinding，直接停机并把控制权交给复位逻辑；
//! 2. `#[global_allocator]`：包一层 newlib 的 `malloc/free`，与 C 侧共用同一个堆；
//! 3. C ABI：固件侧只需要这几个 `extern "C"` 符号即可驱动内核。
//!
//! 本 crate 刻意位于 workspace 之外：工作区 `forbid(unsafe_code)`，而
//! `unsafe impl GlobalAlloc` 与 `extern "C"` 声明在这里不可避免。

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use ailive_gun_spirit_contracts::{
    BootId, Capability, Contact, ContractVersion, GripBinding, NodeUid, ObservationId, Permille,
    SensorObservation, SensorObservationKind,
};
use ailive_gun_spirit_perception_core::{
    CapabilitySet, ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    PerceptionReducer,
};

// ---------------------------------------------------------------------------
// 1. panic handler
// ---------------------------------------------------------------------------

/// 板上 panic：不展开、不打日志，直接停住让看门狗复位。
///
/// 真机上应改成写 coredump 或触发复位寄存器；此处保持最小可链接实现。
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// ---------------------------------------------------------------------------
// 2. 全局分配器（复用 newlib 的堆）
// ---------------------------------------------------------------------------

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(pointer: *mut u8);
}

/// 把 Rust 的分配直接转给 C 运行时的 `malloc/free`，避免出现两个独立堆。
struct NewlibAllocator;

// SAFETY: 仅转发到 C 运行时的 malloc/free；不改变布局，不做对齐重解释。
unsafe impl core::alloc::GlobalAlloc for NewlibAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // newlib 的 malloc 保证 8 字节对齐，满足 Rust 默认要求。
        unsafe { malloc(layout.size()) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, _layout: core::alloc::Layout) {
        unsafe { free(pointer) }
    }
}

#[global_allocator]
static ALLOCATOR: NewlibAllocator = NewlibAllocator;

// ---------------------------------------------------------------------------
// 3. C ABI
// ---------------------------------------------------------------------------

/// ABI 版本：固件侧用它确认头文件与库一致。
#[no_mangle]
pub extern "C" fn gs_kernel_abi_version() -> u32 {
    1
}

fn permille(value: u16) -> Option<Permille> {
    Permille::new(value).ok()
}

/// 构造一个"单节点全能力"的 profile 与 reducer（P0-A 单板形态）。
fn build_reducer() -> Option<Box<PerceptionReducer>> {
    let node = NodeUid::try_new("gun-node").ok()?;
    let freshness = FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 3_500, 200).ok()?;
    let orientation = OrientationCalibration::try_new([1_000, 0, 0], [0, 0, 1_000], permille(500)?).ok()?;
    let confidence = ConfidenceProfile::try_new(permille(900)?, permille(700)?, permille(900)?).ok()?;
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
    let host_boot = BootId::try_new("shim-host-boot-1").ok()?;
    Some(Box::new(PerceptionReducer::new(host_boot, profile, 0)))
}

fn observation(sequence: u64, millis: u64, kind: SensorObservationKind) -> Option<SensorObservation> {
    Some(SensorObservation {
        contract_version: ContractVersion::V0_3,
        observation_id: ObservationId::try_new(alloc::format!("gun-node:{sequence}")).ok()?,
        node_uid: NodeUid::try_new("gun-node").ok()?,
        boot_id: BootId::try_new("gun-node-boot-1").ok()?,
        sequence,
        node_monotonic_ms: millis,
        observation: kind,
    })
}

/// 自检：跑一遍"前握接触 → 后握接触"，成功返回 0。
///
/// 这个函数同时是**链接验证的锚点**：只要它能被 C 侧调用并链接成功，
/// 就说明内核、分配器、panic handler 在真目标上是一条完整链路。
#[no_mangle]
pub extern "C" fn gs_kernel_self_test() -> i32 {
    let Some(mut reducer) = build_reducer() else {
        return -1;
    };
    let Some(front) = observation(
        1,
        0,
        SensorObservationKind::GripContact {
            grip: GripBinding::Front,
            contact: Contact::Engaged,
            strength_milli: Permille::new(700).unwrap_or(Permille::new(0).unwrap_or_else(|_| unreachable!())),
        },
    ) else {
        return -2;
    };
    let Some(rear) = observation(
        2,
        10,
        SensorObservationKind::GripContact {
            grip: GripBinding::Rear,
            contact: Contact::Engaged,
            strength_milli: Permille::new(650).unwrap_or(Permille::new(0).unwrap_or_else(|_| unreachable!())),
        },
    ) else {
        return -3;
    };
    if reducer.ingest(front, 0).is_err() {
        return -4;
    }
    if reducer.ingest(rear, 10).is_err() {
        return -5;
    }
    i32::try_from(reducer.current_state().revision).unwrap_or(-6)
}

/// 返回当前内核状态的结构化摘要（供固件侧 ABI 冒烟）。
#[no_mangle]
pub extern "C" fn gs_kernel_state_revision() -> u64 {
    match build_reducer() {
        Some(reducer) => reducer.current_state().revision,
        None => 0,
    }
}
