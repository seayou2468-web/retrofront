#include "retrofront_menu.h"

#include <ctype.h>
#include <stddef.h>
#include <stdio.h>
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

static const rf_menu_source_file RF_MENU_SOURCE_FILES[] = {
    {"Retrofront/menu/cbs/menu_cbs_cancel.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_deferred_push.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_get_value.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_info.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_label.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_left.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_ok.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_right.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_scan.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_select.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_start.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_sublabel.c", 1},
    {"Retrofront/menu/cbs/menu_cbs_title.c", 1},
    {"Retrofront/menu/drivers/materialui.c", 1},
    {"Retrofront/menu/drivers/ozone.c", 1},
    {"Retrofront/menu/drivers/rgui.c", 1},
    {"Retrofront/menu/drivers/xmb.c", 1},
    {"Retrofront/menu/menu_cbs.h", 1},
    {"Retrofront/menu/menu_contentless_cores.c", 1},
    {"Retrofront/menu/menu_defines.h", 1},
    {"Retrofront/menu/menu_displaylist.c", 1},
    {"Retrofront/menu/menu_displaylist.h", 1},
    {"Retrofront/menu/menu_driver.c", 1},
    {"Retrofront/menu/menu_driver.h", 1},
    {"Retrofront/menu/menu_entries.h", 1},
    {"Retrofront/menu/menu_explore.c", 1},
    {"Retrofront/menu/menu_input.h", 1},
    {"Retrofront/menu/menu_screensaver.c", 1},
    {"Retrofront/menu/menu_screensaver.h", 1},
    {"Retrofront/menu/menu_setting.c", 1},
    {"Retrofront/menu/menu_setting.h", 1},
    {"Retrofront/menu/menu_shader.h", 1},
    {"Retrofront/menu/retrofront_menu.c", 1},
    {"Retrofront/menu/retrofront_menu.h", 1},
};

static rf_menu_host_callbacks rf_menu_host;

static char rf_menu_resolved_driver_dir[4096];
static char rf_menu_resolved_icon_dir[4096];
static char rf_menu_resolved_font_path[4096];
static char rf_menu_resolved_background_path[4096];

static uint32_t rf_menu_join3(char *out_path, uint32_t out_path_len, const char *a, const char *b, const char *c)
{
    int written;
    if (!out_path || out_path_len == 0 || !a || !*a || !b || !*b)
        return 0;
    written = c && *c
        ? snprintf(out_path, out_path_len, "%s/%s/%s", a, b, c)
        : snprintf(out_path, out_path_len, "%s/%s", a, b);
    if (written < 0 || (uint32_t)written >= out_path_len)
    {
        out_path[0] = '\0';
        return 0;
    }
    return 1;
}

static uint32_t rf_menu_host_file_exists(const char *path)
{
    if (!path || !*path)
        return 0;
    if (rf_menu_host.file_exists)
        return rf_menu_host.file_exists(path, rf_menu_host.userdata);
    if (rf_menu_host.directory_exists)
        return rf_menu_host.directory_exists(path, rf_menu_host.userdata);
    return 0;
}

static const char *rf_menu_first_existing(const char *const *candidates, uint32_t count, char *storage, uint32_t storage_len)
{
    uint32_t i;
    if (!candidates || !storage || storage_len == 0)
        return NULL;
    for (i = 0; i < count; i++)
    {
        const char *candidate = candidates[i];
        if (candidate && *candidate && rf_menu_host_file_exists(candidate))
        {
            size_t len = strlen(candidate);
            if (len >= storage_len)
                return NULL;
            memcpy(storage, candidate, len + 1);
            return storage;
        }
    }
    return NULL;
}


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


static uint32_t rf_menu_clamp_u32(uint32_t value, uint32_t min, uint32_t max)
{
    if (value < min)
        return min;
    if (value > max)
        return max;
    return value;
}

static uint32_t rf_menu_scaled(float scale, float value, uint32_t min)
{
    uint32_t result = (uint32_t)(scale * value + 0.5f);
    return result < min ? min : result;
}

uint32_t rf_menu_source_file_count(void)
{
    return (uint32_t)(sizeof(RF_MENU_SOURCE_FILES) / sizeof(RF_MENU_SOURCE_FILES[0]));
}

const rf_menu_source_file *rf_menu_source_file_at(uint32_t index)
{
    if (index >= rf_menu_source_file_count())
        return NULL;
    return &RF_MENU_SOURCE_FILES[index];
}

