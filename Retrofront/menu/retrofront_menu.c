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
        "materialui",
        "dark",
    },
    {
        "ozone",
        "Ozone",
        "desktop_sidebar_detail",
        "pointer_keyboard_retropad",
        "right_panel_dual_thumbnail",
        "fade_slide_sidebar",
        "ozone",
        "dark",
    },
    {
        "xmb",
        "XMB",
        "horizontal_categories_vertical_items",
        "retropad_keyboard",
        "background_thumbnail_wallpaper",
        "carousel_easing",
        "xmb/monochrome",
        "monochrome",
    },
    {
        "rgui",
        "RGUI",
        "fixed_grid_terminal",
        "retropad_keyboard",
        "inline_or_side_thumbnail",
        "instant_low_memory",
        "rgui",
        "default",
    },
};

static rf_menu_host_callbacks rf_menu_host;

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

void rf_menu_connect_host(const rf_menu_host_callbacks *callbacks)
{
    if (callbacks)
        rf_menu_host = *callbacks;
    else
        memset(&rf_menu_host, 0, sizeof(rf_menu_host));
}

const char *rf_menu_setting(const char *key)
{
    if (!key || !rf_menu_host.get_setting)
        return NULL;
    return rf_menu_host.get_setting(key, rf_menu_host.userdata);
}

uint32_t rf_menu_set_driver(const char *ident)
{
    const rf_menu_driver_spec *driver;

    if (!rf_menu_host.set_setting)
        return 0;

    driver = rf_menu_driver_find(ident);
    return rf_menu_host.set_setting("menu_driver", driver->ident, rf_menu_host.userdata);
}

uint32_t rf_menu_assets_ready(void)
{
    const char *assets_directory;
    const char *driver_ident;
    const rf_menu_driver_spec *driver;
    char path[4096];
    size_t root_len;
    size_t sub_len;

    if (!rf_menu_host.get_setting || !rf_menu_host.directory_exists)
        return 0;

    assets_directory = rf_menu_host.get_setting("menu_assets_directory", rf_menu_host.userdata);
    if (!assets_directory || !*assets_directory)
        assets_directory = rf_menu_host.get_setting("assets_directory", rf_menu_host.userdata);
    if (!assets_directory || !*assets_directory)
        return 0;

    driver_ident = rf_menu_host.get_setting("menu_driver", rf_menu_host.userdata);
    driver = rf_menu_driver_find(driver_ident);
    root_len = strlen(assets_directory);
    sub_len = strlen(driver->asset_subdirectory);
    if (root_len + 1 + sub_len + 1 > sizeof(path))
        return 0;
    memcpy(path, assets_directory, root_len);
    path[root_len] = '/';
    memcpy(path + root_len + 1, driver->asset_subdirectory, sub_len + 1);
    return rf_menu_host.directory_exists(path, rf_menu_host.userdata);
}

uint32_t rf_menu_get_runtime_config(rf_menu_runtime_config *out_config)
{
    const char *driver_ident;
    const rf_menu_driver_spec *driver;

    if (!out_config)
        return 0;

    memset(out_config, 0, sizeof(*out_config));
    driver_ident = rf_menu_setting("menu_driver");
    driver = rf_menu_driver_find(driver_ident);
    out_config->driver = driver;
    out_config->driver_ident = driver->ident;
    out_config->assets_directory = rf_menu_setting("menu_assets_directory");
    if (!out_config->assets_directory || !*out_config->assets_directory)
        out_config->assets_directory = rf_menu_setting("assets_directory");
    out_config->theme = rf_menu_setting("menu_theme");
    if (!out_config->theme || !*out_config->theme)
        out_config->theme = driver->default_theme;
    out_config->assets_ready = rf_menu_assets_ready();
    return 1;
}
