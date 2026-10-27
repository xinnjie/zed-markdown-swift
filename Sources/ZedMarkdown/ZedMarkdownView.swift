import AppKit
import CZedGPUI
public import SwiftUI

/// Zed's Markdown preview of `markdown`. Relative image paths resolve against `baseURL`.
///
/// Link clicks open in the default browser unless a handler is set with `zedOnOpenURL(_:)`.
public struct ZedMarkdownView: NSViewRepresentable {
  private let markdown: String
  private let baseURL: URL?

  @Environment(\.zedOpenURL) private var openURL

  /// Renders `markdown`, resolving relative image paths against `baseURL`.
  public init(markdown: String, baseURL: URL? = nil) {
    self.markdown = markdown
    self.baseURL = baseURL
  }

  /// Creates the view that hosts the Zed Markdown preview.
  public func makeNSView(context: Context) -> NSView {
    let view = MarkdownHostView()
    update(view)
    return view
  }

  /// Passes changed Markdown to the Zed preview.
  public func updateNSView(_ nsView: NSView, context: Context) {
    guard let view = nsView as? MarkdownHostView else { return }
    update(view)
  }

  private func update(_ view: MarkdownHostView) {
    view.openURL = openURL.handler
    view.setContent(.init(markdown: markdown, baseDirectory: baseURL?.path))
  }
}

extension View {
  /// Handles link clicks in a `ZedMarkdownView` instead of opening the default browser.
  public func zedOnOpenURL(_ action: @escaping @MainActor (URL) -> Void) -> some View {
    environment(\.zedOpenURL, ZedOpenURLAction(handler: action))
  }
}

struct ZedOpenURLAction {
  var handler: @MainActor (URL) -> Void = { url in NSWorkspace.shared.open(url) }
}

extension EnvironmentValues {
  @Entry var zedOpenURL = ZedOpenURLAction()
}

private final class MarkdownHostView: ZedHostView {
  struct Content: Equatable {
    var markdown: String
    var baseDirectory: String?
  }

  var openURL: @MainActor (URL) -> Void = { _ in }
  private var content: Content?

  func setContent(_ content: Content) {
    guard content != self.content else { return }
    self.content = content
    guard let handle else { return }
    zed_gpui_markdown_set_text(handle, content.markdown, content.baseDirectory)
  }

  override func makeZedView() -> OpaquePointer? {
    guard let content else { return nil }
    return zed_gpui_markdown_create(hostPointer, content.markdown, content.baseDirectory)
  }

  override func didCreate(_ handle: OpaquePointer) {
    // The handle never outlives this view, so the unretained context stays valid.
    zed_gpui_markdown_set_open_url_handler(
      handle,
      { context, url in
        guard let context, let url, let target = URL(string: String(cString: url)) else { return }
        MainActor.assumeIsolated {
          let view = Unmanaged<MarkdownHostView>.fromOpaque(context).takeUnretainedValue()
          view.openURL(target)
        }
      }, hostPointer)
  }
}
