//! A Zed Markdown preview of one document, laid out like Zed's own preview pane.

use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use gpui::{
    App, Context, Entity, FocusHandle, Focusable, ImageSource, Resource, RetainAllImageCache,
    ScrollHandle, SharedString, SharedUri, Window, div, prelude::*,
};
use language::LanguageRegistry;
use markdown::{
    CodeBlockRenderer, CopyButtonVisibility, Markdown, MarkdownElement, MarkdownFont,
    MarkdownStyle, WrapButtonVisibility,
};
use theme::ActiveTheme;

pub(crate) type OpenUrlHandler = Rc<dyn Fn(&str)>;

pub(crate) struct MarkdownRoot {
    markdown: Entity<Markdown>,
    base_directory: Option<PathBuf>,
    image_cache: Entity<RetainAllImageCache>,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    open_url: Option<OpenUrlHandler>,
}

impl MarkdownRoot {
    pub(crate) fn new(
        source: String,
        base_directory: Option<PathBuf>,
        languages: Arc<LanguageRegistry>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let markdown = cx.new(|cx| Markdown::new(source.into(), Some(languages), None, cx));
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            markdown,
            base_directory,
            image_cache: RetainAllImageCache::new(cx),
            scroll_handle: ScrollHandle::new(),
            focus_handle,
            open_url: None,
        }
    }

    pub(crate) fn set_source(
        &mut self,
        source: String,
        base_directory: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        self.base_directory = base_directory;
        self.markdown
            .update(cx, |markdown, cx| markdown.replace(source, cx));
        cx.notify();
    }

    pub(crate) fn set_open_url_handler(&mut self, handler: Option<OpenUrlHandler>) {
        self.open_url = handler;
    }
}

fn resolve_image(dest_url: &str, base_directory: Option<&Path>) -> Option<ImageSource> {
    if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
        return Some(ImageSource::Resource(Resource::Uri(SharedUri::from(
            dest_url.to_string(),
        ))));
    }
    if dest_url.starts_with("data:") {
        return None;
    }
    let decoded = urlencoding::decode(dest_url)
        .map(|decoded| decoded.into_owned())
        .unwrap_or_else(|_| dest_url.to_string());
    let path = Path::new(&decoded);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_directory?.join(path)
    };
    path.exists()
        .then(|| ImageSource::Resource(Resource::Path(Arc::from(path.as_path()))))
}

impl Focusable for MarkdownRoot {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MarkdownRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base_directory = self.base_directory.clone();
        let open_url = self.open_url.clone();
        let element = MarkdownElement::new(
            self.markdown.clone(),
            MarkdownStyle::themed(MarkdownFont::Preview, window, cx),
        )
        .code_block_renderer(CodeBlockRenderer::Default {
            copy_button_visibility: CopyButtonVisibility::VisibleOnHover,
            wrap_button_visibility: WrapButtonVisibility::Hidden,
            border: false,
        })
        .scroll_handle(self.scroll_handle.clone())
        .image_resolver(move |dest_url, _| resolve_image(dest_url, base_directory.as_deref()))
        .on_url_click(move |url: SharedString, _, _| {
            if let Some(open_url) = &open_url {
                open_url(&url);
            }
        });

        div()
            .image_cache(self.image_cache.clone())
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                div()
                    .id("zed-markdown-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .p_4()
                    .child(element),
            )
    }
}
