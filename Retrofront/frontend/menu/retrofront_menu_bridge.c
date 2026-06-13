#include "retrofront_menu_bridge.h"

#include <string.h>

/*
 * Retrofront keeps the upstream RetroArch menu driver sources under
 * Retrofront/frontend/menu/drivers.  Those files are intentionally not pasted
 * into Rust: this tiny C bridge is compiled into retrofront-core and makes the
 * driver-specific geometry/theme contract visible to Rust so Rust can dispatch
 * the correct Ozone/XMB/MaterialUI/RGUI presentation instead of only changing
 * colours.  It also describes each driver's asset/font dependency roots
 * exactly as the C menu code expects them after assets.zip is unpacked.
 */

enum
{
   RETROFRONT_MENU_LAYOUT_OZONE      = 1,
   RETROFRONT_MENU_LAYOUT_XMB        = 2,
   RETROFRONT_MENU_LAYOUT_MATERIALUI = 3,
   RETROFRONT_MENU_LAYOUT_RGUI       = 4
};

static const retrofront_menu_driver_descriptor_t retrofront_menu_drivers[] = {
   { "ozone",      "drivers/ozone.c",      RETROFRONT_MENU_LAYOUT_OZONE,      0x00adefff, 0x101820ff, 50, 46, 408, 320, "assets/ozone",      "assets/pkg/apple", "ozone" },
   { "xmb",        "drivers/xmb.c",        RETROFRONT_MENU_LAYOUT_XMB,        0xffcc00ff, 0x111128ff, 64, 54, 260, 384, "assets/xmb",        "assets/pkg/apple", "xmb" },
   { "materialui", "drivers/materialui.c", RETROFRONT_MENU_LAYOUT_MATERIALUI, 0x2196f3ff, 0x202124ff, 56, 40,   0, 256, "assets/glui",       "assets/pkg/apple", "materialui" },
   { "rgui",       "drivers/rgui.c",       RETROFRONT_MENU_LAYOUT_RGUI,       0x00ff00ff, 0x000000ff, 16,  8,   0,   0, "assets/rgui",       "assets/pkg/apple", "rgui" }
};

uint32_t retrofront_c_menu_driver_count(void)
{
   return (uint32_t)(sizeof(retrofront_menu_drivers) / sizeof(retrofront_menu_drivers[0]));
}

const retrofront_menu_driver_descriptor_t *retrofront_c_menu_driver_by_index(uint32_t index)
{
   if (index >= retrofront_c_menu_driver_count())
      return 0;
   return &retrofront_menu_drivers[index];
}

const retrofront_menu_driver_descriptor_t *retrofront_c_menu_driver_by_name(const char *name)
{
   uint32_t i;

   if (!name)
      return 0;

   for (i = 0; i < retrofront_c_menu_driver_count(); i++)
      if (strcmp(name, retrofront_menu_drivers[i].name) == 0)
         return &retrofront_menu_drivers[i];

   return 0;
}
