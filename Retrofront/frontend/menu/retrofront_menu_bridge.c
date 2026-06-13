#include "retrofront_menu_bridge.h"
#include <stddef.h>

/* Thin C-side bridge: real state, mock settings/lists, notifications and
 * command ownership live in Rust UiRuntime. RetroArch menu code should call
 * these wrappers instead of growing frontend behavior in C. */
RetrofrontUiRuntime *retrofront_menu_bridge_create(const char *driver)
{
   return retrofront_ui_runtime_create(driver ? driver : "xmb");
}

void retrofront_menu_bridge_destroy(RetrofrontUiRuntime *runtime)
{
   retrofront_ui_runtime_destroy(runtime);
}

void retrofront_menu_bridge_begin_frame(RetrofrontUiRuntime *runtime,
      uint32_t width, uint32_t height, float scale)
{
   if (runtime)
      retrofront_ui_runtime_begin_frame(runtime, width, height, scale);
}

void retrofront_menu_bridge_end_frame(RetrofrontUiRuntime *runtime)
{
   if (runtime)
      retrofront_ui_runtime_end_frame(runtime);
}

const char *retrofront_menu_bridge_current_screen(RetrofrontUiRuntime *runtime)
{
   return runtime ? retrofront_ui_runtime_get_screen(runtime) : "Main Menu";
}
