#pragma once

#include <Arduino.h>

// 板端帧协议（S2）最小实现，与 Rust `ailive-gun-spirit-protocol` 的线格式一致。
//
// 设计目标：把"板子能按正式协议说话"这件事先跑通；STATE 的语义内容
// 在 M4 接入感知内核后替换，此处先发**全 Unknown 的占位快照**，
// 避免在没有内核时伪造看似真实的事实。

namespace gunspirit::wire {

/// 进入帧协议模式（退出 ASCII 行模式）。
void begin();

/// 退出帧协议模式，回到 ASCII 行模式（保留台架调试能力）。
void end();

/// 当前是否处于帧协议模式。
bool active();

/// 喂入一个字节；内部按帧解析并分发。
void feed(uint8_t byte);

/// 轮询：在帧协议模式下按约 1 Hz 发布 STATE 快照。
void pump(uint32_t now_ms);

/// 显示层实现：按表情 id 切换，成功返回 true。
bool apply_mood(const char *id);

}  // namespace gunspirit::wire
