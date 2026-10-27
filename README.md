# zed-markdown-swift

Zed's GPUI Markdown view, embedded in a SwiftUI app as a plain `NSView`-backed component.

Zed's dependency tree is huge, so this lives in its own repository instead of inside a larger app,
where it would dominate build times.

## Layout

- `packages/gpui_macos` — Zed's `gpui_macos` crate with a hosted `NSView` mode (see its `UPSTREAM.md`).
- `packages/zed-gpui-c-api` — C API over Zed's Markdown view.
- `Sources/ZedGPUI` — SwiftUI wrapper.
- `Demo/` — Markdown preview demo app.

## License

`packages/gpui_macos` is Apache-2.0 (from Zed). Crates built on Zed's GPL-3.0 crates are GPL-3.0-or-later.
