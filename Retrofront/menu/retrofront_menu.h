#ifndef RETROFRONT_MENU_H
#define RETROFRONT_MENU_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct rf_menu_driver_spec {
    const char *ident;
    const char *display_name;
    const char *layout_model;
    const char *input_model;
    const char *thumbnail_model;
    const char *animation_model;
    const char *asset_subdirectory;
    const char *default_theme;
} rf_menu_driver_spec;

typedef const char *(*rf_menu_get_setting_fn)(const char *key, void *userdata);
typedef uint32_t (*rf_menu_set_setting_fn)(const char *key, const char *value, void *userdata);
typedef uint32_t (*rf_menu_directory_exists_fn)(const char *path, void *userdata);
typedef uint32_t (*rf_menu_file_exists_fn)(const char *path, void *userdata);

typedef struct rf_menu_host_callbacks {
    rf_menu_get_setting_fn get_setting;
    rf_menu_set_setting_fn set_setting;
    rf_menu_directory_exists_fn directory_exists;
    rf_menu_file_exists_fn file_exists;
    void *userdata;
} rf_menu_host_callbacks;


typedef struct rf_menu_source_file {
    const char *path;
    uint32_t compiled;
} rf_menu_source_file;

typedef struct rf_menu_layout_metrics {
    uint32_t viewport_width;
    uint32_t viewport_height;
    uint32_t content_x;
    uint32_t content_y;
    uint32_t content_width;
    uint32_t content_height;
    uint32_t sidebar_width;
    uint32_t header_height;
    uint32_t footer_height;
    uint32_t row_height;
    uint32_t icon_size;
    uint32_t horizontal_padding;
    uint32_t vertical_padding;
    uint32_t background_mode;
    float scale;
} rf_menu_layout_metrics;

typedef struct rf_menu_runtime_config {
    const rf_menu_driver_spec *driver;
    const char *driver_ident;
    const char *assets_directory;
    const char *theme;
    uint32_t assets_ready;
} rf_menu_runtime_config;

typedef struct rf_menu_resolved_assets {
    const char *root_directory;
    const char *driver_directory;
    const char *icon_directory;
    const char *font_path;
    const char *background_path;
    uint32_t assets_ready;
} rf_menu_resolved_assets;


uint32_t rf_menu_source_file_count(void);
const rf_menu_source_file *rf_menu_source_file_at(uint32_t index);
uint32_t rf_menu_layout_for_viewport(const char *driver_ident, uint32_t viewport_width, uint32_t viewport_height, rf_menu_layout_metrics *out_metrics);
uint32_t rf_menu_asset_path(const char *driver_ident, const char *asset_name, char *out_path, uint32_t out_path_len);
uint32_t rf_menu_resolve_assets(const char *driver_ident, rf_menu_resolved_assets *out_assets);

uint32_t rf_menu_driver_count(void);
const rf_menu_driver_spec *rf_menu_driver_at(uint32_t index);
const rf_menu_driver_spec *rf_menu_driver_find(const char *ident);
const rf_menu_driver_spec *rf_menu_driver_default(void);
const char *rf_menu_driver_next_ident(const char *ident);
uint32_t rf_menu_driver_is_supported(const char *ident);

void rf_menu_connect_host(const rf_menu_host_callbacks *callbacks);
uint32_t rf_menu_get_runtime_config(rf_menu_runtime_config *out_config);
uint32_t rf_menu_set_driver(const char *ident);
const char *rf_menu_setting(const char *key);
uint32_t rf_menu_assets_ready(void);

#ifdef __cplusplus
}
#endif

#endif
