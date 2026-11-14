#pragma once

#include <stdint.h>

// Rust 感知内核的 C ABI（由 firmware/kernel-shim 提供）。
// 仅在定义 GS_KERNEL_ENABLED 时链接；未链接的构建不引用这些符号。
extern "C" {
uint32_t gs_kernel_abi_version(void);
int32_t gs_kernel_self_test(void);
uint64_t gs_kernel_state_revision(void);
}
