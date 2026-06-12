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
} rf_menu_driver_spec;

uint32_t rf_menu_driver_count(void);
const rf_menu_driver_spec *rf_menu_driver_at(uint32_t index);
const rf_menu_driver_spec *rf_menu_driver_find(const char *ident);
const rf_menu_driver_spec *rf_menu_driver_default(void);
const char *rf_menu_driver_next_ident(const char *ident);
uint32_t rf_menu_driver_is_supported(const char *ident);

#ifdef __cplusplus
}
#endif

#endif
