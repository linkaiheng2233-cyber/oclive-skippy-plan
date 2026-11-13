//! 感知内核堆占用探针（CP1b）。
//!
//! 目的：为板上预算给出实测依据——双射频开启后内部堆只剩 ≈60 KB，
//! 其中内核堆建议 ≤20 KB。本探针用计数分配器统计"内核代码路径实际申请了多少字节"。
//!
//! 它不是性能基准：只统计分配量，不统计 CPU。跨平台成立，因为分配行为与指令集无关，
//! 所以可以在主机上测，再与 Xtensa 目标共用同一结论。

// 工作区默认 deny unsafe_code；本文件是**纯测量用测试二进制**，实现计数分配器必须写 unsafe。
// 该 unsafe 只转发系统分配器并做原子加减，不涉及裸指针运算，也不进入任何产品代码。

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use ailive_gun_spirit_contracts::{
    BootId, Capability, Contact, ContractVersion, GripBinding, MotionClass, NodeUid, ObservationId,
    Permille, PrimaryControlId, SensorObservation, SensorObservationKind,
};
use ailive_gun_spirit_perception_core::{
    CapabilitySet, ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    PerceptionReducer,
};

struct Counting;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ACTIVE: AtomicBool = AtomicBool::new(false);

// SAFETY: 转发到系统分配器，只在计数上增加原子操作。
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && ACTIVE.load(Ordering::Relaxed) {
            let now = CURRENT.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        if ACTIVE.load(Ordering::Relaxed) {
            let _ = CURRENT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                Some(value.saturating_sub(layout.size()))
            });
        }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn permille(value: u16) -> Permille {
    Permille::new(value).expect("permille in range")
}

fn new_reducer() -> PerceptionReducer {
    let node = NodeUid::try_new("probe-node").expect("node uid");
    let freshness = FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 3_500, 200)
        .expect("freshness profile");
    let orientation = OrientationCalibration::try_new([1_000, 0, 0], [0, 0, 1_000], permille(500))
        .expect("orientation calibration");
    let confidence = ConfidenceProfile::try_new(permille(900), permille(700), permille(900))
        .expect("confidence profile");
    let capabilities = CapabilitySet::try_new([
        Capability::FrontGrip,
        Capability::RearGrip,
        Capability::Motion,
        Capability::Orientation,
        Capability::PrimaryControl,
        Capability::ShockDiagnostic,
    ])
    .expect("capability set");
    let profile = PerceptionProfile::try_new(
        [(node.clone(), capabilities)],
        freshness,
        Some(orientation),
        confidence,
    )
    .expect("profile");
    PerceptionReducer::new(
        BootId::try_new("probe-host-boot-1").expect("host boot id"),
        profile,
        0,
    )
}

fn observations() -> Vec<SensorObservation> {
    let node = NodeUid::try_new("probe-node").expect("node uid");
    let boot = BootId::try_new("probe-node-boot-1").expect("boot id");
    let mut sequence = 0u64;
    let mut build = |millis: u64, kind: SensorObservationKind| {
        sequence += 1;
        SensorObservation {
            contract_version: ContractVersion::V0_3,
            observation_id: ObservationId::try_new(format!("probe:{sequence}")).expect("obs id"),
            node_uid: node.clone(),
            boot_id: boot.clone(),
            sequence,
            node_monotonic_ms: millis,
            observation: kind,
        }
    };
    vec![
        build(
            0,
            SensorObservationKind::GripContact {
                grip: GripBinding::Front,
                contact: Contact::Engaged,
                strength_milli: permille(700),
            },
        ),
        build(
            10,
            SensorObservationKind::GripContact {
                grip: GripBinding::Rear,
                contact: Contact::Engaged,
                strength_milli: permille(650),
            },
        ),
        build(20, SensorObservationKind::MotionClass { motion: MotionClass::Idle }),
        build(
            30,
            SensorObservationKind::OrientationEstimate {
                gravity_mg_x: 1_000,
                gravity_mg_y: 0,
                gravity_mg_z: 0,
                quality_milli: permille(900),
            },
        ),
        build(
            40,
            SensorObservationKind::AuxControlContact {
                control_id: PrimaryControlId::PrimaryTrigger,
                contact: Contact::Engaged,
            },
        ),
        build(
            50,
            SensorObservationKind::AuxControlContact {
                control_id: PrimaryControlId::PrimaryTrigger,
                contact: Contact::Released,
            },
        ),
        build(60, SensorObservationKind::ShockObserved { severity_milli: permille(300) }),
        build(
            70,
            SensorObservationKind::GripContact {
                grip: GripBinding::Rear,
                contact: Contact::Released,
                strength_milli: permille(0),
            },
        ),
    ]
}

fn main() {
    // 构造阶段不计数：板上这部分是常量初始化，不是运行期申请。
    let mut reducer = new_reducer();
    let samples = observations();
    let mut now = 0u64;

    CURRENT.store(0, Ordering::Relaxed);
    PEAK.store(0, Ordering::Relaxed);
    ACTIVE.store(true, Ordering::Relaxed);

    for sample in &samples {
        let _ = reducer.ingest(sample.clone(), now);
        now += 10;
    }

    let peak = PEAK.load(Ordering::Relaxed);
    let held = CURRENT.load(Ordering::Relaxed);

    // 逐帧状态克隆：对应"快照下发/100 Hz 角速率流"的真实顾虑。
    let state = reducer.current_state().clone();
    let after_clone = PEAK.load(Ordering::Relaxed);
    drop(state);
    ACTIVE.store(false, Ordering::Relaxed);

    println!(
        "KERNEL_HEAP peak={} held={} with_clone_peak={} samples={}",
        peak,
        held,
        after_clone,
        samples.len()
    );
}


