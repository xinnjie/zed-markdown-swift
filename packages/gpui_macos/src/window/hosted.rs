//! GPUI windows that render into a view owned by another application.
//!
//! A hosted window creates no `NSWindow`. Its `GPUIView` is added as a subview of a view the
//! embedding application provides (for example an `NSViewRepresentable`), and GPUI treats that
//! view's bounds as the window's content. The host window's key, occlusion and screen changes are
//! forwarded through notifications instead of `GPUIWindow` delegate callbacks; window-level
//! operations such as titles, fullscreen and resizing stay with the host.

use super::*;
use parking_lot::MutexGuard;

const HOSTED_VIEW_IVAR: &str = "hostedView";
static mut HOSTED_OBSERVER_CLASS: *const Class = ptr::null();

#[ctor(unsafe)]
unsafe fn build_hosted_observer_class() {
    unsafe {
        let mut decl = ClassDecl::new("GPUIHostedWindowObserver", class!(NSObject)).unwrap();
        decl.add_ivar::<*mut c_void>(HOSTED_VIEW_IVAR);
        decl.add_method(
            sel!(hostWindowDidBecomeKey:),
            host_window_did_become_key as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(hostWindowDidResignKey:),
            host_window_did_resign_key as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(hostWindowDidChangeOcclusionState:),
            host_window_did_change_occlusion_state as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(hostWindowDidChangeScreen:),
            host_window_did_change_screen as extern "C" fn(&Object, Sel, id),
        );
        HOSTED_OBSERVER_CLASS = decl.register();
    }
}

fn observed_view(observer: &Object) -> &Object {
    unsafe {
        let view: *mut c_void = *observer.get_ivar(HOSTED_VIEW_IVAR);
        &*(view as *const Object)
    }
}

// The window delegate callbacks only read the shared state from the `windowState` ivar, which the
// hosted `GPUIView` carries as well, so they can be driven with the view as the receiver.
extern "C" fn host_window_did_become_key(this: &Object, _: Sel, _: id) {
    window_did_change_key_status(observed_view(this), sel!(windowDidBecomeKey:), nil);
}

extern "C" fn host_window_did_resign_key(this: &Object, _: Sel, _: id) {
    window_did_change_key_status(observed_view(this), sel!(windowDidResignKey:), nil);
}

extern "C" fn host_window_did_change_occlusion_state(this: &Object, _: Sel, _: id) {
    window_did_change_occlusion_state(
        observed_view(this),
        sel!(windowDidChangeOcclusionState:),
        nil,
    );
}

extern "C" fn host_window_did_change_screen(this: &Object, _: Sel, _: id) {
    window_did_change_screen(observed_view(this), sel!(windowDidChangeScreen:), nil);
}

