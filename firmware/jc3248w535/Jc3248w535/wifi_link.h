#pragma once

#include <Arduino.h>

// Wi-Fi 链路（M5）：STA 连接、TCP 服务端、UDP 发现应答、凭据存 NVS。
//
// 约定：
// - **凭据永不进仓库**。由串口命令 `WIFI <ssid> <pass>` 在运行时写入 NVS。
// - 板子做 TCP 服务端（默认 47900），电脑作为客户端连进来；这样板子不需要
//   知道电脑地址，只需要能被发现。
// - 发现：电脑向 255.255.255.255:47901 广播 `GS-DISCOVER`，板子回
//   `GS-HERE <ip>:<port>`。
// - 仅在 `GS_WIFI_ENABLED` 构建中出现；Wi-Fi 版固件必须配合外部资产
//   （`-BlobAssets`），否则二进制超过 2 MiB 分区。

namespace gunspirit::wifi_link {

/// 读取 NVS 中的凭据并尝试连接（无凭据则静默跳过）。
void begin();

/// 处理连接状态、接受客户端、转发收发。
void pump();

/// 处理 `WIFI` 子命令；返回 true 表示已消费该行。
/// 语法：`WIFI <ssid> <pass>` / `WIFI status` / `WIFI forget`
bool command(const char *line);

/// 当前是否已连接路由器。
bool connected();

/// 当前 TCP 客户端是否已连接。
bool client_connected();

/// 当前 IP（未连接返回空串）。
const char *ip();

}  // namespace gunspirit::wifi_link
