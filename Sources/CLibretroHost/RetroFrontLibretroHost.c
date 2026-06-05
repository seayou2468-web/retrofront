#include "RetroFrontLibretroHost.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <windows.h>
#else
#include <dlfcn.h>
#endif

struct RFCoreHandle {
#if defined(_WIN32)
    HMODULE dylib;
#else
    void *dylib;
#endif
    void (*retro_init)(void);
    void (*retro_deinit)(void);
    unsigned (*retro_api_version)(void);
    void (*retro_get_system_info)(struct retro_system_info *info);
    void (*retro_get_system_av_info)(struct retro_system_av_info *info);
    void (*retro_set_environment)(retro_environment_t cb);
    void (*retro_set_video_refresh)(retro_video_refresh_t cb);
    void (*retro_set_audio_sample)(retro_audio_sample_t cb);
    void (*retro_set_audio_sample_batch)(retro_audio_sample_batch_t cb);
    void (*retro_set_input_poll)(retro_input_poll_t cb);
    void (*retro_set_input_state)(retro_input_state_t cb);
    bool (*retro_load_game)(const struct retro_game_info *game);
    void (*retro_unload_game)(void);
    void (*retro_run)(void);
    size_t (*retro_serialize_size)(void);
    bool (*retro_serialize)(void *data, size_t size);
    bool (*retro_unserialize)(const void *data, size_t size);
    void (*retro_reset)(void);
    RFVideoFrameCallback video;
    RFAudioBatchCallback audio;
    RFInputStateCallback input;
    RFLogCallback log;
    void *context;
    char last_error[512];
};

static RFCoreHandle *active_handle;

static void rf_set_error(RFCoreHandle *h, const char *message) {
    if (!h) return;
    snprintf(h->last_error, sizeof(h->last_error), "%s", message ? message : "Unknown libretro error");
    if (h->log) h->log(h->last_error, h->context);
}

static void *rf_symbol(RFCoreHandle *h, const char *name) {
#if defined(_WIN32)
    void *sym = (void *)GetProcAddress(h->dylib, name);
#else
    void *sym = dlsym(h->dylib, name);
#endif
    if (!sym) {
        char buffer[512];
        snprintf(buffer, sizeof(buffer), "Missing libretro symbol: %s", name);
        rf_set_error(h, buffer);
    }
    return sym;
}

static bool rf_environment(unsigned cmd, void *data) {
    switch (cmd) {
        case RETRO_ENVIRONMENT_SET_PIXEL_FORMAT:
        case RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS:
        case RETRO_ENVIRONMENT_SET_CONTROLLER_INFO:
        case RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME:
        case RETRO_ENVIRONMENT_SET_VARIABLES:
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS:
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL:
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2:
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL:
        case RETRO_ENVIRONMENT_SET_MESSAGE:
        case RETRO_ENVIRONMENT_SET_MESSAGE_EXT:
            return true;
        case RETRO_ENVIRONMENT_GET_CAN_DUPE:
            *(bool *)data = true;
            return true;
        case RETRO_ENVIRONMENT_GET_LOG_INTERFACE:
            return false;
        case RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY:
        case RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY:
        case RETRO_ENVIRONMENT_GET_CONTENT_DIRECTORY:
            *(const char **)data = NULL;
            return true;
        default:
            return false;
    }
}

static void rf_video(const void *data, unsigned width, unsigned height, size_t pitch) {
    if (!active_handle || !active_handle->video) return;
    RFFrameBuffer frame = { data, width, height, pitch, 0 };
    active_handle->video(&frame, active_handle->context);
}

static void rf_audio_sample(int16_t left, int16_t right) {
    int16_t stereo[2] = { left, right };
    if (active_handle && active_handle->audio) active_handle->audio(stereo, 1, active_handle->context);
}

static size_t rf_audio_batch(const int16_t *data, size_t frames) {
    if (active_handle && active_handle->audio) active_handle->audio(data, frames, active_handle->context);
    return frames;
}

static void rf_input_poll(void) {}

static int16_t rf_input_state(unsigned port, unsigned device, unsigned index, unsigned id) {
    if (!active_handle || !active_handle->input) return 0;
    return active_handle->input(port, device, index, id, active_handle->context);
}

