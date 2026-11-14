#include "settings_ui.h"

#include <stdio.h>

namespace gunspirit::settings_ui {
namespace {

constexpr uint32_t kRefreshIntervalMs = 1000;
constexpr int kTabConnect = 0;
constexpr int kTabDiagnose = 1;

lv_obj_t *root = nullptr;      // 全屏遮罩
lv_obj_t *tab_button[2] = {nullptr, nullptr};
lv_obj_t *tab_label[2] = {nullptr, nullptr};
lv_obj_t *content = nullptr;   // 正文文本（多行）

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

/// 把快照渲染成页签正文。所有字符串都在栈上格式化，避免动态分配。
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
                 snapshot.wifi_connected ? "已连接" : (snapshot.wifi_present ? "未连接" : "未配置"),
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
}

void on_close_clicked(lv_event_t *) {
    hide();
}

lv_obj_t *make_tab(lv_obj_t *parent, const char *text, int x, int index) {
    lv_obj_t *button = lv_obj_create(parent);
    lv_obj_clear_flag(button, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_size(button, 150, 40);
    lv_obj_set_pos(button, x, 46);
    lv_obj_set_style_radius(button, 8, 0);
    lv_obj_set_style_border_width(button, 0, 0);
    lv_obj_add_event_cb(button, on_tab_clicked, LV_EVENT_CLICKED,
                        reinterpret_cast<void *>(static_cast<intptr_t>(index)));
    lv_obj_t *label = lv_label_create(button);
    lv_label_set_text(label, text);
    style_label(label, &lv_font_montserrat_20, 0x9C90B4);
    lv_obj_center(label);
    tab_button[index] = button;
    tab_label[index] = label;
    return button;
}

}  // namespace

void create(lv_disp_t *display) {
    if (root != nullptr) {
        return;
    }
    lv_obj_t *screen = lv_disp_get_scr_act(display);
    root = lv_obj_create(screen);
    lv_obj_clear_flag(root, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_size(root, 480, 320);
    lv_obj_set_pos(root, 0, 0);
    lv_obj_set_style_radius(root, 0, 0);
    lv_obj_set_style_border_width(root, 0, 0);
    lv_obj_set_style_bg_color(root, lv_color_hex(0x151022), 0);
    lv_obj_set_style_pad_all(root, 0, 0);
    lv_obj_add_flag(root, LV_OBJ_FLAG_HIDDEN);

    lv_obj_t *title = lv_label_create(root);
    lv_label_set_text(title, "设置");
    style_label(title, &lv_font_montserrat_28, 0xFFF5EF);
    lv_obj_set_pos(title, 16, 10);

    make_tab(root, "连接", 16, kTabConnect);
    make_tab(root, "诊断", 174, kTabDiagnose);

    lv_obj_t *close = lv_obj_create(root);
    lv_obj_clear_flag(close, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_size(close, 110, 40);
    lv_obj_set_pos(close, 356, 46);
    lv_obj_set_style_radius(close, 8, 0);
    lv_obj_set_style_border_width(close, 0, 0);
    lv_obj_set_style_bg_color(close, lv_color_hex(0x3A2F4E), 0);
    lv_obj_add_event_cb(close, on_close_clicked, LV_EVENT_CLICKED, nullptr);
    lv_obj_t *close_label = lv_label_create(close);
    lv_label_set_text(close_label, "返回");
    style_label(close_label, &lv_font_montserrat_20, 0xF7F0FF);
    lv_obj_center(close_label);

    content = lv_label_create(root);
    style_label(content, &lv_font_montserrat_14, 0xE6DDF5);
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
    lv_obj_clear_flag(root, LV_OBJ_FLAG_HIDDEN);
    shown = true;
    dirty = true;
}

void hide() {
    if (root == nullptr) {
        return;
    }
    lv_obj_add_flag(root, LV_OBJ_FLAG_HIDDEN);
    shown = false;
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