uint32_t rf_menu_layout_for_viewport(const char *driver_ident, uint32_t viewport_width, uint32_t viewport_height, rf_menu_layout_metrics *out_metrics)
{
    const rf_menu_driver_spec *driver;
    uint32_t short_edge;
    float scale;

    if (!out_metrics || viewport_width == 0 || viewport_height == 0)
        return 0;

    memset(out_metrics, 0, sizeof(*out_metrics));
    driver = rf_menu_driver_find(driver_ident);
    short_edge = viewport_width < viewport_height ? viewport_width : viewport_height;
    scale = (float)short_edge / 720.0f;
    if (scale < 0.75f)
        scale = 0.75f;
    if (scale > 2.25f)
        scale = 2.25f;

    out_metrics->viewport_width = viewport_width;
    out_metrics->viewport_height = viewport_height;
    out_metrics->scale = scale;

    if (rf_ascii_casecmp(driver->ident, "rgui") == 0)
    {
        out_metrics->horizontal_padding = rf_menu_scaled(scale, 12.0f, 8);
        out_metrics->vertical_padding = rf_menu_scaled(scale, 12.0f, 8);
        out_metrics->header_height = rf_menu_scaled(scale, 24.0f, 20);
        out_metrics->footer_height = out_metrics->header_height;
        out_metrics->row_height = rf_menu_scaled(scale, 28.0f, 22);
        out_metrics->icon_size = rf_menu_scaled(scale, 16.0f, 12);
        out_metrics->background_mode = 4;
    }
    else if (rf_ascii_casecmp(driver->ident, "ozone") == 0)
    {
        out_metrics->sidebar_width = rf_menu_clamp_u32((uint32_t)(viewport_width * 0.18f), 88, 220);
        out_metrics->horizontal_padding = rf_menu_scaled(scale, 28.0f, 20);
        out_metrics->vertical_padding = rf_menu_scaled(scale, 24.0f, 18);
        out_metrics->header_height = rf_menu_scaled(scale, 88.0f, 64);
        out_metrics->footer_height = rf_menu_scaled(scale, 32.0f, 20);
        out_metrics->row_height = rf_menu_scaled(scale, 56.0f, 44);
        out_metrics->icon_size = rf_menu_scaled(scale, 28.0f, 22);
        out_metrics->background_mode = 2;
    }
    else if (rf_ascii_casecmp(driver->ident, "xmb") == 0)
    {
        out_metrics->horizontal_padding = rf_menu_scaled(scale, 80.0f, 34);
        out_metrics->vertical_padding = rf_menu_scaled(scale, 48.0f, 28);
        out_metrics->header_height = rf_menu_scaled(scale, 104.0f, 74);
        out_metrics->footer_height = rf_menu_scaled(scale, 28.0f, 18);
        out_metrics->row_height = rf_menu_scaled(scale, 64.0f, 48);
        out_metrics->icon_size = rf_menu_scaled(scale, 48.0f, 34);
        out_metrics->background_mode = 3;
    }
    else
    {
        out_metrics->horizontal_padding = rf_menu_scaled(scale, 24.0f, 16);
        out_metrics->vertical_padding = rf_menu_scaled(scale, 18.0f, 14);
        out_metrics->header_height = rf_menu_scaled(scale, 64.0f, 52);
        out_metrics->footer_height = rf_menu_scaled(scale, 64.0f, 52);
        out_metrics->row_height = rf_menu_scaled(scale, 64.0f, 48);
        out_metrics->icon_size = rf_menu_scaled(scale, 28.0f, 22);
        out_metrics->background_mode = 1;
    }

    out_metrics->content_x = out_metrics->sidebar_width + out_metrics->horizontal_padding;
    out_metrics->content_y = out_metrics->header_height;
    if (out_metrics->content_x + out_metrics->horizontal_padding < viewport_width)
        out_metrics->content_width = viewport_width - out_metrics->content_x - out_metrics->horizontal_padding;
    if (out_metrics->content_y + out_metrics->footer_height + out_metrics->vertical_padding < viewport_height)
        out_metrics->content_height = viewport_height - out_metrics->content_y - out_metrics->footer_height - out_metrics->vertical_padding;
    return 1;
}

uint32_t rf_menu_asset_path(const char *driver_ident, const char *asset_name, char *out_path, uint32_t out_path_len)
{
    const char *assets_directory;
    const rf_menu_driver_spec *driver;
    if (!asset_name || !*asset_name || !out_path || out_path_len == 0 || !rf_menu_host.get_setting)
        return 0;

    assets_directory = rf_menu_host.get_setting("menu_assets_directory", rf_menu_host.userdata);
    if (!assets_directory || !*assets_directory)
        assets_directory = rf_menu_host.get_setting("assets_directory", rf_menu_host.userdata);
    if (!assets_directory || !*assets_directory)
        return 0;

    driver = rf_menu_driver_find(driver_ident);
    return rf_menu_join3(out_path, out_path_len, assets_directory, driver->asset_subdirectory, asset_name);
}


