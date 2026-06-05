#include "RetroFrontLibretroHost.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <windows.h>
#else
#include <dlfcn.h>
#endif

typedef struct RFVariableEntry {
    char *key;
    char *description;
    char *values;
    char *value;
} RFVariableEntry;

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
    void (*retro_cheat_reset)(void);
    void (*retro_cheat_set)(unsigned index, bool enabled, const char *code);
    RFVideoFrameCallback video;
    RFAudioBatchCallback audio;
    RFInputStateCallback input;
    RFLogCallback log;
    void *context;
    char *system_directory;
    char *save_directory;
    char *content_directory;
    RFVariableEntry variables[256];
    size_t variable_count;
    char last_error[512];
    unsigned pixel_format;
};

static RFCoreHandle *active_handle;

static char *rf_strdup(const char *value) {
    if (!value) return NULL;
    size_t length = strlen(value) + 1;
    char *copy = (char *)malloc(length);
    if (copy) memcpy(copy, value, length);
    return copy;
}

static RFVariableEntry *rf_find_variable(RFCoreHandle *h, const char *key) {
    if (!h || !key) return NULL;
    for (size_t i = 0; i < h->variable_count; i++) {
        if (h->variables[i].key && strcmp(h->variables[i].key, key) == 0) return &h->variables[i];
    }
    return NULL;
}

static RFVariableEntry *rf_get_or_create_variable(RFCoreHandle *h, const char *key) {
    RFVariableEntry *existing = rf_find_variable(h, key);
    if (existing) return existing;
    if (!h || !key || h->variable_count >= (sizeof(h->variables) / sizeof(h->variables[0]))) return NULL;
    RFVariableEntry *entry = &h->variables[h->variable_count++];
    memset(entry, 0, sizeof(*entry));
    entry->key = rf_strdup(key);
    return entry;
}

static void rf_replace_string(char **slot, const char *value) {
    if (*slot) free(*slot);
    *slot = rf_strdup(value);
}

static void rf_register_legacy_variable(RFCoreHandle *h, const struct retro_variable *variable) {
    if (!h || !variable || !variable->key || !variable->value) return;
    RFVariableEntry *entry = rf_get_or_create_variable(h, variable->key);
    if (!entry) return;
    const char *separator = strstr(variable->value, ";");
    if (separator) {
        size_t description_length = (size_t)(separator - variable->value);
        char *description = (char *)calloc(description_length + 1, 1);
        if (description) {
            memcpy(description, variable->value, description_length);
            rf_replace_string(&entry->description, description);
            free(description);
        }
        const char *values = separator + 1;
        while (*values == ' ') values++;
        rf_replace_string(&entry->values, values);
        if (!entry->value || !entry->value[0]) {
            char *first = rf_strdup(values);
            if (first) {
                char *pipe = strchr(first, '|');
                if (pipe) *pipe = '\0';
                rf_replace_string(&entry->value, first);
                free(first);
            }
        }
    } else {
        rf_replace_string(&entry->description, variable->value);
    }
}

static void rf_register_core_options_v1(RFCoreHandle *h, const struct retro_core_option_definition *option) {
    if (!h || !option || !option->key) return;
    RFVariableEntry *entry = rf_get_or_create_variable(h, option->key);
    if (!entry) return;
    rf_replace_string(&entry->description, option->desc ? option->desc : option->key);
    size_t total = 1;
    for (size_t i = 0; i < 128 && option->values[i].value; i++) total += strlen(option->values[i].value) + 1;
    char *values = (char *)calloc(total, 1);
    if (values) {
        for (size_t i = 0; i < 128 && option->values[i].value; i++) {
            if (i > 0) strcat(values, "|");
            strcat(values, option->values[i].value);
        }
        rf_replace_string(&entry->values, values);
        free(values);
    }
    rf_replace_string(&entry->value, option->default_value ? option->default_value : (option->values[0].value ? option->values[0].value : ""));
}

static bool rf_get_variable_value(RFCoreHandle *h, struct retro_variable *variable) {
    if (!h || !variable || !variable->key) return false;
    RFVariableEntry *entry = rf_find_variable(h, variable->key);
    if (!entry || !entry->value) return false;
    variable->value = entry->value;
    return true;
}

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


static void *rf_optional_symbol(RFCoreHandle *h, const char *name) {
#if defined(_WIN32)
    return (void *)GetProcAddress(h->dylib, name);
#else
    return dlsym(h->dylib, name);
#endif
}

