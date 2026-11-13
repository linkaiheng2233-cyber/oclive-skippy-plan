#include "imu.h"

#include <Wire.h>

namespace gunspirit {
namespace {

// ---- 总线 ----
// 显示板的 P3/P4 是同一组网络：1=GND、2=3.3V、3=GPIO17、4=GPIO18。
constexpr int kI2cSda = 17;
constexpr int kI2cScl = 18;
constexpr uint32_t kI2cClockHz = 400000;
// 0x68 是多数 breakout 的默认地址（AD0/SDO 接地）；若模块焊了 AD0 高则可能是 0x69。
constexpr uint8_t kPrimaryAddress = 0x68;
constexpr uint8_t kAlternateAddress = 0x69;

// ---- 寄存器（TDK ICM-456xx 家族）----
// 注意：ICM-45686 与 ICM-42688-P **不是同一套寄存器布局**，
// 最典型的差别就是 WHO_AM_I 在本型号位于 0x72、值为 0xE9。
constexpr uint8_t kRegWhoAmI = 0x72;
constexpr uint8_t kWhoAmIExpected = 0xE9;
constexpr uint8_t kRegPwrMgmt0 = 0x10;
constexpr uint8_t kRegAccelConfig0 = 0x1B;
constexpr uint8_t kRegGyroConfig0 = 0x1C;
constexpr uint8_t kRegAccelDataX1 = 0x1F;  // 0x1F..0x24（accel XYZ）
constexpr uint8_t kRegGyroDataX1 = 0x25;   // 0x25..0x2A（gyro XYZ）

// ---- 配置值（CANDIDATE：到货后必须用 `IMU RD` 核对回读）----
// PWR_MGMT0：accel 与 gyro 同时进低噪声模式。
constexpr uint8_t kPwrMgmt0LowNoise = 0x0F;
// ACCEL_CONFIG0 / GYRO_CONFIG0：高 3 位为量程，低 4 位为 ODR。
// 这里按 ±16 g / ±2000 dps 与 100 Hz 附近的常见编码取值，
// **量程与 ODR 的实际位编码必须对照数据手册核对后再信任读数**。
constexpr uint8_t kAccelConfig0 = 0x08;
constexpr uint8_t kGyroConfig0 = 0x08;

// 灵敏度（量程确定后才成立）：±16 g → 2048 LSB/g；±2000 dps → 16.4 LSB/dps。
constexpr float kAccelLsbPerG = 2048.0f;
constexpr float kGyroLsbPerDps = 16.4f;

bool bus_ready = false;
uint8_t active_address = kPrimaryAddress;

bool write_register(uint8_t reg, uint8_t value) {
    Wire.beginTransmission(active_address);
    Wire.write(reg);
    Wire.write(value);
    return Wire.endTransmission() == 0;
}

bool read_registers(uint8_t reg, uint8_t *buffer, size_t length) {
    Wire.beginTransmission(active_address);
    Wire.write(reg);
    if (Wire.endTransmission(false) != 0) {
        return false;
    }
    const size_t received = Wire.requestFrom(static_cast<int>(active_address), static_cast<int>(length));
    if (received != length) {
        return false;
    }
    for (size_t index = 0; index < length; ++index) {
        buffer[index] = static_cast<uint8_t>(Wire.read());
    }
    return true;
}

bool read_register(uint8_t reg, uint8_t *value) {
    return read_registers(reg, value, 1);
}

int16_t to_int16(uint8_t high, uint8_t low) {
    return static_cast<int16_t>((static_cast<uint16_t>(high) << 8) | low);
}

}  // namespace

bool imu_init(uint8_t *chip_id) {
    Wire.begin(kI2cSda, kI2cScl);
    Wire.setClock(kI2cClockHz);
    bus_ready = true;

    // 两个候选地址都试一遍：模块可能把 AD0 拉高。
    const uint8_t candidates[2] = {kPrimaryAddress, kAlternateAddress};
    for (uint8_t address : candidates) {
        active_address = address;
        uint8_t value = 0;
        if (read_register(kRegWhoAmI, &value)) {
            if (chip_id != nullptr) {
                *chip_id = value;
            }
            return true;
        }
    }
    if (chip_id != nullptr) {
        *chip_id = 0;
    }
    return false;
}

bool imu_is_expected_chip(uint8_t chip_id) {
    return chip_id == kWhoAmIExpected;
}

bool imu_configure() {
    if (!bus_ready) {
        return false;
    }
    // 顺序：先进低噪声模式，再设量程与 ODR。
    bool ok = write_register(kRegPwrMgmt0, kPwrMgmt0LowNoise);
    ok = write_register(kRegAccelConfig0, kAccelConfig0) && ok;
    ok = write_register(kRegGyroConfig0, kGyroConfig0) && ok;
    return ok;
}

ImuReading imu_read() {
    ImuReading reading{};
    if (!bus_ready) {
        return reading;
    }
    uint8_t accel[6] = {0};
    uint8_t gyro[6] = {0};
    if (!read_registers(kRegAccelDataX1, accel, sizeof(accel))) {
        return reading;
    }
    if (!read_registers(kRegGyroDataX1, gyro, sizeof(gyro))) {
        return reading;
    }
    for (size_t axis = 0; axis < 3; ++axis) {
        reading.accel_raw[axis] = to_int16(accel[axis * 2], accel[axis * 2 + 1]);
        reading.gyro_raw[axis] = to_int16(gyro[axis * 2], gyro[axis * 2 + 1]);
        reading.accel_g[axis] = static_cast<float>(reading.accel_raw[axis]) / kAccelLsbPerG;
        reading.gyro_dps[axis] = static_cast<float>(reading.gyro_raw[axis]) / kGyroLsbPerDps;
    }
    reading.valid = true;
    return reading;
}

bool imu_peek(uint8_t reg, uint8_t *value) {
    if (!bus_ready || value == nullptr) {
        return false;
    }
    return read_register(reg, value);
}

bool imu_poke(uint8_t reg, uint8_t value) {
    if (!bus_ready) {
        return false;
    }
    return write_register(reg, value);
}

}  // namespace gunspirit
