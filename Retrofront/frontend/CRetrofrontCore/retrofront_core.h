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

typedef struct RfOpenGlRenderCommand {
    uint64_t native_view;
    uint64_t gl_context;
    uintptr_t framebuffer;
    int32_t viewport[4];
    uint32_t texture_size[2];
    bool source_is_hardware;
    bool bottom_left_origin;
    float clear_color[4];
    const char *vertex_shader;
    const char *fragment_shader;
} RfOpenGlRenderCommand;

typedef struct RfVulkanRenderCommand {
    uint64_t native_view;
    uint64_t instance;
    uint64_t device;
    uint64_t queue;
    uint64_t command_buffer;
    uint64_t image;
    uint32_t extent[2];
    bool source_is_hardware;
    bool uses_moltenvk;
    float clear_color[4];
} RfVulkanRenderCommand;

typedef bool (*RfOpenGlRenderCallback)(const RfOpenGlRenderCommand *command, const uint8_t *rgba, uintptr_t rgba_len, void *user_data);
typedef bool (*RfVulkanRenderCallback)(const RfVulkanRenderCommand *command, const uint8_t *rgba, uintptr_t rgba_len, void *user_data);
typedef void (*RfRetroProcAddress)(void);
typedef RfRetroProcAddress (*RfGetProcAddressCallback)(const char *symbol, void *user_data);

typedef struct RfGfxHostHandles {
    uint64_t native_view;
    uint64_t gl_context;
    uintptr_t gl_framebuffer;
    uint64_t vulkan_instance;
    uint64_t vulkan_device;
    uint64_t vulkan_queue;
    uint64_t vulkan_command_buffer;
    uint64_t vulkan_image;
    RfOpenGlRenderCallback opengl_render;
    RfVulkanRenderCallback vulkan_render;
    RfGetProcAddressCallback get_proc_address;
    void *user_data;
} RfGfxHostHandles;

typedef struct RfGfxDriverInfo {
    uint32_t backend;
    uint64_t frame_number;
    bool hardware_ready;
    bool rendered;
} RfGfxDriverInfo;

RfFrontend *rf_frontend_create(void);
void rf_frontend_destroy(RfFrontend *frontend);
uint32_t rf_frontend_state(const RfFrontend *frontend);
bool rf_frontend_load_core(RfFrontend *frontend, const char *path);
bool rf_frontend_load_game(RfFrontend *frontend, const char *path, const char *meta);
bool rf_frontend_run_frame(RfFrontend *frontend);
void rf_frontend_unload_game(RfFrontend *frontend);
bool rf_frontend_set_gfx_backend(RfFrontend *frontend, uint32_t backend);
bool rf_frontend_set_gfx_host_handles(RfFrontend *frontend, const RfGfxHostHandles *handles);
bool rf_frontend_gfx_driver_info(const RfFrontend *frontend, RfGfxDriverInfo *out_info);
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
