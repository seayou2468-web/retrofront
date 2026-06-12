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

typedef struct rf_menu_host_callbacks {
    rf_menu_get_setting_fn get_setting;
    rf_menu_set_setting_fn set_setting;
    rf_menu_directory_exists_fn directory_exists;
    void *userdata;
} rf_menu_host_callbacks;

typedef struct rf_menu_runtime_config {
    const rf_menu_driver_spec *driver;
    const char *driver_ident;
    const char *assets_directory;
    const char *theme;
    uint32_t assets_ready;
} rf_menu_runtime_config;

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
