#pragma once

#include <Arduino.h>

// IMU（ICM-45686）最小驱动。
//
// 设计取向：**到货即可验证**。寄存器常量按 TDK ICM-456xx 家族资料给出，
// 凡是有跨型号差异的项都标注 CANDIDATE，并允许用串口 `IMU RD/WR` 现场核对。
//
// 接线（显示板 P3/P4，同一网络）：
//   1 = GND、2 = 3.3 V、3 = GPIO17、4 = GPIO18
//   本驱动按 SDA=17、SCL=18 打开总线；若读不到设备，先考虑这两根是否接反
//   （I²C 接反不会损坏器件，只是无法通信）。

namespace gunspirit {

struct ImuReading {
    bool valid;
    int16_t accel_raw[3];
    int16_t gyro_raw[3];
    float accel_g[3];
    float gyro_dps[3];
};

/// 初始化 I²C 并读取 WHO_AM_I。成功返回 true。
/// `chip_id` 返回实际读到的值，便于调试时区分"没接"与"接错芯片"。
bool imu_init(uint8_t *chip_id);

/// 芯片 ID 是否为 ICM-45686（0xE9）。
bool imu_is_expected_chip(uint8_t chip_id);

/// 写配置寄存器（电源模式、量程、ODR）。返回是否全部写成功。
bool imu_configure();

/// 读取一次三轴数据与换算结果。
ImuReading imu_read();

/// 读单个寄存器，供台架核对寄存器表。
bool imu_peek(uint8_t reg, uint8_t *value);

/// 写单个寄存器，供台架试探配置。
bool imu_poke(uint8_t reg, uint8_t value);

}  // namespace gunspirit
