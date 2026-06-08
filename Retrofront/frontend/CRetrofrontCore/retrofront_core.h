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

typedef struct RfCoreInfo {
    const char *path;
    const char *display_name;
    const char *system_name;
    const char *supported_extensions;
} RfCoreInfo;

typedef struct RfMenuEntry {
    const char *label;
    const char *sublabel;
    uint32_t kind;
    const char *value;
    uint32_t action_id;
} RfMenuEntry;

typedef struct RfMenuList {
    const char *title;
    uintptr_t entry_count;
} RfMenuList;

typedef struct RfSettingEntry {
    const char *key;
    const char *value;
} RfSettingEntry;

typedef struct RfLaunchPlan {
    const char *content_path;
    const char *content_extension;
    uint32_t decision;
    const char *selected_core_path;
    uintptr_t candidate_count;
    const char *reason;
} RfLaunchPlan;

RfFrontend *rf_frontend_create(void);
void rf_frontend_destroy(RfFrontend *frontend);
uint32_t rf_frontend_state(const RfFrontend *frontend);
bool rf_frontend_load_core(RfFrontend *frontend, const char *path);
bool rf_frontend_load_game(RfFrontend *frontend, const char *path, const char *meta);
bool rf_frontend_launch_content(RfFrontend *frontend, const char *path, const char *preferred_core, const char *meta);
bool rf_frontend_run_frame(RfFrontend *frontend);
void rf_frontend_unload_game(RfFrontend *frontend);
bool rf_frontend_set_gfx_backend(RfFrontend *frontend, uint32_t backend);
bool rf_frontend_get_gfx_video_config(const RfFrontend *frontend, RfGfxVideoConfig *out_config);
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

// Core Discovery API
void rf_frontend_set_info_dir(RfFrontend *frontend, const char *path);
void rf_frontend_scan_cores(RfFrontend *frontend, const char *cores_dir);
void rf_frontend_scan_configured_cores(RfFrontend *frontend);
const char *rf_frontend_all_extensions(RfFrontend *frontend);
uintptr_t rf_frontend_cores_count(const RfFrontend *frontend);
bool rf_frontend_get_core_info(RfFrontend *frontend, uintptr_t index, RfCoreInfo *out_info);
typedef struct RfGameEntry {
    const char *path;
    const char *label;
} RfGameEntry;

void rf_frontend_scan_games(RfFrontend *frontend, const char *directory, const char *extensions);
uintptr_t rf_frontend_games_count(const RfFrontend *frontend);
bool rf_frontend_get_game_info(RfFrontend *frontend, uintptr_t index, RfGameEntry *out_info);
bool rf_frontend_plan_content_launch(RfFrontend *frontend, const char *path, const char *preferred_core, RfLaunchPlan *out_plan);
uintptr_t rf_frontend_launch_candidate_count(const RfFrontend *frontend);
bool rf_frontend_get_launch_candidate(RfFrontend *frontend, uintptr_t index, RfCoreInfo *out_info);

// Menu Engine API
bool rf_frontend_menu_current_list(RfFrontend *frontend, RfMenuList *out_list);
bool rf_frontend_menu_get_entry(RfFrontend *frontend, uintptr_t index, RfMenuEntry *out_entry);
void rf_frontend_menu_push_core_list(RfFrontend *frontend);
void rf_frontend_menu_push_content_list(RfFrontend *frontend);
void rf_frontend_menu_push_settings(RfFrontend *frontend);
void rf_frontend_menu_push_information(RfFrontend *frontend);
void rf_frontend_menu_push_skin_settings(RfFrontend *frontend);
bool rf_frontend_menu_activate(RfFrontend *frontend, uint32_t action_id);
bool rf_frontend_menu_pop(RfFrontend *frontend);

// RetroArch-style Settings API
bool rf_frontend_load_settings(RfFrontend *frontend, const char *path);
bool rf_frontend_set_base_dir(RfFrontend *frontend, const char *path);
void rf_frontend_save_settings(RfFrontend *frontend);
const char *rf_frontend_get_setting(RfFrontend *frontend, const char *key);
bool rf_frontend_set_setting(RfFrontend *frontend, const char *key, const char *value);
uintptr_t rf_frontend_settings_count(const RfFrontend *frontend);
bool rf_frontend_get_setting_at(RfFrontend *frontend, uintptr_t index, RfSettingEntry *out_setting);

#ifdef __cplusplus
}
#endif

#endif /* RETROFRONT_CORE_H */
