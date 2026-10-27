#ifndef ZED_GPUI_H
#define ZED_GPUI_H

// C interface of zed-gpui-c-api: Zed's Markdown preview rendered by GPUI
// into an NSView owned by the caller. Call every function on the main thread. Create a view only
// while its host NSView is in a window, and destroy it before the host view leaves that window.

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ZedGpuiView ZedGpuiView;

typedef void (*ZedGpuiOpenUrlCallback)(void *context, const char *url);

// Switches every Zed view between Zed's One Light and One Dark themes.
void zed_gpui_set_dark_appearance(bool dark);

// `base_directory` may be NULL; relative image paths resolve against it.
ZedGpuiView *zed_gpui_markdown_create(void *host_view, const char *markdown,
                                      const char *base_directory);
void zed_gpui_markdown_set_text(ZedGpuiView *view, const char *markdown,
                                const char *base_directory);
// Pass a NULL callback to ignore link clicks. `context` must outlive the handler.
void zed_gpui_markdown_set_open_url_handler(ZedGpuiView *view,
                                            ZedGpuiOpenUrlCallback callback,
                                            void *context);

void zed_gpui_view_destroy(ZedGpuiView *view);

#ifdef __cplusplus
}
#endif

#endif
