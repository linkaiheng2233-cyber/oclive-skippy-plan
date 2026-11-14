#pragma once

#include <lvgl.h>

// 屏幕设置层（P1 连接页 + P4 诊断页）。
//
// 设计取舍（刻意从简）：
// - **只读展示**：本阶段不做"清凭据/切 Wi-Fi"这类动作，避免误触导致设备失联；
// - 数据由外部通过 `SettingsProvider` 填入，本模块不认识 Wi-Fi/内核/协议；
// - 两个页签 + 一个返回键，不做滑动与多级导航（小屏误触率高）。
namespace gunspirit::settings_ui {

/// 一次快照：调用方在 tick 时填好，本模块只负责显示。
struct Snapshot {
    char firmware_version[24];

    bool wifi_present;
    bool wifi_connected;
    bool wifi_client;
    char wifi_ssid[33];
    char wifi_ip[16];

    bool imu_present;
    bool imu_auto;
    char imu_status[16];

    bool kernel_ready;
    unsigned kernel_abi;
    unsigned long long kernel_revision;

    unsigned heap_free;
    unsigned heap_min;
    unsigned psram_free;

    unsigned frames_ok;
    unsigned frames_bad;
};

using SettingsProvider = void (*)(Snapshot *snapshot);

/// 创建设置层（默认隐藏）。`display` 由调用方提供，用于挂到当前屏幕。
void create(lv_disp_t *display);

/// 注册快照来源。
void set_provider(SettingsProvider provider);

/// 显示/隐藏/是否可见。
void show();
void hide();
bool visible();

/// 周期刷新（仅在可见时真正更新文本）。
void tick(unsigned long now_ms);

/// 外部状态变化时调用，强制下一次 tick 立即刷新。
void invalidate();

}  // namespace gunspirit::settings_ui
