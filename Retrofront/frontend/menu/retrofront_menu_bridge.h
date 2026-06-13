#ifndef RETROFRONT_MENU_BRIDGE_H
#define RETROFRONT_MENU_BRIDGE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct UiRuntime RetrofrontUiRuntime;
RetrofrontUiRuntime *retrofront_ui_runtime_create(const char *driver);
void retrofront_ui_runtime_destroy(RetrofrontUiRuntime *runtime);
void retrofront_ui_runtime_begin_frame(RetrofrontUiRuntime *runtime, uint32_t width, uint32_t height, float scale);
void retrofront_ui_runtime_end_frame(RetrofrontUiRuntime *runtime);
const char *retrofront_ui_runtime_get_screen(RetrofrontUiRuntime *runtime);
#ifdef __cplusplus
}
#endif
#endif
