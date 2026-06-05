#ifndef RETRO_FRONT_LIBRETRO_HOST_H
#define RETRO_FRONT_LIBRETRO_HOST_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "../../../Externals/libretro-common/include/libretro.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RFCoreHandle RFCoreHandle;

typedef struct RFSystemInfo {
    const char *library_name;
    const char *library_version;
    const char *valid_extensions;
    bool need_fullpath;
    bool block_extract;
} RFSystemInfo;

typedef struct RFAVInfo {
    unsigned geometry_base_width;
    unsigned geometry_base_height;
    unsigned geometry_max_width;
    unsigned geometry_max_height;
    float geometry_aspect_ratio;
    double timing_fps;
    double timing_sample_rate;
} RFAVInfo;

typedef struct RFFrameBuffer {
    const void *data;
    unsigned width;
    unsigned height;
    size_t pitch;
    unsigned pixel_format;
} RFFrameBuffer;

typedef void (*RFVideoFrameCallback)(const RFFrameBuffer *frame, void *context);
typedef void (*RFAudioBatchCallback)(const int16_t *data, size_t frames, void *context);
typedef int16_t (*RFInputStateCallback)(unsigned port, unsigned device, unsigned index, unsigned id, void *context);
typedef void (*RFLogCallback)(const char *message, void *context);

RFCoreHandle *rf_core_open(const char *path, RFLogCallback log, void *context);
void rf_core_close(RFCoreHandle *handle);
bool rf_core_is_open(RFCoreHandle *handle);
bool rf_core_init(RFCoreHandle *handle);
void rf_core_deinit(RFCoreHandle *handle);
RFSystemInfo rf_core_get_system_info(RFCoreHandle *handle);
RFAVInfo rf_core_get_av_info(RFCoreHandle *handle);
bool rf_core_load_game(RFCoreHandle *handle, const char *path, const void *data, size_t size);
void rf_core_unload_game(RFCoreHandle *handle);
void rf_core_run(RFCoreHandle *handle);
size_t rf_core_serialize_size(RFCoreHandle *handle);
bool rf_core_serialize(RFCoreHandle *handle, void *data, size_t size);
bool rf_core_unserialize(RFCoreHandle *handle, const void *data, size_t size);
void rf_core_reset(RFCoreHandle *handle);
void rf_core_set_callbacks(RFCoreHandle *handle,
                           RFVideoFrameCallback video,
                           RFAudioBatchCallback audio,
                           RFInputStateCallback input,
                           void *context);
const char *rf_core_last_error(RFCoreHandle *handle);

#ifdef __cplusplus
}
#endif

#endif