impl MacWindow {
    /// Opens a GPUI window whose content is a `GPUIView` added to `host_view`.
    ///
    /// `host_view` must already be in a window; the embedder recreates the GPUI window when the
    /// host view moves to another window.
    pub(crate) fn open_hosted(
        handle: AnyWindowHandle,
        host_view: id,
        params: WindowParams,
        cursor_visible: Arc<AtomicBool>,
        foreground_executor: ForegroundExecutor,
        background_executor: BackgroundExecutor,
        renderer_context: renderer::Context,
        marker: MainThreadMarker,
    ) -> Self {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            let native_window: id = msg_send![host_view, window];
            assert!(
                !native_window.is_null(),
                "a hosted GPUI view needs a window"
            );

            let native_view: id = msg_send![VIEW_CLASS, alloc];
            let native_view = NSView::initWithFrame_(native_view, NSView::bounds(host_view));
            assert!(!native_view.is_null());
            let content_size = NSView::bounds(host_view).size;

            let state = Arc::new(Mutex::new(MacWindowState {
                handle,
                foreground_executor,
                background_executor,
                native_window,
                native_view: NonNull::new_unchecked(native_view),
                blurred_view: None,
                background_appearance: WindowBackgroundAppearance::Opaque,
                cursor_style: CursorStyle::Arrow,
                cursor_visible,
                frame_source: None,
                renderer: renderer::new_renderer(
                    renderer_context,
                    native_window as *mut _,
                    native_view as *mut _,
                    size(content_size.width as f32, content_size.height as f32),
                    false,
                    params.use_display_p3,
                ),
                request_frame_callback: None,
                event_callback: None,
                activate_callback: None,
                visibility_callback: None,
                last_visibility: None,
                resize_callback: None,
                moved_callback: None,
                display_changed_callback: None,
                should_close_callback: None,
                close_callback: None,
                appearance_changed_callback: None,
                input_handler: None,
                last_key_equivalent: None,
                last_left_mouse_down_event: None,
                synthetic_drag_counter: 0,
                traffic_light_position: None,
                traffic_light_frames: None,
                pre_fullscreen_traffic_light_frames: None,
                transparent_titlebar: false,
                previous_modifiers_changed_event: None,
                keystroke_for_do_command: None,
                do_command_handled: None,
                external_files_dragged: false,
                first_mouse: false,
                app_owns_titlebar_drag: false,
                fullscreen_restore_bounds: Bounds::default(),
                is_exiting_fullscreen: false,
                simple_fullscreen_state: None,
                move_tab_to_new_window_callback: None,
                merge_all_windows_callback: None,
                select_next_tab_callback: None,
                select_previous_tab_callback: None,
                toggle_tab_bar_callback: None,
                activated_least_once: false,
                closed: Arc::new(AtomicBool::new(false)),
                accesskit_adapter: None,
                sheet_parent: None,
                hosted: true,
                hosted_observer: None,
            }));

            (*native_view).set_ivar(
                WINDOW_STATE_IVAR,
                Arc::into_raw(state.clone()) as *const c_void,
            );
            native_view.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
            native_view.setWantsBestResolutionOpenGLSurface_(YES);
            native_view.setWantsLayer(YES);
            let _: () = msg_send![
                native_view,
                setLayerContentsRedrawPolicy: NSViewLayerContentsRedrawDuringViewResize
            ];
            host_view.addSubview_(native_view.autorelease());
            add_mouse_tracking_area(&*native_view.cast::<Objc2NSView>());

            let observer: id = msg_send![HOSTED_OBSERVER_CLASS, new];
            (*observer).set_ivar(HOSTED_VIEW_IVAR, native_view as *mut c_void);
            let center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
            for (selector, name) in [
                (
                    sel!(hostWindowDidBecomeKey:),
                    "NSWindowDidBecomeKeyNotification",
                ),
                (
                    sel!(hostWindowDidResignKey:),
                    "NSWindowDidResignKeyNotification",
                ),
                (
                    sel!(hostWindowDidChangeOcclusionState:),
                    "NSWindowDidChangeOcclusionStateNotification",
                ),
                (
                    sel!(hostWindowDidChangeScreen:),
                    "NSWindowDidChangeScreenNotification",
                ),
            ] {
                let _: () = msg_send![
                    center,
                    addObserver: observer
                    selector: selector
                    name: ns_string(name)
                    object: native_window
                ];
            }
            state.lock().hosted_observer = Some(observer);

            update_window_scale_factor(&state);
            state.lock().start_display_link();

            pool.drain();
            Self(state, marker)
        }
    }
}

/// Tears down a hosted window without touching the host's `NSWindow`.
pub(super) fn drop_hosted(mut state: MutexGuard<'_, MacWindowState>) {
    drop(state.accesskit_adapter.take());
    state.renderer.destroy();
    state.frame_source.take();
    state.input_handler.take();
    state.visibility_callback.take();
    state.closed.store(true, Ordering::Release);
    let view = state.native_view.as_ptr();
    let observer = state.hosted_observer.take();
    drop(state);
    unsafe {
        if let Some(observer) = observer {
            let center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
            let _: () = msg_send![center, removeObserver: observer];
            let _: () = msg_send![observer, release];
        }
        // The host's subview reference is the last strong one; releasing it runs `dealloc_view`,
        // which drops the view's share of the window state.
        let _: () = msg_send![view, removeFromSuperview];
    }
}

/// Whether a key equivalent should reach this view. The window offers key equivalents to every
/// view, so a hosted view only takes them while it is the first responder.
pub(super) fn is_key_target(view: &Object) -> bool {
    let window_state = unsafe { get_window_state(view) };
    let lock = window_state.lock();
    if !lock.hosted {
        return true;
    }
    unsafe {
        let first_responder: id = msg_send![lock.native_window, firstResponder];
        first_responder == view as *const Object as id
    }
}

pub(super) fn focus_view(view: &Object) {
    unsafe {
        let window: id = msg_send![view, window];
        if !window.is_null() {
            let _: BOOL = msg_send![window, makeFirstResponder: view];
        }
    }
}
