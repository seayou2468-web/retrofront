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

typedef struct RfBgfxRenderCommand {
    uint64_t native_view;
    uint64_t context;
    uintptr_t framebuffer;
    int32_t viewport[4];
    uint32_t output_size[2];
    uint32_t texture_size[2];
    bool source_is_hardware;
    bool bottom_left_origin;
    uint32_t rotation_quarters;
    uint32_t scale_mode;
    uint32_t filter_mode;
    bool vsync;
    float clear_color[4];
} RfBgfxRenderCommand;

typedef bool (*RfBgfxRenderCallback)(const RfBgfxRenderCommand *command, const uint8_t *rgba, uintptr_t rgba_len, void *user_data);
typedef void (*RfRetroProcAddress)(void);
typedef RfRetroProcAddress (*RfGetProcAddressCallback)(const char *symbol, void *user_data);

typedef struct RfGfxVideoConfig {
    uint32_t base_width;
    uint32_t base_height;
    uint32_t max_width;
    uint32_t max_height;
    float aspect_ratio;
    uint32_t output_width;
    uint32_t output_height;
    uint32_t scale_mode;
    uint32_t filter_mode;
    uint32_t rotation_quarters;
    bool vsync;
} RfGfxVideoConfig;

typedef struct RfGfxHostHandles {
    uint64_t native_view;
    uint64_t context;
    uintptr_t framebuffer;
    RfBgfxRenderCallback render_callback;
    RfGetProcAddressCallback get_proc_address;
    void *user_data;
} RfGfxHostHandles;

typedef struct RfGfxDriverInfo {
    uint32_t backend;
    uint64_t frame_number;
    bool hardware_ready;
    bool rendered;
} RfGfxDriverInfo;

typedef struct RfCoreOptionValue {
    const char *value;
    const char *label;
} RfCoreOptionValue;

typedef struct RfCoreOption {
    const char *key;
    const char *desc;
    const char *info;
    const char *value;
    const RfCoreOptionValue *values;
    uintptr_t values_count;
} RfCoreOption;

RfFrontend *rf_frontend_create(void);
void rf_frontend_destroy(RfFrontend *frontend);
uint32_t rf_frontend_state(const RfFrontend *frontend);
bool rf_frontend_load_core(RfFrontend *frontend, const char *path);
bool rf_frontend_load_game(RfFrontend *frontend, const char *path, const char *meta);
bool rf_frontend_run_frame(RfFrontend *frontend);
void rf_frontend_unload_game(RfFrontend *frontend);
bool rf_frontend_set_gfx_backend(RfFrontend *frontend, uint32_t backend);
bool rf_frontend_set_gfx_video_config(RfFrontend *frontend, const RfGfxVideoConfig *config);
bool rf_frontend_set_joypad_button(RfFrontend *frontend, uint32_t button_id, bool pressed);
bool rf_frontend_set_gfx_host_handles(RfFrontend *frontend, const RfGfxHostHandles *handles);
bool rf_frontend_gfx_driver_info(const RfFrontend *frontend, RfGfxDriverInfo *out_info);
bool rf_frontend_video_frame_info(const RfFrontend *frontend, RfVideoFrameInfo *out_info);
uintptr_t rf_frontend_copy_video_frame_rgba(const RfFrontend *frontend, uint8_t *out_rgba, uintptr_t out_len);
bool rf_frontend_system_info(const RfFrontend *frontend, RfSystemInfo *out_info);
bool rf_frontend_next_event(RfFrontend *frontend, RfEvent *out_event);
const char *rf_frontend_last_error(const RfFrontend *frontend);

// Core Options API
bool rf_frontend_set_options_config_path(RfFrontend *frontend, const char *path);
uintptr_t rf_frontend_options_count(const RfFrontend *frontend);
bool rf_frontend_get_option(RfFrontend *frontend, uintptr_t index, RfCoreOption *out_option);
bool rf_frontend_set_option(RfFrontend *frontend, const char *key, const char *value);
void rf_frontend_clear_options_cache(RfFrontend *frontend);

#ifdef __cplusplus
}
#endif

#endif /* RETROFRONT_CORE_H */
