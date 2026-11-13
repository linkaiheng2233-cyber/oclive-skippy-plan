#pragma once

#include <lvgl.h>

// Canvas color used when compositing the source PNG alpha channel.
constexpr uint32_t portrait_background = 0x141321;

struct Portrait {
    const char *id;
    const char *label;
    const lv_img_dsc_t *image;
};

// 外部资产模式下像素不编进固件，启动时从 ffat 资产区读入 PSRAM 并回填指针，
// 因此该数组必须可写；内嵌模式保持 const。
#ifdef USE_BLOB_ASSETS
extern Portrait portraits[];
#else
extern const Portrait portraits[];
#endif
extern const size_t portrait_count;