RFCoreHandle *rf_core_open(const char *path, RFLogCallback log, void *context) {
    RFCoreHandle *h = (RFCoreHandle *)calloc(1, sizeof(RFCoreHandle));
    if (!h) return NULL;
    h->log = log;
    h->context = context;
#if defined(_WIN32)
    h->dylib = LoadLibraryA(path);
#else
    h->dylib = dlopen(path, RTLD_LAZY | RTLD_LOCAL);
#endif
    if (!h->dylib) {
#if defined(_WIN32)
        rf_set_error(h, "Could not open libretro core dynamic library");
#else
        rf_set_error(h, dlerror());
#endif
        return h;
    }
#define LOAD(name) do { h->name = rf_symbol(h, #name); if (!h->name) return h; } while (0)
    LOAD(retro_init); LOAD(retro_deinit); LOAD(retro_api_version); LOAD(retro_get_system_info);
    LOAD(retro_get_system_av_info); LOAD(retro_set_environment); LOAD(retro_set_video_refresh);
    LOAD(retro_set_audio_sample); LOAD(retro_set_audio_sample_batch); LOAD(retro_set_input_poll);
    LOAD(retro_set_input_state); LOAD(retro_load_game); LOAD(retro_unload_game); LOAD(retro_run);
    LOAD(retro_serialize_size); LOAD(retro_serialize); LOAD(retro_unserialize); LOAD(retro_reset);
#undef LOAD
    return h;
}

void rf_core_close(RFCoreHandle *h) {
    if (!h) return;
    if (h->dylib) {
#if defined(_WIN32)
        FreeLibrary(h->dylib);
#else
        dlclose(h->dylib);
#endif
    }
    free(h);
}

bool rf_core_is_open(RFCoreHandle *h) { return h && h->dylib && h->retro_run; }

bool rf_core_init(RFCoreHandle *h) {
    if (!rf_core_is_open(h)) return false;
    active_handle = h;
    h->retro_set_environment(rf_environment);
    h->retro_set_video_refresh(rf_video);
    h->retro_set_audio_sample(rf_audio_sample);
    h->retro_set_audio_sample_batch(rf_audio_batch);
    h->retro_set_input_poll(rf_input_poll);
    h->retro_set_input_state(rf_input_state);
    if (h->retro_api_version() != RETRO_API_VERSION) {
        rf_set_error(h, "Unsupported libretro API version");
        return false;
    }
    h->retro_init();
    return true;
}

void rf_core_deinit(RFCoreHandle *h) { if (rf_core_is_open(h)) h->retro_deinit(); }

RFSystemInfo rf_core_get_system_info(RFCoreHandle *h) {
    RFSystemInfo out = {0};
    if (!rf_core_is_open(h)) return out;
    struct retro_system_info info;
    memset(&info, 0, sizeof(info));
    h->retro_get_system_info(&info);
    out.library_name = info.library_name;
    out.library_version = info.library_version;
    out.valid_extensions = info.valid_extensions;
    out.need_fullpath = info.need_fullpath;
    out.block_extract = info.block_extract;
    return out;
}

RFAVInfo rf_core_get_av_info(RFCoreHandle *h) {
    RFAVInfo out = {0};
    if (!rf_core_is_open(h)) return out;
    struct retro_system_av_info info;
    memset(&info, 0, sizeof(info));
    h->retro_get_system_av_info(&info);
    out.geometry_base_width = info.geometry.base_width;
    out.geometry_base_height = info.geometry.base_height;
    out.geometry_max_width = info.geometry.max_width;
    out.geometry_max_height = info.geometry.max_height;
    out.geometry_aspect_ratio = info.geometry.aspect_ratio;
    out.timing_fps = info.timing.fps;
    out.timing_sample_rate = info.timing.sample_rate;
    return out;
}

bool rf_core_load_game(RFCoreHandle *h, const char *path, const void *data, size_t size) {
    if (!rf_core_is_open(h)) return false;
    struct retro_game_info game;
    memset(&game, 0, sizeof(game));
    game.path = path;
    game.data = data;
    game.size = size;
    return h->retro_load_game(&game);
}

void rf_core_unload_game(RFCoreHandle *h) { if (rf_core_is_open(h)) h->retro_unload_game(); }
void rf_core_run(RFCoreHandle *h) { if (rf_core_is_open(h)) { active_handle = h; h->retro_run(); } }
size_t rf_core_serialize_size(RFCoreHandle *h) { return rf_core_is_open(h) ? h->retro_serialize_size() : 0; }
bool rf_core_serialize(RFCoreHandle *h, void *data, size_t size) { return rf_core_is_open(h) && h->retro_serialize(data, size); }
bool rf_core_unserialize(RFCoreHandle *h, const void *data, size_t size) { return rf_core_is_open(h) && h->retro_unserialize(data, size); }
void rf_core_reset(RFCoreHandle *h) { if (rf_core_is_open(h)) h->retro_reset(); }
void rf_core_set_callbacks(RFCoreHandle *h, RFVideoFrameCallback video, RFAudioBatchCallback audio, RFInputStateCallback input, void *context) {
    if (!h) return;
    h->video = video; h->audio = audio; h->input = input; h->context = context;
}
const char *rf_core_last_error(RFCoreHandle *h) { return h ? h->last_error : "No core handle"; }
