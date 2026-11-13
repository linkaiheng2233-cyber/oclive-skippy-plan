#include <Arduino.h>
#include <freertos/FreeRTOS.h>
#include <freertos/queue.h>
#include <freertos/task.h>
#include <inttypes.h>
#include <string.h>

// 诊断用：把 Wi-Fi / BLE 协议栈单独或一起链进来，量化各自的 flash 与静态 RAM 代价。
// 生产构建不定义任何 DIAG_* 宏。
#if defined(DIAG_WIFI) || defined(DIAG_BLE) || defined(DIAG_RADIO)
#define DIAG_ANY_RADIO 1
#endif
#ifdef DIAG_WIFI
#include <WiFi.h>
#endif
#ifdef DIAG_BLE
#include <BLEDevice.h>
#include <BLEServer.h>
#endif

#include "display.h"
#include "imu.h"
#include "gs_wire.h"
#include "esp_bsp.h"
#include "portrait_assets.h"

#ifdef USE_BLOB_ASSETS
#include <esp_heap_caps.h>
#include <esp_partition.h>

namespace {

// 外部资产 blob：16 字节头 + N x 24 字节索引 + 原始 RGB565 数据。
// 与 pack-portraits.py 的 generate_external() 严格对齐。
constexpr uint32_t kAssetMagic = 0x31415347;  // "GSA1" 小端
struct AssetIndex {
    char name[16];
    uint32_t offset;
    uint32_t length;
};

bool load_assets_from_flash() {
    const esp_partition_t *partition = esp_partition_find_first(
        ESP_PARTITION_TYPE_DATA, ESP_PARTITION_SUBTYPE_DATA_FAT, "ffat");
    if (partition == nullptr) {
        Serial.println("ERR ASSET_PARTITION");
        return false;
    }
    uint8_t header[16] = {0};
    if (esp_partition_read(partition, 0, header, sizeof(header)) != ESP_OK) {
        Serial.println("ERR ASSET_HEADER");
        return false;
    }
    uint32_t magic = 0;
    uint16_t version = 0;
    uint16_t count = 0;
    uint16_t width = 0;
    uint16_t height = 0;
    memcpy(&magic, header + 0, 4);
    memcpy(&version, header + 4, 2);
    memcpy(&count, header + 6, 2);
    memcpy(&width, header + 8, 2);
    memcpy(&height, header + 10, 2);
    if (magic != kAssetMagic || version != 1) {
        Serial.println("ERR ASSET_MAGIC");
        return false;
    }
    if (count != portrait_count) {
        Serial.printf("ERR ASSET_COUNT have=%u want=%u\n", static_cast<unsigned>(count),
                      static_cast<unsigned>(portrait_count));
        return false;
    }
    // 描述符与像素都放静态/PSRAM，生命周期覆盖整个运行期。
    static lv_img_dsc_t descriptors[16] = {};
    for (size_t index = 0; index < portrait_count; ++index) {
        AssetIndex entry{};
        const uint32_t entry_offset = sizeof(header) + sizeof(AssetIndex) * index;
        if (esp_partition_read(partition, entry_offset, &entry, sizeof(entry)) != ESP_OK) {
            Serial.println("ERR ASSET_INDEX");
            return false;
        }
        if (strncmp(entry.name, portraits[index].id, sizeof(entry.name)) != 0) {
            Serial.printf("ERR ASSET_NAME idx=%u\n", static_cast<unsigned>(index));
            return false;
        }
        if (entry.length != static_cast<uint32_t>(width) * height * 2) {
            Serial.printf("ERR ASSET_LENGTH idx=%u len=%u\n", static_cast<unsigned>(index),
                          static_cast<unsigned>(entry.length));
            return false;
        }
        void *pixels = heap_caps_malloc(entry.length, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT);
        if (pixels == nullptr) {
            Serial.println("ERR ASSET_PSRAM");
            return false;
        }
        if (esp_partition_read(partition, entry.offset, pixels, entry.length) != ESP_OK) {
            Serial.println("ERR ASSET_READ");
            return false;
        }
        descriptors[index].header.cf = LV_IMG_CF_TRUE_COLOR;
        descriptors[index].header.w = width;
        descriptors[index].header.h = height;
        descriptors[index].data_size = entry.length;
        descriptors[index].data = static_cast<const uint8_t *>(pixels);
        portraits[index].image = &descriptors[index];
    }
    Serial.printf("ASSETS external=%u w=%u h=%u bytes=%u\n", static_cast<unsigned>(count),
                  static_cast<unsigned>(width), static_cast<unsigned>(height),
                  static_cast<unsigned>(width) * height * 2 * count);
    return true;
}

}  // namespace
#endif