static bool rf_environment(unsigned cmd, void *data) {
    RFCoreHandle *h = active_handle;
    switch (cmd) {
        case RETRO_ENVIRONMENT_SET_PIXEL_FORMAT:
            if (h && data) h->pixel_format = *(const unsigned *)data;
            return true;
        case RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS:
        case RETRO_ENVIRONMENT_SET_CONTROLLER_INFO:
        case RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME:
            return true;
        case RETRO_ENVIRONMENT_SET_VARIABLES: {
            const struct retro_variable *variables = (const struct retro_variable *)data;
            if (!variables) return true;
            for (size_t i = 0; variables[i].key; i++) rf_register_legacy_variable(h, &variables[i]);
            return true;
        }
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS: {
            const struct retro_core_option_definition *options = (const struct retro_core_option_definition *)data;
            if (!options) return true;
            for (size_t i = 0; options[i].key; i++) rf_register_core_options_v1(h, &options[i]);
            return true;
        }
        case RETRO_ENVIRONMENT_GET_VARIABLE:
            return rf_get_variable_value(h, (struct retro_variable *)data);
        case RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE:
            *(bool *)data = false;
            return true;
        case RETRO_ENVIRONMENT_SET_MESSAGE: {
            const struct retro_message *message = (const struct retro_message *)data;
            if (h && h->log && message && message->msg) h->log(message->msg, h->context);
            return true;
        }
        case RETRO_ENVIRONMENT_SET_MESSAGE_EXT: {
            const struct retro_message_ext *message = (const struct retro_message_ext *)data;
            if (h && h->log && message && message->msg) h->log(message->msg, h->context);
            return true;
        }
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL:
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2:
        case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL:
            return true;
        case RETRO_ENVIRONMENT_GET_CAN_DUPE:
            *(bool *)data = true;
            return true;
        case RETRO_ENVIRONMENT_GET_LOG_INTERFACE:
            return false;
        case RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY:
            *(const char **)data = h ? h->save_directory : NULL;
            return true;
        case RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY:
            *(const char **)data = h ? h->system_directory : NULL;
            return true;
        case RETRO_ENVIRONMENT_GET_CONTENT_DIRECTORY:
            *(const char **)data = h ? h->content_directory : NULL;
            return true;
        default:
            return false;
    }
}

static void rf_video(const void *data, unsigned width, unsigned height, size_t pitch) {
    if (!active_handle || !active_handle->video) return;
    RFFrameBuffer frame = { data, width, height, pitch, active_handle->pixel_format };
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
    h->retro_cheat_reset = rf_optional_symbol(h, "retro_cheat_reset");
    h->retro_cheat_set = rf_optional_symbol(h, "retro_cheat_set");
    return h;
}

void rf_core_close(RFCoreHandle *h) {
    if (!h) return;
    for (size_t i = 0; i < h->variable_count; i++) {
        free(h->variables[i].key);
        free(h->variables[i].description);
        free(h->variables[i].values);
        free(h->variables[i].value);
    }
    free(h->system_directory);
    free(h->save_directory);
    free(h->content_directory);
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

void rf_core_set_directories(RFCoreHandle *h, const char *system_directory, const char *save_directory, const char *content_directory) {
    if (!h) return;
    rf_replace_string(&h->system_directory, system_directory);
    rf_replace_string(&h->save_directory, save_directory);
    rf_replace_string(&h->content_directory, content_directory);
}

void rf_core_set_variable(RFCoreHandle *h, const char *key, const char *value) {
    RFVariableEntry *entry = rf_get_or_create_variable(h, key);
    if (entry) rf_replace_string(&entry->value, value);
}

size_t rf_core_variable_count(RFCoreHandle *h) { return h ? h->variable_count : 0; }

RFCoreVariable rf_core_get_variable(RFCoreHandle *h, size_t index) {
    RFCoreVariable out = {0};
    if (!h || index >= h->variable_count) return out;
    RFVariableEntry *entry = &h->variables[index];
    out.key = entry->key;
    out.description = entry->description;
    out.values = entry->values;
    out.value = entry->value;
    return out;
}

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
void rf_core_cheat_reset(RFCoreHandle *h) { if (rf_core_is_open(h) && h->retro_cheat_reset) h->retro_cheat_reset(); }
void rf_core_set_cheat(RFCoreHandle *h, unsigned index, bool enabled, const char *code) { if (rf_core_is_open(h) && h->retro_cheat_set) h->retro_cheat_set(index, enabled, code); }
void rf_core_set_callbacks(RFCoreHandle *h, RFVideoFrameCallback video, RFAudioBatchCallback audio, RFInputStateCallback input, void *context) {
    if (!h) return;
    h->video = video; h->audio = audio; h->input = input; h->context = context;
}
const char *rf_core_last_error(RFCoreHandle *h) { return h ? h->last_error : "No core handle"; }
