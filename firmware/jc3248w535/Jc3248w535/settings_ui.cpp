#include "settings_ui.h"

#include <Arduino.h>
#include <stdio.h>

LV_FONT_DECLARE(lv_font_ui_16);

namespace gunspirit::settings_ui {
namespace {

constexpr uint32_t kRefreshIntervalMs = 1000;
constexpr int kTabConnect = 0;
constexpr int kTabDiagnose = 1;

// 独立屏幕：不用遮罩层。板厂 BSP 的触摸走 LVGL 标准 indev，
// 遮罩层上自定义对象的手势分发容易踩坑（实测点不动），
// 而"切换屏幕 + 标准 lv_btn"是最稳的组合。
lv_obj_t *root = nullptr;
lv_obj_t *main_screen = nullptr;
lv_obj_t *tab_button[2] = {nullptr, nullptr};
lv_obj_t *tab_label[2] = {nullptr, nullptr};
lv_obj_t *content = nullptr;

int active_tab = kTabConnect;
bool shown = false;
bool dirty = true;
uint32_t last_refresh_ms = 0;
SettingsProvider provider = nullptr;

void style_label(lv_obj_t *label, const lv_font_t *font, uint32_t color) {
    lv_obj_set_style_text_font(label, font, 0);
    lv_obj_set_style_text_color(label, lv_color_hex(color), 0);
}

void refresh_tabs() {
    for (int index = 0; index < 2; ++index) {
        const bool active = index == active_tab;
        lv_obj_set_style_bg_color(tab_button[index],
                                  lv_color_hex(active ? 0xD6C8FF : 0x2A2338), 0);
        lv_obj_set_style_text_color(tab_label[index],
                                    lv_color_hex(active ? 0x211735 : 0x9C90B4), 0);
    }
}

/// 把快照渲染成页签正文。
void render_content(const Snapshot &snapshot) {
    char buffer[640];
    if (active_tab == kTabConnect) {
        snprintf(buffer, sizeof(buffer),
                 "Wi-Fi\n"
                 "  %s\n"
                 "  SSID  %s\n"
                 "  地址  %s\n"
                 "  电脑  %s\n"
                 "\n"
                 "说明：凭据只存在设备内部；\n"
                 "电脑与设备需在同一网络。\n"
                 "\n"
                 "操作请用串口命令（WIFI ...），\n"
                 "本页只显示状态，避免误触失联。",
                 snapshot.wifi_connected ? "已连接"
                                         : (snapshot.wifi_present ? "未连接" : "未配置"),
                 snapshot.wifi_ssid[0] != '\0' ? snapshot.wifi_ssid : "-",
                 snapshot.wifi_ip[0] != '\0' && snapshot.wifi_ip[0] != '0' ? snapshot.wifi_ip : "-",
                 snapshot.wifi_client ? "已连接" : "未连接");
    } else {
        snprintf(buffer, sizeof(buffer),
                 "固件  %s\n"
                 "\n"
                 "内核  %s  ABI %u\n"
                 "        修订 %llu\n"
                 "\n"
                 "IMU   %s  自动 %s\n"
                 "\n"
                 "内存\n"
                 "  内部空闲 %u B\n"
                 "  内部最低 %u B\n"
                 "  PSRAM  %u B\n"
                 "\n"
                 "协议帧  正常 %u / 错误 %u",
                 snapshot.firmware_version, snapshot.kernel_ready ? "就绪" : "未启用",
                 snapshot.kernel_abi, snapshot.kernel_revision, snapshot.imu_status,
                 snapshot.imu_auto ? "开" : "关", snapshot.heap_free, snapshot.heap_min,
                 snapshot.psram_free, snapshot.frames_ok, snapshot.frames_bad);
    }
    lv_label_set_text(content, buffer);
}

void on_tab_clicked(lv_event_t *event) {
    const int index = static_cast<int>(reinterpret_cast<intptr_t>(lv_event_get_user_data(event)));
    if (index < 0 || index > 1) {
        return;
    }
    active_tab = index;
    refresh_tabs();
    dirty = true;
    Serial.printf("SETTINGS tab=%d\n", index);
}

void on_close_clicked(lv_event_t *) {
    Serial.println("SETTINGS close");
    hide();
}

/// 标准按钮：与主屏 Prev/Next 用同一类对象，触摸行为已被验证可用。
lv_obj_t *make_button(lv_obj_t *parent, const char *text, lv_coord_t x, lv_coord_t y,
                      lv_coord_t width, uint32_t background, uint32_t text_color) {
    lv_obj_t *button = lv_btn_create(parent);
    lv_obj_clear_flag(button, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_size(button, width, 40);
    lv_obj_set_pos(button, x, y);
    lv_obj_set_style_radius(button, 8, 0);
    lv_obj_set_style_shadow_width(button, 0, 0);
    lv_obj_set_style_bg_color(button, lv_color_hex(background), 0);
    lv_obj_set_style_bg_color(button, lv_color_hex(0xBEABF8), LV_STATE_PRESSED);
    lv_obj_t *label = lv_label_create(button);
    lv_label_set_text(label, text);
    style_label(label, &lv_font_ui_16, text_color);
    lv_obj_center(label);
    return button;
}

}  // namespace

void create(lv_obj_t *screen) {
    if (root != nullptr) {
        return;
    }
    main_screen = screen;
    root = lv_obj_create(nullptr);  // 独立屏幕
    lv_obj_clear_flag(root, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_style_bg_color(root, lv_color_hex(0x151022), 0);
    lv_obj_set_style_pad_all(root, 0, 0);

    lv_obj_t *title = lv_label_create(root);
    lv_label_set_text(title, "设置");
    style_label(title, &lv_font_ui_16, 0xFFF5EF);
    lv_obj_set_pos(title, 16, 8);

    tab_button[kTabConnect] =
        make_button(root, "连接", 16, 44, 150, 0xD6C8FF, 0x211735);
    lv_obj_add_event_cb(tab_button[kTabConnect], on_tab_clicked, LV_EVENT_CLICKED,
                        reinterpret_cast<void *>(static_cast<intptr_t>(kTabConnect)));
    tab_label[kTabConnect] = lv_obj_get_child(tab_button[kTabConnect], 0);

    tab_button[kTabDiagnose] =
        make_button(root, "诊断", 174, 44, 150, 0x2A2338, 0x9C90B4);
    lv_obj_add_event_cb(tab_button[kTabDiagnose], on_tab_clicked, LV_EVENT_CLICKED,
                        reinterpret_cast<void *>(static_cast<intptr_t>(kTabDiagnose)));
    tab_label[kTabDiagnose] = lv_obj_get_child(tab_button[kTabDiagnose], 0);

    lv_obj_t *close = make_button(root, "返回", 356, 44, 110, 0x3A2F4E, 0xF7F0FF);
    lv_obj_add_event_cb(close, on_close_clicked, LV_EVENT_CLICKED, nullptr);

    content = lv_label_create(root);
    style_label(content, &lv_font_ui_16, 0xE6DDF5);
    lv_obj_set_pos(content, 16, 100);
    lv_label_set_text(content, "");

    refresh_tabs();
    dirty = true;
}

void set_provider(SettingsProvider new_provider) {
    provider = new_provider;
    dirty = true;
}

void show() {
    if (root == nullptr) {
        return;
    }
    lv_scr_load(root);
    shown = true;
    dirty = true;
    Serial.println("SETTINGS shown");
}

void hide() {
    if (root == nullptr || main_screen == nullptr) {
        return;
    }
    lv_scr_load(main_screen);
    shown = false;
    Serial.println("SETTINGS hidden");
}

bool visible() {
    return shown;
}

void invalidate() {
    dirty = true;
}

void tick(unsigned long now_ms) {
    if (!shown || provider == nullptr) {
        return;
    }
    if (!dirty && now_ms - last_refresh_ms < kRefreshIntervalMs) {
        return;
    }
    last_refresh_ms = now_ms;
    dirty = false;
    Snapshot snapshot{};
    provider(&snapshot);
    render_content(snapshot);
}

}  // namespace gunspirit::settings_ui