uint32_t rf_menu_resolve_assets(const char *driver_ident, rf_menu_resolved_assets *out_assets)
{
    const char *assets_directory;
    const rf_menu_driver_spec *driver;
    const char *font_candidates[5];
    const char *background_candidates[6];
    uint32_t font_count = 0;
    uint32_t background_count = 0;
    char driver_png[4096];
    char xmb_png[4096];
    char ozone_png[4096];
    char font_a[4096];
    char font_b[4096];
    char font_c[4096];
    char bg_a[4096];
    char bg_b[4096];
    char bg_c[4096];
    char bg_d[4096];

    if (!out_assets || !rf_menu_host.get_setting)
        return 0;

    memset(out_assets, 0, sizeof(*out_assets));
    assets_directory = rf_menu_host.get_setting("menu_assets_directory", rf_menu_host.userdata);
    if (!assets_directory || !*assets_directory)
        assets_directory = rf_menu_host.get_setting("assets_directory", rf_menu_host.userdata);
    if (!assets_directory || !*assets_directory)
        return 0;

    driver = rf_menu_driver_find(driver_ident);
    if (!rf_menu_join3(rf_menu_resolved_driver_dir, sizeof(rf_menu_resolved_driver_dir), assets_directory, driver->asset_subdirectory, NULL))
        return 0;

    out_assets->root_directory = assets_directory;
    out_assets->driver_directory = rf_menu_resolved_driver_dir;
    out_assets->assets_ready = rf_menu_assets_ready();

    if (rf_ascii_casecmp(driver->ident, "xmb") == 0)
    {
        if (rf_menu_join3(rf_menu_resolved_icon_dir, sizeof(rf_menu_resolved_icon_dir), assets_directory, driver->asset_subdirectory, "png"))
            out_assets->icon_directory = rf_menu_resolved_icon_dir;
    }
    else if (rf_menu_join3(rf_menu_resolved_icon_dir, sizeof(rf_menu_resolved_icon_dir), assets_directory, driver->asset_subdirectory, "png"))
    {
        if (rf_menu_host.directory_exists && rf_menu_host.directory_exists(rf_menu_resolved_icon_dir, rf_menu_host.userdata))
            out_assets->icon_directory = rf_menu_resolved_icon_dir;
        else
            out_assets->icon_directory = rf_menu_resolved_driver_dir;
    }

    if (rf_menu_join3(font_a, sizeof(font_a), rf_menu_resolved_driver_dir, "font.ttf", NULL)) font_candidates[font_count++] = font_a;
    if (rf_menu_join3(font_b, sizeof(font_b), rf_menu_resolved_driver_dir, "regular.ttf", NULL)) font_candidates[font_count++] = font_b;
    if (rf_menu_join3(font_c, sizeof(font_c), rf_menu_resolved_driver_dir, "bold.ttf", NULL)) font_candidates[font_count++] = font_c;
    if (rf_menu_join3(xmb_png, sizeof(xmb_png), assets_directory, "xmb/monochrome", "font.ttf")) font_candidates[font_count++] = xmb_png;
    if (rf_menu_join3(ozone_png, sizeof(ozone_png), assets_directory, "ozone", "regular.ttf")) font_candidates[font_count++] = ozone_png;
    out_assets->font_path = rf_menu_first_existing(font_candidates, font_count, rf_menu_resolved_font_path, sizeof(rf_menu_resolved_font_path));

    if (rf_menu_join3(bg_a, sizeof(bg_a), rf_menu_resolved_driver_dir, "bg.png", NULL)) background_candidates[background_count++] = bg_a;
    if (rf_menu_join3(bg_b, sizeof(bg_b), rf_menu_resolved_driver_dir, "wallpaper.png", NULL)) background_candidates[background_count++] = bg_b;
    if (rf_menu_join3(bg_c, sizeof(bg_c), rf_menu_resolved_driver_dir, "png/retroarch.png", NULL)) background_candidates[background_count++] = bg_c;
    if (rf_menu_join3(bg_d, sizeof(bg_d), rf_menu_resolved_driver_dir, "monochrome/png/retroarch.png", NULL)) background_candidates[background_count++] = bg_d;
    if (rf_menu_join3(driver_png, sizeof(driver_png), assets_directory, "xmb/monochrome/png", "retroarch.png")) background_candidates[background_count++] = driver_png;
    if (rf_menu_join3(ozone_png, sizeof(ozone_png), assets_directory, "ozone/png", "retroarch.png")) background_candidates[background_count++] = ozone_png;
    out_assets->background_path = rf_menu_first_existing(background_candidates, background_count, rf_menu_resolved_background_path, sizeof(rf_menu_resolved_background_path));

    return 1;
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
