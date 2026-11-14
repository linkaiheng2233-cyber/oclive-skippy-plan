#include "imu.h"

#include <Preferences.h>
#include <Wire.h>

namespace gunspirit {
namespace {

// ---- 总线 ----
constexpr int kI2cSda = 17;
constexpr int kI2cScl = 18;
constexpr uint32_t kI2cClockHz = 400000;
constexpr uint8_t kPrimaryAddress = 0x68;
constexpr uint8_t kAlternateAddress = 0x69;

// ---- 寄存器（TDK ICM-456xx 家族）----
// 注意：ICM-45686 与 ICM-42688-P 不是同一套寄存器布局，
// 最典型的差别是 WHO_AM_I 在本型号位于 0x72、值为 0xE9。
constexpr uint8_t kRegWhoAmI = 0x72;
constexpr uint8_t kWhoAmIExpected = 0xE9;
constexpr uint8_t kRegPwrMgmt0 = 0x10;
constexpr uint8_t kRegAccelConfig0 = 0x1B;
constexpr uint8_t kRegGyroConfig0 = 0x1C;
constexpr uint8_t kRegAccelDataX1 = 0x1F;  // 0x1F..0x24
constexpr uint8_t kRegGyroDataX1 = 0x25;   // 0x25..0x2A

// ---- 配置值（CANDIDATE：到货后必须用 IMU RD 核对回读）----
constexpr uint8_t kPwrMgmt0LowNoise = 0x0F;
constexpr uint8_t kAccelConfig0 = 0x08;
constexpr uint8_t kGyroConfig0 = 0x08;

// 灵敏度（量程确定后才成立）：±16 g → 2048 LSB/g；±2000 dps → 16.4 LSB/dps。
constexpr float kAccelLsbPerG = 2048.0f;
constexpr float kGyroLsbPerDps = 16.4f;

// ---- 服务参数 ----
constexpr uint32_t kSamplePeriodMs = 10;      // 100 Hz
constexpr uint32_t kMotionWindowMs = 500;     // 运动判定的观察窗
constexpr float kMotionThresholdG = 0.08f;    // 合加速度偏离 1 g 的阈值
constexpr uint8_t kFaultThreshold = 5;        // 连续失败次数达到即降级
constexpr uint16_t kCalibrationSamples = 200; // 2 秒 ×100 Hz
constexpr const char *kNvsNamespace = "gunspirit";

bool bus_ready = false;
uint8_t active_address = kPrimaryAddress;
ImuStatus status = ImuStatus::Uninitialized;
bool auto_enabled = false;
uint8_t consecutive_failures = 0;
uint32_t last_sample_ms = 0;

// 标定零偏（g）。采样时扣除。
float bias_g[3] = {0.0f, 0.0f, 0.0f};
Preferences preferences;

// 运动判定的滑动窗口：记录最近窗口内合加速度与 1 g 的最大偏离。
float motion_peak_deviation = 0.0f;
uint32_t motion_window_start_ms = 0;

ImuIngestSink ingest_sink = nullptr;

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
    const size_t received =
        Wire.requestFrom(static_cast<int>(active_address), static_cast<int>(length));
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

void load_bias() {
    if (!preferences.begin(kNvsNamespace, true)) {
        return;
    }
    bias_g[0] = preferences.getFloat("imu_bx", 0.0f);
    bias_g[1] = preferences.getFloat("imu_by", 0.0f);
    bias_g[2] = preferences.getFloat("imu_bz", 0.0f);
    preferences.end();
}

void store_bias() {
    if (!preferences.begin(kNvsNamespace, false)) {
        return;
    }
    preferences.putFloat("imu_bx", bias_g[0]);
    preferences.putFloat("imu_by", bias_g[1]);
    preferences.putFloat("imu_bz", bias_g[2]);
    preferences.end();
}

// 标定采集状态
bool calibrating = false;
uint16_t calibration_count = 0;
double calibration_sum[3] = {0.0, 0.0, 0.0};

}  // namespace

bool imu_init(uint8_t *chip_id) {
    Wire.begin(kI2cSda, kI2cScl);
    Wire.setClock(kI2cClockHz);
    bus_ready = true;
    load_bias();

    const uint8_t candidates[2] = {kPrimaryAddress, kAlternateAddress};
    for (uint8_t address : candidates) {
        active_address = address;
        uint8_t value = 0;
        if (read_register(kRegWhoAmI, &value)) {
            if (chip_id != nullptr) {
                *chip_id = value;
            }
            consecutive_failures = 0;
            status = ImuStatus::Ready;
            return true;
        }
    }
    if (chip_id != nullptr) {
        *chip_id = 0;
    }
    status = ImuStatus::Missing;
    Serial.println("IMU ERR NO_DEVICE (check P3: 1=GND 2=3V3 3=GPIO17 4=GPIO18)");
    return false;
}

bool imu_is_expected_chip(uint8_t chip_id) {
    return chip_id == kWhoAmIExpected;
}

bool imu_configure() {
    if (!bus_ready) {
        return false;
    }
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

// ---------------------------------------------------------------------------
// 服务框架
// ---------------------------------------------------------------------------

ImuStatus imu_status() {
    return status;
}

const char *imu_status_name(ImuStatus current) {
    switch (current) {
        case ImuStatus::Uninitialized:
            return "uninitialized";
        case ImuStatus::Missing:
            return "missing";
        case ImuStatus::Ready:
            return "ready";
        case ImuStatus::Sampling:
            return "sampling";
        case ImuStatus::Fault:
            return "fault";
    }
    return "unknown";
}

bool imu_present() {
    return status == ImuStatus::Ready || status == ImuStatus::Sampling;
}

void imu_set_auto(bool enabled) {
    if (enabled && !imu_present()) {
        Serial.println("IMU ERR NO_DEVICE (auto not enabled)");
        return;
    }
    auto_enabled = enabled;
    last_sample_ms = 0;
    motion_window_start_ms = 0;
    motion_peak_deviation = 0.0f;
    if (enabled) {
        status = ImuStatus::Sampling;
        Serial.printf("IMU auto on period=%lums\n", static_cast<unsigned long>(kSamplePeriodMs));
    } else if (status == ImuStatus::Sampling) {
        status = ImuStatus::Ready;
        Serial.println("IMU auto off");
    }
}

bool imu_auto_enabled() {
    return auto_enabled;
}

void imu_set_ingest_sink(ImuIngestSink sink) {
    ingest_sink = sink;
}

bool imu_service(uint32_t now_ms) {
    if (!auto_enabled || !bus_ready) {
        return false;
    }
    if (now_ms - last_sample_ms < kSamplePeriodMs) {
        return false;
    }
    last_sample_ms = now_ms;

    const ImuReading reading = imu_read();
    if (!reading.valid) {
        if (++consecutive_failures >= kFaultThreshold) {
            status = ImuStatus::Fault;
            auto_enabled = false;
            Serial.println("IMU ERR READ_FAILURE (auto stopped)");
        }
        return false;
    }
    consecutive_failures = 0;

    // 标定采集：只累加原始读数，不改标定结果。
    if (calibrating) {
        for (size_t axis = 0; axis < 3; ++axis) {
            calibration_sum[axis] += static_cast<double>(reading.accel_g[axis]);
        }
        if (++calibration_count >= kCalibrationSamples) {
            Serial.println("IMU CAL samples ready (run: IMU CAL end)");
            calibration_count = kCalibrationSamples;
        }
        return true;
    }

    // 扣零偏。
    float gravity_g[3];
    for (size_t axis = 0; axis < 3; ++axis) {
        gravity_g[axis] = reading.accel_g[axis] - bias_g[axis];
    }

    // 运动判定：合加速度偏离 1 g 的幅度，在窗口内取峰值。
    const float magnitude =
        sqrtf(gravity_g[0] * gravity_g[0] + gravity_g[1] * gravity_g[1] + gravity_g[2] * gravity_g[2]);
    const float deviation = fabsf(magnitude - 1.0f);
    if (motion_window_start_ms == 0) {
        motion_window_start_ms = now_ms;
        motion_peak_deviation = deviation;
    }
    if (deviation > motion_peak_deviation) {
        motion_peak_deviation = deviation;
    }
    bool moving = false;
    if (now_ms - motion_window_start_ms >= kMotionWindowMs) {
        moving = motion_peak_deviation >= kMotionThresholdG;
        motion_window_start_ms = now_ms;
        motion_peak_deviation = 0.0f;
    }

    if (ingest_sink != nullptr) {
        ingest_sink(static_cast<int32_t>(gravity_g[0] * 1000.0f),
                    static_cast<int32_t>(gravity_g[1] * 1000.0f),
                    static_cast<int32_t>(gravity_g[2] * 1000.0f), moving,
                    static_cast<uint64_t>(now_ms));
    }
    return true;
}

// ---------------------------------------------------------------------------
// 标定
// ---------------------------------------------------------------------------

bool imu_calibrate_begin() {
    if (!imu_present()) {
        Serial.println("IMU ERR NO_DEVICE (calibration needs a detected IMU)");
        return false;
    }
    calibrating = true;
    calibration_count = 0;
    calibration_sum[0] = 0.0;
    calibration_sum[1] = 0.0;
    calibration_sum[2] = 0.0;
    Serial.printf("IMU CAL start samples=%u (keep still, auto must be on)\n",
                  static_cast<unsigned>(kCalibrationSamples));
    return true;
}

bool imu_calibrate_end() {
    if (!calibrating) {
        Serial.println("IMU ERR CAL not started");
        return false;
    }
    if (calibration_count < kCalibrationSamples) {
        Serial.printf("IMU ERR CAL not enough samples (%u/%u)\n",
                      static_cast<unsigned>(calibration_count),
                      static_cast<unsigned>(kCalibrationSamples));
        return false;
    }
    calibrating = false;
    // 静止时合加速度应为 1 g；把均值直接作为零偏扣除。
    for (size_t axis = 0; axis < 3; ++axis) {
        bias_g[axis] = static_cast<float>(calibration_sum[axis] / calibration_count);
    }
    store_bias();
    Serial.printf("IMU CAL done bias_g=[%.4f,%.4f,%.4f]\n", bias_g[0], bias_g[1], bias_g[2]);
    return true;
}

void imu_bias(float *x_g, float *y_g, float *z_g) {
    if (x_g != nullptr) {
        *x_g = bias_g[0];
    }
    if (y_g != nullptr) {
        *y_g = bias_g[1];
    }
    if (z_g != nullptr) {
        *z_g = bias_g[2];
    }
}

void imu_calibrate_clear() {
    calibrating = false;
    calibration_count = 0;
    bias_g[0] = 0.0f;
    bias_g[1] = 0.0f;
    bias_g[2] = 0.0f;
    store_bias();
    Serial.println("IMU CAL cleared");
}

bool imu_calibrating() {
    return calibrating;
}

}  // namespace gunspirit
