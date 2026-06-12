#include "retrofront_menu.h"

#include <ctype.h>
#include <stddef.h>
#include <string.h>

static const rf_menu_driver_spec RF_MENU_DRIVERS[] = {
    {
        "materialui",
        "Material UI",
        "mobile_appbar_navigation",
        "touch_navigation_retropad",
        "responsive_dual_thumbnail_list",
        "material_elevation_ripple",
    },
    {
        "ozone",
        "Ozone",
        "desktop_sidebar_detail",
        "pointer_keyboard_retropad",
        "right_panel_dual_thumbnail",
        "fade_slide_sidebar",
    },
    {
        "xmb",
        "XMB",
        "horizontal_categories_vertical_items",
        "retropad_keyboard",
        "background_thumbnail_wallpaper",
        "carousel_easing",
    },
    {
        "rgui",
        "RGUI",
        "fixed_grid_terminal",
        "retropad_keyboard",
        "inline_or_side_thumbnail",
        "instant_low_memory",
    },
};

static int rf_ascii_casecmp(const char *left, const char *right)
{
    unsigned char l;
    unsigned char r;

    if (!left || !right)
        return left == right ? 0 : left ? 1 : -1;

    while (*left && *right)
    {
        l = (unsigned char)tolower((unsigned char)*left++);
        r = (unsigned char)tolower((unsigned char)*right++);
        if (l != r)
            return (int)l - (int)r;
    }

    return (int)(unsigned char)tolower((unsigned char)*left)
         - (int)(unsigned char)tolower((unsigned char)*right);
}

uint32_t rf_menu_driver_count(void)
{
    return (uint32_t)(sizeof(RF_MENU_DRIVERS) / sizeof(RF_MENU_DRIVERS[0]));
}

const rf_menu_driver_spec *rf_menu_driver_at(uint32_t index)
{
    if (index >= rf_menu_driver_count())
        return NULL;
    return &RF_MENU_DRIVERS[index];
}

const rf_menu_driver_spec *rf_menu_driver_find(const char *ident)
{
    uint32_t i;

    if (!ident || !*ident)
        return rf_menu_driver_default();

    for (i = 0; i < rf_menu_driver_count(); i++)
    {
        if (rf_ascii_casecmp(ident, RF_MENU_DRIVERS[i].ident) == 0)
            return &RF_MENU_DRIVERS[i];
    }

    return rf_menu_driver_default();
}

const rf_menu_driver_spec *rf_menu_driver_default(void)
{
    return &RF_MENU_DRIVERS[0];
}

const char *rf_menu_driver_next_ident(const char *ident)
{
    uint32_t i;
    const rf_menu_driver_spec *current = rf_menu_driver_find(ident);

    for (i = 0; i < rf_menu_driver_count(); i++)
    {
        if (&RF_MENU_DRIVERS[i] == current)
            return RF_MENU_DRIVERS[(i + 1) % rf_menu_driver_count()].ident;
    }

    return rf_menu_driver_default()->ident;
}

uint32_t rf_menu_driver_is_supported(const char *ident)
{
    uint32_t i;

    if (!ident || !*ident)
        return 0;

    for (i = 0; i < rf_menu_driver_count(); i++)
    {
        if (rf_ascii_casecmp(ident, RF_MENU_DRIVERS[i].ident) == 0)
            return 1;
    }

    return 0;
}
