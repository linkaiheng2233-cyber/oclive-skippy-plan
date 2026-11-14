#pragma once

#include <Arduino.h>

// IMU（ICM-45686）驱动与**服务框架**。
//
// 三个层次：
// 1. **探测/配置**（init + configure）：已实现，寄存器常量标注 CANDIDATE；
// 2. **采样服务**（service）：由主循环按固定周期调用，读一次三轴并送入感知内核；
// 3. **标定**（calibrate）：静止采样求加速度计零偏，结果存 NVS，采样时扣除。
//
// 无硬件时框架照样成立：`AUTO off` 时 service 什么都不做；标定命令会明确报告
// "没有可标定的设备"，不会伪造数据。
//
// 接线（显示板 P3/P4，同一网络）：
//   1 = GND、2 = 3.3 V、3 = GPIO17、4 = GPIO18
//   本驱动按 SDA=17、SCL=18 打开总线；读不到设备时先考虑这两根是否接反
//   （I²C 接反不会损坏器件，只是无法通信）。

namespace gunspirit {

/// 驱动状态。
enum class ImuStatus : uint8_t {
    Uninitialized = 0,  ///< 尚未探测
    Missing,            ///< 探测失败（未接线/地址不对/芯片不响应）
    Ready,              ///< 探测成功且已配置，但未开启自动采样
    Sampling,           ///< 自动采样中
    Fault,              ///< 采样连续失败，已自动降级
};

struct ImuReading {
    bool valid;
    int16_t accel_raw[3];
    int16_t gyro_raw[3];
    float accel_g[3];
    float gyro_dps[3];
};

/// 初始化 I²C 并读取 WHO_AM_I。成功返回 true，`chip_id` 返回实际读到的值。
bool imu_init(uint8_t *chip_id);

/// 芯片 ID 是否为 ICM-45686（0xE9）。
bool imu_is_expected_chip(uint8_t chip_id);

/// 写配置寄存器（电源模式、量程、ODR）。
bool imu_configure();

/// 读取一次三轴数据（已扣除标定零偏）。
ImuReading imu_read();

/// 读/写单个寄存器，供台架核对寄存器表。
bool imu_peek(uint8_t reg, uint8_t *value);
bool imu_poke(uint8_t reg, uint8_t value);

// --- 服务框架 -------------------------------------------------------------

/// 当前状态。
ImuStatus imu_status();

/// 状态名（用于串口输出）。
const char *imu_status_name(ImuStatus status);

/// 是否已成功探测到设备。
bool imu_present();

/// 开启/关闭自动采样服务。
void imu_set_auto(bool enabled);
bool imu_auto_enabled();

/// 主循环周期调用：到期则读一次、扣除零偏、按需送入感知内核。
/// 返回本次是否真正采样（便于上层统计）。
bool imu_service(uint32_t now_ms);

/// 注入送内核的回调。默认实现调用 gs_kernel_ingest_orientation/motion；
/// 在未链接内核的构建里由 display_app 传空。
using ImuIngestSink = void (*)(int32_t gravity_x_mg, int32_t gravity_y_mg, int32_t gravity_z_mg,
                               bool moving, uint64_t millis);
void imu_set_ingest_sink(ImuIngestSink sink);

// --- 标定 -----------------------------------------------------------------

/// 开始静止标定（采集 N 个样本求均值）。
bool imu_calibrate_begin();
/// 结束标定；成功时把零偏写入 NVS 并立即生效。
bool imu_calibrate_end();
/// 读取当前零偏（g）。
void imu_bias(float *x_g, float *y_g, float *z_g);
/// 清除零偏（NVS 与内存）。
void imu_calibrate_clear();
/// 标定是否进行中。
bool imu_calibrating();

}  // namespace gunspirit
