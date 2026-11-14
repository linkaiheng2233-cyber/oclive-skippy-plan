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

/// 帧出口当前是否指向 socket（网络客户端）而不是串口。
bool using_socket();

/// 对端是否已完成 HELLO 握手（事件必须等握手后再发，否则主机按 HandshakeRequired 拒绝）。
bool peer_seen();

/// 发送一条 EVENT 载荷（板端封装好的 DeviceEvent 字节）。
bool publish_event(const uint8_t *payload, size_t length);

/// 统计：已分发的合法帧总数。
uint32_t frames_received();

/// 统计：坏帧数（版本不符、未知类型、CRC 失败）。
uint32_t frame_errors();

/// 喂入一个字节；内部按帧解析并分发。
void feed(uint8_t byte);

/// 轮询：在帧协议模式下按约 1 Hz 发布 STATE 快照。
void pump(uint32_t now_ms);

/// 显示层实现：按表情 id 切换，成功返回 true。
bool apply_mood(const char *id);

/// 帧发送出口。默认写串口；Wi-Fi 连接后由传输层替换为 socket 写入。
using FrameWriter = size_t (*)(const uint8_t *data, size_t length);
void set_frame_writer(FrameWriter writer);

}  // namespace gunspirit::wire





