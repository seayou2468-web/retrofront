#ifndef RETROFRONT_MENU_BRIDGE_H
#define RETROFRONT_MENU_BRIDGE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct retrofront_menu_driver_descriptor
{
   const char *name;
   const char *source_file;
   uint32_t layout;
   uint32_t accent_rgba;
   uint32_t background_rgba;
   uint32_t row_height;
   uint32_t icon_size;
   uint32_t sidebar_width;
   uint32_t thumbnail_size;
   const char *asset_dir;
   const char *font_dir;
   const char *dependency_group;
} retrofront_menu_driver_descriptor_t;

uint32_t retrofront_c_menu_driver_count(void);
const retrofront_menu_driver_descriptor_t *retrofront_c_menu_driver_by_index(uint32_t index);
const retrofront_menu_driver_descriptor_t *retrofront_c_menu_driver_by_name(const char *name);

#ifdef __cplusplus
}
#endif

#endif