namespace {

struct TouchEvent {
    size_t portrait;
    uint32_t sequence;
};

lv_obj_t *portrait_image;
lv_obj_t *mood_label;
lv_obj_t *page_label;
// The current portrait and all LVGL objects are accessed under the BSP mutex.
size_t current_portrait = 0;
uint32_t touch_sequence = 0;
QueueHandle_t touch_events;
bool display_ready = false;

constexpr size_t max_line = 48;
char line[max_line + 1];
size_t line_length = 0;
bool discard_line = false;

void text_style(lv_obj_t *label, const lv_font_t *font, uint32_t color) {
    lv_obj_set_style_text_font(label, font, 0);
    lv_obj_set_style_text_color(label, lv_color_hex(color), 0);
}

void show_portrait(size_t index) {
    current_portrait = index;
    lv_img_set_src(portrait_image, portraits[index].image);
    lv_label_set_text(mood_label, portraits[index].label);
    lv_label_set_text_fmt(page_label, "%u / %u", static_cast<unsigned>(index + 1),
                          static_cast<unsigned>(portrait_count));
}

void step_portrait(bool next) {
    const size_t delta = next ? 1 : portrait_count - 1;
    show_portrait((current_portrait + delta) % portrait_count);
}

void on_step(lv_event_t *event) {
    step_portrait(lv_event_get_user_data(event) != nullptr);
    const TouchEvent touch{current_portrait, ++touch_sequence};
    // Keep navigation responsive even with no PC reading serial messages.
    xQueueSend(touch_events, &touch, 0);
}

void add_button(lv_obj_t *screen, const char *text, lv_coord_t x, bool next) {
    static bool next_button;
    lv_obj_t *button = lv_btn_create(screen);
    lv_obj_clear_flag(button, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_size(button, 218, 44);
    lv_obj_set_pos(button, x, 266);
    lv_obj_set_style_radius(button, 9, 0);
    lv_obj_set_style_shadow_width(button, 0, 0);
    lv_obj_set_style_bg_color(button, lv_color_hex(next ? 0xD6C8FF : 0x302742), 0);
    lv_obj_set_style_bg_color(button, lv_color_hex(next ? 0xBEABF8 : 0x4D3F68), LV_STATE_PRESSED);
    lv_obj_add_event_cb(button, on_step, LV_EVENT_CLICKED, next ? &next_button : nullptr);

    lv_obj_t *button_label = lv_label_create(button);
    lv_label_set_text(button_label, text);
    text_style(button_label, &lv_font_montserrat_20, next ? 0x211735 : 0xF7F0FF);
    lv_obj_center(button_label);
}

void create_ui(lv_disp_t *display) {
    lv_obj_t *screen = lv_disp_get_scr_act(display);
    lv_obj_clear_flag(screen, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_style_bg_color(screen, lv_color_hex(portrait_background), 0);

    portrait_image = lv_img_create(screen);
    lv_obj_set_pos(portrait_image, 13, 1);

    lv_obj_t *name = lv_label_create(screen);
    lv_label_set_text(name, "Mumu");
    text_style(name, &lv_font_montserrat_20, 0xC9B7F2);
    lv_obj_set_pos(name, 301, 76);

    mood_label = lv_label_create(screen);
    text_style(mood_label, &lv_font_montserrat_28, 0xFFF5EF);
    lv_obj_set_pos(mood_label, 299, 107);

    page_label = lv_label_create(screen);
    text_style(page_label, &lv_font_montserrat_20, 0xB4A8C8);
    lv_obj_set_pos(page_label, 301, 151);

    add_button(screen, LV_SYMBOL_LEFT "  Prev", 14, false);
    add_button(screen, "Next  " LV_SYMBOL_RIGHT, 248, true);
    show_portrait(0);
}

void report_ready() {
    Serial.println(display_ready ? "READY JC3248W535 1" : "ERR DISPLAY_INIT");
}

// 诊断：打印内部堆 / PSRAM / 最大可用块 / DMA / 任务栈水位。
// 只读，不改变任何运行状态；用于 CP1b 的空间实测。
void report_memory(const char *tag) {
    Serial.printf(
        "MEM %s heap=%u min=%u internal=%u largest=%u dma=%u psram_free=%u psram_total=%u stack_hwm=%u\n",
        tag,
        static_cast<unsigned>(ESP.getFreeHeap()),
        static_cast<unsigned>(ESP.getMinFreeHeap()),
        static_cast<unsigned>(heap_caps_get_free_size(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT)),
        static_cast<unsigned>(heap_caps_get_largest_free_block(MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT)),
        static_cast<unsigned>(heap_caps_get_free_size(MALLOC_CAP_DMA)),
        static_cast<unsigned>(heap_caps_get_free_size(MALLOC_CAP_SPIRAM)),
        static_cast<unsigned>(ESP.getPsramSize()),
        static_cast<unsigned>(uxTaskGetStackHighWaterMark(nullptr)));
}

void handle_command(const char *command) {
    if (strcmp(command, "PING") == 0) {
        report_ready();
        return;
    }

    // 帧协议开关：开启后串口进入帧模式，ASCII 调试暂停；
    // 帧模式下输入 "WIRE off" 可随时退回（见 read_commands 的转义检测）。
    if (strcmp(command, "WIRE on") == 0) {
        gunspirit::wire::begin();
        return;
    }
    if (strcmp(command, "WIRE off") == 0) {
        gunspirit::wire::end();
        return;
    }
    // 诊断命令不需要显示就绪：IMU（ICM-45686）。
    // 到货后 5 分钟内确认"接线对不对、芯片真不真、数据有没有"。
    if (strncmp(command, "IMU", 3) == 0) {
        const char *arg = command + 3;
        while (*arg == ' ') {
            ++arg;
        }
        if (*arg == '\0') {
            uint8_t chip = 0;
            if (!gunspirit::imu_init(&chip)) {
                Serial.println("IMU ERR NO_DEVICE (check P3: 1=GND 2=3V3 3=GPIO17 4=GPIO18)");
                return;
            }
            const bool configured = gunspirit::imu_configure();
            Serial.printf("IMU chip=0x%02X expected=0x%02X match=%d configured=%d\n",
                          static_cast<unsigned>(chip), static_cast<unsigned>(0xE9),
                          gunspirit::imu_is_expected_chip(chip) ? 1 : 0, configured ? 1 : 0);
            const gunspirit::ImuReading reading = gunspirit::imu_read();
            if (!reading.valid) {
                Serial.println("IMU ERR READ");
                return;
            }
            Serial.printf("IMU raw  accel=[%d,%d,%d] gyro=[%d,%d,%d]\n",
                          reading.accel_raw[0], reading.accel_raw[1], reading.accel_raw[2],
                          reading.gyro_raw[0], reading.gyro_raw[1], reading.gyro_raw[2]);
            Serial.printf("IMU eng  accel_g=[%.3f,%.3f,%.3f] gyro_dps=[%.2f,%.2f,%.2f]\n",
                          reading.accel_g[0], reading.accel_g[1], reading.accel_g[2],
                          reading.gyro_dps[0], reading.gyro_dps[1], reading.gyro_dps[2]);
            return;
        }
        if (strncmp(arg, "RD ", 3) == 0) {
            uint8_t chip = 0;
            gunspirit::imu_init(&chip);
            uint8_t value = 0;
            const unsigned reg = strtoul(arg + 3, nullptr, 16);
            if (!gunspirit::imu_peek(static_cast<uint8_t>(reg), &value)) {
                Serial.println("IMU ERR RD");
                return;
            }
            Serial.printf("IMU RD 0x%02X = 0x%02X (%u)\n", reg & 0xFF,
                          static_cast<unsigned>(value), static_cast<unsigned>(value));
            return;
        }
        if (strncmp(arg, "WR ", 3) == 0) {
            char *end = nullptr;
            const unsigned reg = strtoul(arg + 3, &end, 16);
            const unsigned value = (end != nullptr) ? strtoul(end, nullptr, 16) : 0;
            uint8_t chip = 0;
            gunspirit::imu_init(&chip);
            const bool ok = gunspirit::imu_poke(static_cast<uint8_t>(reg), static_cast<uint8_t>(value));
            Serial.printf("IMU WR 0x%02X <- 0x%02X %s\n", reg & 0xFF, value & 0xFF, ok ? "ok" : "fail");
            return;
        }
        Serial.println("ERR IMU: (none) | RD <hexreg> | WR <hexreg> <hexval>");
        return;
    }

    // 诊断命令不需要显示就绪
    if (strcmp(command, "MEM") == 0) {
        report_memory("idle");
        return;
    }
#ifdef DIAG_ANY_RADIO
    if (strncmp(command, "RADIO ", 6) == 0) {
        const char *arg = command + 6;
#ifdef DIAG_WIFI
        if (strncmp(arg, "WIFI", 4) == 0) {
            const char *p = arg + 4;
            while (*p == ' ') {
                ++p;
            }
            const char *separator = strchr(p, ' ');
            if (separator == nullptr) {
                Serial.println("ERR ARGS need: RADIO WIFI <ssid> <pass>");
                return;
            }
            char ssid[33] = {0};
            char pass[65] = {0};
            size_t ssid_length = static_cast<size_t>(separator - p);
            if (ssid_length > 32) {
                ssid_length = 32;
            }
            memcpy(ssid, p, ssid_length);
            strncpy(pass, separator + 1, sizeof(pass) - 1);
            Serial.println("RADIO WIFI connecting");
            WiFi.mode(WIFI_STA);
            WiFi.begin(ssid, pass);
            const unsigned deadline = millis() + 20000;
            while (WiFi.status() != WL_CONNECTED && millis() < deadline) {
                delay(100);
            }
            const bool connected = WiFi.status() == WL_CONNECTED;
            Serial.printf("RADIO WIFI connected=%d status=%d\n", connected ? 1 : 0,
                          static_cast<int>(WiFi.status()));
            report_memory(connected ? "after_wifi_ok" : "after_wifi_fail");
            return;
        }
#endif
#ifdef DIAG_BLE
        if (strcmp(arg, "BLE") == 0) {
            BLEDevice::init("GS-DIAG");
            BLEDevice::createServer();
            BLEAdvertising *advertising = BLEDevice::getAdvertising();
            advertising->start();
            Serial.println("RADIO BLE advertising as GS-DIAG");
            report_memory("after_ble");
            return;
        }
#endif
#ifdef DIAG_WIFI
        // 不提供凭据，只把 Wi-Fi STA 协议栈初始化起来，用于测量协议栈本身的 RAM 代价。
        // 注意必须用 "STA" 而不是 "WIFISTA"：后者会被上面的 "WIFI" 前缀分支截获。
        if (strcmp(arg, "STA") == 0) {
            WiFi.mode(WIFI_STA);
            WiFi.disconnect();
            Serial.println("RADIO WIFISTA initialized (no credentials)");
            report_memory("after_wifista");
            return;
        }
        if (strcmp(arg, "OFF") == 0) {
            WiFi.disconnect(true);
            WiFi.mode(WIFI_OFF);
            report_memory("after_wifi_off");
            return;
        }
#endif
        Serial.println("ERR RADIO subcommand unsupported in this build");
        return;
    }
#endif

    if (!display_ready) {
        Serial.println("ERR DISPLAY_INIT");
        return;
    }
    if (!bsp_display_lock(250)) {
        Serial.println("ERR DISPLAY_BUSY");
        return;
    }
    const char *reply = nullptr;
    const char *id = nullptr;
    if (strncmp(command, "STATE ", 6) == 0) {
        const char *requested = command + 6;
        const char *mood = requested;
        if (strcmp(requested, "idle") == 0) {
            mood = "neutral";
        } else if (strcmp(requested, "curious") == 0) {
            mood = "confused";
        }
        for (size_t i = 0; i < portrait_count; ++i) {
            if (strcmp(mood, portraits[i].id) == 0) {
                show_portrait(i);
                reply = "STATE";
                id = requested;
                break;
            }
        }
    } else if (strcmp(command, "GET STATE") == 0) {
        reply = "CURRENT";
        id = portraits[current_portrait].id;
    } else if (strcmp(command, "STEP next") == 0 || strcmp(command, "STEP prev") == 0) {
        step_portrait(command[5] == 'n');
        reply = "STEP";
        id = portraits[current_portrait].id;
    }
    bsp_display_unlock();
    if (reply) {
        Serial.printf("OK %s %s\n", reply, id);
    } else {
        Serial.println("ERR BAD_COMMAND");
    }
}

void read_commands() {
    if (gunspirit::wire::active()) {
        // 帧模式：字节交给协议解析器；同时保留 "WIRE off" 转义，
        // 否则进入帧模式后将无法再用 ASCII 退出。
        static char escape[9] = {0};
        for (int budget = 256; budget > 0 && Serial.available() > 0; --budget) {
            const int byte = Serial.read();
            if (byte < 0) {
                break;
            }
            memmove(escape, &escape[1], 8);
            escape[8] = static_cast<char>(byte);
            if (memcmp(escape, "WIRE off", 8) == 0) {
                escape[0] = '\0';
                gunspirit::wire::end();
                return;
            }
            gunspirit::wire::feed(static_cast<uint8_t>(byte));
        }
        return;
    }
    // Cap work per pass so incoming bytes cannot starve touch reporting.
    for (int budget = 128; budget > 0 && Serial.available() > 0; --budget) {
        const int byte = Serial.read();
        if (byte == '\n') {
            if (discard_line) {
                Serial.println("ERR BAD_FRAME");
            } else {
                if (line_length > 0 && line[line_length - 1] == '\r') {
                    --line_length;
                }
                if (line_length > 0) {
                    line[line_length] = '\0';
                    handle_command(line);
                }
            }
            line_length = 0;
            discard_line = false;
        } else if (!discard_line) {
            const bool printable = byte >= 0x20 && byte <= 0x7e;
            const bool follows_cr = line_length > 0 && line[line_length - 1] == '\r';
            if (line_length == max_line || follows_cr || (!printable && byte != '\r')) {
                discard_line = true;
            } else {
                line[line_length++] = static_cast<char>(byte);
            }
        }
    }
}

}  // namespace

namespace gunspirit::wire {
// 显示层实现：把表情 id 映射到立绘。找不到返回 false，由协议层回 NACK。
bool apply_mood(const char *id) {
    if (id == nullptr) {
        return false;
    }
    for (size_t index = 0; index < portrait_count; ++index) {
        if (strcmp(id, portraits[index].id) == 0) {
            show_portrait(index);
            return true;
        }
    }
    return false;
}
}  // namespace gunspirit::wire
void setup() {
    Serial.begin(115200);
    report_memory("boot");
#ifdef USE_BLOB_ASSETS
    if (!load_assets_from_flash()) {
        Serial.println("ERR ASSETS_UNAVAILABLE");
    }
    report_memory("assets_loaded");
#endif
    touch_events = xQueueCreate(16, sizeof(TouchEvent));
    if (!touch_events) {
        report_ready();
        return;
    }

    const bsp_display_cfg_t config = {
        .lvgl_port_cfg = ESP_LVGL_PORT_INIT_CONFIG(),
        .buffer_size = EXAMPLE_LCD_QSPI_H_RES * EXAMPLE_LCD_QSPI_V_RES,
        .rotate = LV_DISP_ROT_90,
    };
    lv_disp_t *display = bsp_display_start_with_config(&config);
    if (display && bsp_display_lock(1000)) {
        create_ui(display);
        bsp_display_unlock();
        display_ready = bsp_display_backlight_on() == ESP_OK;
    }
    report_ready();
    report_memory(display_ready ? "display_ready" : "display_failed");
}

void loop() {
    read_commands();
    gunspirit::wire::pump(millis());
    TouchEvent touch;
    if (touch_events && xQueueReceive(touch_events, &touch, 0) == pdTRUE && Serial) {
        Serial.printf("TOUCH %" PRIu32 "\nMOOD %s\n", touch.sequence, portraits[touch.portrait].id);
    }
    delay(5);
}




