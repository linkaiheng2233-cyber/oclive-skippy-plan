#pragma once

#include <stddef.h>
#include <stdint.h>

// Rust 感知内核的 C ABI（由 firmware/kernel-shim 提供）。
// 仅在定义 GS_KERNEL_ENABLED 时链接；未链接的构建不引用这些符号。
extern "C" {
uint32_t gs_kernel_abi_version(void);
int32_t gs_kernel_init(void);
int32_t gs_kernel_ready(void);
int32_t gs_kernel_self_test(void);
uint64_t gs_kernel_state_revision(void);

// grip: 0=前握 1=后握；contact: 0=接触 1=释放
int32_t gs_kernel_ingest_grip(uint8_t grip, uint8_t contact, uint16_t strength_milli,
                              uint64_t millis);
// motion: 0=静止 1=移动
int32_t gs_kernel_ingest_motion(uint8_t motion, uint64_t millis);
int32_t gs_kernel_ingest_orientation(int32_t gravity_x_mg, int32_t gravity_y_mg,
                                     int32_t gravity_z_mg, uint16_t quality_milli,
                                     uint64_t millis);
// contact: 0=接触 1=释放
int32_t gs_kernel_ingest_control(uint8_t contact, uint64_t millis);
int32_t gs_kernel_tick(uint64_t millis);

// 把内核快照编码成协议 STATE 载荷；返回字节数，负数表示错误。
int32_t gs_kernel_encode_state(uint8_t *output, size_t capacity);
}
