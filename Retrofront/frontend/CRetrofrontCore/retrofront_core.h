#ifndef RETROFRONT_CORE_H
#define RETROFRONT_CORE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RfFrontend RfFrontend;

typedef struct RfSystemInfo {
    const char *library_name;
    const char *library_version;
    const char *valid_extensions;
    bool need_fullpath;
    bool block_extract;
} RfSystemInfo;

typedef struct RfEvent {
    uint32_t kind;
    uint64_t a;
    uint64_t b;
    uint64_t c;
} RfEvent;

typedef struct RfVideoFrameInfo {
    uint32_t width;
    uint32_t height;
    uint64_t pitch;
    uint64_t rgba_len;
    uint32_t pixel_format;
    uint64_t frame_number;
} RfVideoFrameInfo;

RfFrontend *rf_frontend_create(void);
void rf_frontend_destroy(RfFrontend *frontend);
uint32_t rf_frontend_state(const RfFrontend *frontend);
bool rf_frontend_load_core(RfFrontend *frontend, const char *path);
bool rf_frontend_load_game(RfFrontend *frontend, const char *path, const char *meta);
bool rf_frontend_run_frame(RfFrontend *frontend);
void rf_frontend_unload_game(RfFrontend *frontend);
bool rf_frontend_set_gfx_backend(RfFrontend *frontend, uint32_t backend);
bool rf_frontend_video_frame_info(const RfFrontend *frontend, RfVideoFrameInfo *out_info);
uintptr_t rf_frontend_copy_video_frame_rgba(const RfFrontend *frontend, uint8_t *out_rgba, uintptr_t out_len);
void rf_frontend_opengl_shader_sources(const char **vertex_out, const char **fragment_out);
bool rf_frontend_system_info(const RfFrontend *frontend, RfSystemInfo *out_info);
bool rf_frontend_next_event(RfFrontend *frontend, RfEvent *out_event);
const char *rf_frontend_last_error(const RfFrontend *frontend);

#ifdef __cplusplus
}
#endif

#endif /* RETROFRONT_CORE_H */
