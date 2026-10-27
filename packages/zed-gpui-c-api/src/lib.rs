//! C ABI that lets a SwiftUI app embed Zed's Markdown preview.
//!
//! Every function must be called on the main thread. A view handle is created into an `NSView`
//! that is already in a window and destroyed before that view leaves its window.

mod markdown_view;
mod runtime;

use std::{
    ffi::{CStr, CString, c_char, c_void},
    path::PathBuf,
    rc::Rc,
};

use gpui::{AppContext as _, WindowHandle};
use markdown_view::MarkdownRoot;
use runtime::runtime;

/// Opaque handle to one hosted Zed view.
pub struct ZedGpuiView(ViewKind);

enum ViewKind {
    Markdown(WindowHandle<MarkdownRoot>),
}

/// Called with the clicked link's URL; `context` is passed through unchanged.
pub type ZedGpuiOpenUrlCallback = extern "C" fn(context: *mut c_void, url: *const c_char);

unsafe fn string(text: *const c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

unsafe fn path(path: *const c_char) -> Option<PathBuf> {
    (!path.is_null()).then(|| PathBuf::from(unsafe { string(path) }))
}

fn into_handle(view: anyhow::Result<ViewKind>) -> *mut ZedGpuiView {
    match view {
        Ok(view) => Box::into_raw(Box::new(ZedGpuiView(view))),
        Err(error) => {
            log::error!("failed to open a hosted Zed view: {error:#}");
            std::ptr::null_mut()
        }
    }
}

/// Switches all Zed views between the One Light and One Dark themes.
#[unsafe(no_mangle)]
pub extern "C" fn zed_gpui_set_dark_appearance(dark: bool) {
    runtime::set_dark_appearance(dark);
}

/// Creates a Markdown preview inside `host_view`. Relative image paths resolve against
/// `base_directory` (nullable).
///
/// # Safety
/// `host_view` is a live `NSView` in a window; strings are NUL-terminated UTF-8 or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zed_gpui_markdown_create(
    host_view: *mut c_void,
    markdown: *const c_char,
    base_directory: *const c_char,
) -> *mut ZedGpuiView {
    let (markdown, base_directory) = unsafe { (string(markdown), path(base_directory)) };
    let runtime = runtime();
    let languages = runtime.languages.clone();
    into_handle(
        runtime
            .open_hosted(host_view, |window, cx| {
                cx.new(|cx| MarkdownRoot::new(markdown, base_directory, languages, window, cx))
            })
            .map(ViewKind::Markdown),
    )
}

/// # Safety
/// `view` comes from `zed_gpui_markdown_create`; strings are NUL-terminated UTF-8 or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zed_gpui_markdown_set_text(
    view: *mut ZedGpuiView,
    markdown: *const c_char,
    base_directory: *const c_char,
) {
    let (markdown, base_directory) = unsafe { (string(markdown), path(base_directory)) };
    unsafe {
        update_markdown(view, |root, cx| {
            root.set_source(markdown, base_directory, cx)
        })
    };
}

/// Routes link clicks to `callback`; pass a null callback to ignore them.
///
/// # Safety
/// `view` comes from `zed_gpui_markdown_create`; `context` stays valid until the handler is
/// replaced or the view is destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zed_gpui_markdown_set_open_url_handler(
    view: *mut ZedGpuiView,
    callback: Option<ZedGpuiOpenUrlCallback>,
    context: *mut c_void,
) {
    let handler = callback.map(|callback| {
        Rc::new(move |url: &str| {
            if let Ok(url) = CString::new(url) {
                callback(context, url.as_ptr());
            }
        }) as markdown_view::OpenUrlHandler
    });
    unsafe { update_markdown(view, |root, _| root.set_open_url_handler(handler)) };
}

/// Closes the view and removes its content from the host view.
///
/// # Safety
/// `view` comes from a create function and is not used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn zed_gpui_view_destroy(view: *mut ZedGpuiView) {
    if view.is_null() {
        return;
    }
    let view = unsafe { Box::from_raw(view) };
    let window = match view.0 {
        ViewKind::Markdown(handle) => handle.into(),
    };
    runtime().close(window);
}

unsafe fn update_markdown(
    view: *mut ZedGpuiView,
    update: impl FnOnce(&mut MarkdownRoot, &mut gpui::Context<MarkdownRoot>),
) {
    if let Some(ZedGpuiView(ViewKind::Markdown(handle))) = unsafe { view.as_ref() } {
        let handle = *handle;
        runtime()
            .cx
            .update(|cx| handle.update(cx, |root, _, cx| update(root, cx)))
            .ok();
    }
}
