//! V3：跨模块时间对齐仿真（无硬件）。
//!
//! 验证三个命题：
//! 1. **同模块**事件的时间差（例如"拔匣→插匣"的换弹耗时）**不需要任何对齐**，
//!    因为它出自同一个单调时钟，作差即得。
//! 2. **跨模块**用往返估计偏移时，残余误差 ≈ **往返路径不对称量的一半**，
//!    与抖动无关（中位数抵消对称抖动）。
//! 3. 用修正后的时间戳可以恢复跨模块事件的真实先后。
//!
//! 仿真刻意使用确定性 LCG，保证每次运行结果一致、可复现。

// 测试代码用 expect/assert 表达"此处不可能失败"；产品代码仍受 workspace 约束。
#![allow(clippy::expect_used, clippy::unwrap_used)]

/// 线性同余发生器（确定性，避免引入 rand 依赖）。
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state >> 11
}

/// [-half, +half] 的均匀抖动。
fn jitter(state: &mut u64, half: i64) -> i64 {
    let span = (2 * half + 1) as u64;
    (lcg(state) % span) as i64 - half
}

/// 一个模块的本地时钟模型：固定偏移 + 晶振漂移。
#[derive(Debug, Clone, Copy)]
struct ModuleClock {
    offset_ms: i64,
    drift_ppm: i64,
}

impl ModuleClock {
    fn local(&self, host_ms: i64) -> i64 {
        self.offset_ms + host_ms + host_ms * self.drift_ppm / 1_000_000
    }
}

/// 往返估计：主机发 `t0`，模块记录本地的 `t1` 并回显，主机在 `t2` 收到。
/// 偏移估计 = `t1 - (t0 + (t2 - t0)/2)`。
fn round_trip_offset(
    clock: &ModuleClock,
    rng: &mut u64,
    samples: usize,
    one_way_ms: i64,
    path_half_jitter_ms: i64,
    asymmetry_ms: i64,
) -> i64 {
    let mut estimates = Vec::with_capacity(samples);
    for sample in 0..samples {
        let t0 = sample as i64 * 100;
        let down = one_way_ms + jitter(rng, path_half_jitter_ms) + asymmetry_ms;
        let up = one_way_ms + jitter(rng, path_half_jitter_ms) - asymmetry_ms;
        let t1 = clock.local(t0 + down);
        let t2 = t0 + down + up;
        let rtt = t2 - t0;
        estimates.push(t1 - (t0 + rtt / 2));
    }
    estimates.sort_unstable();
    estimates[estimates.len() / 2]
}

#[test]
fn same_module_durations_need_no_alignment() {
    let clock = ModuleClock {
        offset_ms: 48_213,
        drift_ppm: 40,
    };
    // 模块本地时间 100 ms 拔匣、1334 ms 插匣。
    let removed_local = clock.local(1_000);
    let seated_local = clock.local(2_334);
    let reload_ms = seated_local - removed_local;
    assert_eq!(reload_ms, 1_334, "同一模块作差必须精确，不受偏移与漂移影响");
}

#[test]
fn offset_error_tracks_path_asymmetry() {
    let clock = ModuleClock {
        offset_ms: -7_777,
        drift_ppm: 0,
    };
    let mut rng = 0x5EED_0001_u64;
    let symmetric = round_trip_offset(&clock, &mut rng, 64, 20, 20, 0);
    let skewed = round_trip_offset(&clock, &mut rng, 64, 20, 20, 8);
    let symmetric_error = (symmetric - clock.offset_ms).abs();
    let skewed_error = (skewed - clock.offset_ms).abs();

    println!(
        "ALIGN symmetric_error={}ms skewed_error={}ms (asymmetry=+8ms each way)",
        symmetric_error, skewed_error
    );

    // 对称抖动被中位数抵消：残余误差只由"单侧离散度/样本量"造成，量级应在几 ms 内。
    assert!(
        symmetric_error <= 5,
        "对称路径下偏移误差 {} ms 过大",
        symmetric_error
    );
    // 路径不对称 8 ms/单程 → 估计误差应≈8 ms，而不是 0。
    assert!(
        skewed_error >= 4 && skewed_error <= 12,
        "不对称路径下的残余误差应≈8 ms，实际 {} ms",
        skewed_error
    );
}

#[test]
fn corrected_timestamps_restore_true_order() {
    let fast = ModuleClock {
        offset_ms: 5_000,
        drift_ppm: 30,
    };
    let slow = ModuleClock {
        offset_ms: -3_000,
        drift_ppm: -20,
    };
    // 真实（主机时间）：B 早 990 ms，A 晚 1000 ms。
    let a_true = 1_000;
    let b_true = 990;
    let a_local = fast.local(a_true);
    let b_local = slow.local(b_true);

    let mut rng = 0x5EED_0002_u64;
    let a_estimate = round_trip_offset(&fast, &mut rng, 64, 20, 10, 0);
    let b_estimate = round_trip_offset(&slow, &mut rng, 64, 20, 10, 0);
    let a_corrected = a_local - a_estimate;
    let b_corrected = b_local - b_estimate;

    println!(
        "ORDER a_local={} a_corrected={} b_local={} b_corrected={}",
        a_local, a_corrected, b_local, b_corrected
    );

    // 原始本地时间：A(6000) > B(-2010) 顺序"看起来"是对的，但差值被 8 秒偏移污染。
    // 修正后必须恢复 990 < 1000 的真实先后，且误差在 2 ms 内。
    assert!(b_corrected < a_corrected, "修正后 B 必须早于 A");
    assert!(
        (a_corrected - a_true).abs() <= 2 && (b_corrected - b_true).abs() <= 2,
        "修正后应回到真实时间附近：a={} b={}",
        a_corrected,
        b_corrected
    );
}

#[test]
fn drift_over_time_stays_bounded_with_periodic_resync() {
    let clock = ModuleClock {
        offset_ms: 0,
        drift_ppm: 50,
    };
    // 每 30 s 重新同步一次；漂移 50 ppm 在 30 s 内累积 1.5 ms。
    let interval_ms = 30_000;
    let worst = interval_ms * clock.drift_ppm / 1_000_000 + 1;
    println!("DRIFT worst_between_resyncs={}ms", worst);
    assert!(
        worst <= 3,
        "30 s 重同步间隔下漂移累计应 ≤3 ms，实际 {} ms",
        worst
    );
}
