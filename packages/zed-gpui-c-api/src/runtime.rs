//! The process-wide GPUI app embedded in the host application's run loop.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use assets::Assets;
use gpui::{AnyWindowHandle, App, AsyncApp, WindowOptions, prelude::*};
use gpui_macos::MacPlatform;
use language::LanguageRegistry;
use node_runtime::NodeRuntime;
use settings::{KeymapFile, SettingsStore};
use theme::{ActiveTheme, LoadThemes};

pub(crate) struct Runtime {
    platform: Rc<MacPlatform>,
    // Keeps the app alive; GPUI holds it weakly everywhere else.
    _app: gpui::ApplicationHandle,
    pub(crate) cx: AsyncApp,
    pub(crate) languages: Arc<LanguageRegistry>,
}

thread_local! {
    static RUNTIME: RefCell<Option<Rc<Runtime>>> = const { RefCell::new(None) };
}

/// Returns the embedded app, starting it on first use. Must run on the main thread.
pub(crate) fn runtime() -> Rc<Runtime> {
    RUNTIME.with(|slot| {
        slot.borrow_mut()
            .get_or_insert_with(|| Rc::new(Runtime::start()))
            .clone()
    })
}

impl Runtime {
    fn start() -> Self {
        zlog::init();
        let platform = Rc::new(MacPlatform::new_hosted());
        let http_client = Arc::new(
            reqwest_client::ReqwestClient::user_agent("firehorse-zed-gpui-demo")
                .expect("failed to create HTTP client"),
        );
        let languages = Rc::new(RefCell::new(None));
        let app = gpui::Application::new_inaccessible(platform.clone())
            .with_assets(Assets)
            .with_http_client(http_client)
            .run_embedded({
                let languages = languages.clone();
                move |cx| *languages.borrow_mut() = Some(init_app(cx))
            });
        let cx = app.to_async();
        let languages = languages.borrow_mut().take().expect("app did not launch");
        Self {
            platform,
            _app: app,
            cx,
            languages,
        }
    }

    /// Opens a GPUI window whose content is drawn into `host_view`.
    pub(crate) fn open_hosted<V: 'static + Render>(
        &self,
        host_view: *mut std::ffi::c_void,
        build: impl FnOnce(&mut gpui::Window, &mut App) -> gpui::Entity<V>,
    ) -> anyhow::Result<gpui::WindowHandle<V>> {
        // SAFETY: the Swift wrapper passes its own NSView, which is in a window, on the main thread.
        unsafe { self.platform.set_next_window_host_view(host_view) };
        let options = WindowOptions {
            titlebar: None,
            focus: false,
            show: true,
            ..Default::default()
        };
        self.cx.update(|cx| cx.open_window(options, build))
    }

    pub(crate) fn close(&self, window: AnyWindowHandle) {
        self.cx.update(|cx| {
            window
                .update(cx, |_, window, _| window.remove_window())
                .ok()
        });
    }
}

fn init_app(cx: &mut App) -> Arc<LanguageRegistry> {
    settings::init(cx);
    release_channel::init(semver::Version::new(0, 0, 0), cx);
    theme_settings::init(LoadThemes::All(Box::new(Assets)), cx);
    Assets.load_fonts(cx).expect("failed to load Zed fonts");

    match KeymapFile::load_asset_allow_partial_failure(settings::DEFAULT_KEYMAP_PATH, cx) {
        Ok(bindings) => cx.bind_keys(bindings),
        Err(error) => log::error!("failed to load the default keymap: {error:#}"),
    }

    let languages = LanguageRegistry::new(cx.background_executor().clone());
    languages.set_theme(cx.theme().clone());
    let languages = Arc::new(languages);
    let fs = fs::RealFs::new(None, cx.background_executor().clone());
    languages::init(languages.clone(), fs, NodeRuntime::unavailable(), cx);
    languages
}

/// Switches every embedded view between Zed's One Light and One Dark themes.
pub(crate) fn set_dark_appearance(dark: bool) {
    let runtime = runtime();
    runtime.cx.update(|cx| {
        let theme = if dark { "One Dark" } else { "One Light" };
        let settings = format!(r#"{{ "theme": "{theme}" }}"#);
        cx.update_global::<SettingsStore, _>(|store, cx| {
            let _ = store.set_user_settings(&settings, cx);
        });
        theme_settings::reload_theme(cx);
        runtime.languages.set_theme(cx.theme().clone());
    });
}
