#include <Arduino.h>
#include <freertos/FreeRTOS.h>
#include <freertos/queue.h>
#include <inttypes.h>
#include <string.h>

#include "display.h"
#include "esp_bsp.h"
#include "portrait_assets.h"

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

void handle_command(const char *command) {
    if (strcmp(command, "PING") == 0) {
        report_ready();
        return;
    }

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

void setup() {
    Serial.begin(115200);
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
}

void loop() {
    read_commands();
    TouchEvent touch;
    if (touch_events && xQueueReceive(touch_events, &touch, 0) == pdTRUE && Serial) {
        Serial.printf("TOUCH %" PRIu32 "\nMOOD %s\n", touch.sequence, portraits[touch.portrait].id);
    }
    delay(5);
}
