#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

bool retrofront_runtime_init(const char *data_dir);
void retrofront_runtime_shutdown(void);
uint32_t retrofront_menu_api_version(void);
bool retrofront_menu_set_title(const char *title);
bool retrofront_menu_clear_entries(void);
bool retrofront_menu_append_entry(const char *label, const char *path);
size_t retrofront_menu_entry_count(void);
size_t retrofront_menu_selected_index(void);
bool retrofront_input_bind_key(uint32_t key, uint32_t action);
bool retrofront_input_push_key(uint32_t key, bool pressed);
bool retrofront_menu_pump_input(void);
bool retrofront_renderer_resize(uint32_t width, uint32_t height);
bool retrofront_shader_set_preset(const char *path);
void retrofront_tick(void);

#ifdef __cplusplus
}
#endif
