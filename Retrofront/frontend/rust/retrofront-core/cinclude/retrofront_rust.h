#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct retrofront_menu_entry {
    size_t entry_idx;
    uint32_t entry_type;
    uint32_t flags;
    bool checked;
    char path[1024];
    char label[1024];
    char rich_label[1024];
    char sublabel[1024];
    char value[1024];
} retrofront_menu_entry_t;

#ifdef __cplusplus
extern "C" {
#endif

bool retrofront_runtime_init(const char *data_dir);
void retrofront_runtime_shutdown(void);
uint32_t retrofront_menu_api_version(void);
bool retrofront_menu_set_title(const char *title);
bool retrofront_menu_clear_entries(void);
bool retrofront_menu_append_entry(const char *label, const char *path);
bool retrofront_menu_bootstrap(void);
size_t retrofront_menu_entry_count(void);
size_t retrofront_menu_selected_index(void);
bool retrofront_menu_set_selected_index(size_t index);
bool retrofront_menu_entry(size_t index, retrofront_menu_entry_t *out);
bool retrofront_menu_entry_label(size_t index, char *dst, size_t dst_len);
bool retrofront_menu_entry_sublabel(size_t index, char *dst, size_t dst_len);
bool retrofront_menu_title(char *dst, size_t dst_len);
size_t retrofront_menu_source_file_count(void);
bool retrofront_menu_source_file(size_t index, char *dst, size_t dst_len);
bool retrofront_menu_contract_complete(void);
bool retrofront_menu_source_is_driver(size_t index);
bool retrofront_menu_active_driver_source(char *dst, size_t dst_len);
bool retrofront_menu_set_driver(const char *name);
bool retrofront_menu_driver(char *dst, size_t dst_len);
bool retrofront_menu_draw(void);
bool retrofront_input_bind_key(uint32_t key, uint32_t action);
bool retrofront_input_push_key(uint32_t key, bool pressed);
bool retrofront_input_push_gamepad_button(uint8_t port, uint16_t id, bool pressed);
bool retrofront_input_set_analog(uint8_t port, uint32_t device, uint32_t index, int16_t value);
bool retrofront_menu_pump_input(void);
bool retrofront_menu_action(uint32_t action);
bool retrofront_renderer_resize(uint32_t width, uint32_t height);
bool retrofront_renderer_write_snapshot_ppm(const char *path);
bool retrofront_shader_set_preset(const char *path);
size_t retrofront_resources_unpack(const char *zip_path);
bool retrofront_assets_load_defaults(void);
size_t retrofront_menu_asset_count(void);
uint32_t retrofront_menu_asset_kind(size_t index);
bool retrofront_menu_asset_path(size_t index, char *dst, size_t dst_len);
bool retrofront_menu_asset_driver(size_t index, char *dst, size_t dst_len);
uint32_t retrofront_menu_driver_row_height(void);
uint32_t retrofront_menu_driver_icon_size(void);
uint32_t retrofront_menu_driver_sidebar_width(void);
bool retrofront_import_rom(const char *path, const char *playlist);
bool retrofront_settings_set_string(const char *key, const char *value);
bool retrofront_core_open(const char *core_path);
bool retrofront_core_load_game(const char *game_path);
bool retrofront_core_launch_pending(void);
bool retrofront_core_run_frame(void);
void retrofront_tick(void);

#ifdef __cplusplus
}
#endif
