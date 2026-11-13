#pragma once

#include <lvgl.h>

// Canvas color used when compositing the source PNG alpha channel.
constexpr uint32_t portrait_background = 0x141321;

struct Portrait {
    const char *id;
    const char *label;
    const lv_img_dsc_t *image;
};

extern const Portrait portraits[];
extern const size_t portrait_count;
