import AppKit
import SwiftUI
import UniformTypeIdentifiers
import ZedMarkdown

@main
struct ZedMarkdownPreviewApp: App {
  init() {
    NSApplication.shared.setActivationPolicy(.regular)
    if CommandLine.arguments.contains("--dark") {
      NSApplication.shared.appearance = NSAppearance(named: .darkAqua)
    }
  }

  var body: some Scene {
    WindowGroup("Zed Markdown Preview") {
      MarkdownPreviewView()
        .frame(minWidth: 640, minHeight: 480)
    }
  }
}

struct MarkdownPreviewView: View {
  @State private var markdown = SampleMarkdown.text
  @State private var fileURL: URL?
  @State private var isImporting = false

  var body: some View {
    content
      .navigationTitle(fileURL?.lastPathComponent ?? "Sample")
      .toolbar {
        ToolbarItemGroup {
          Button("Open…", systemImage: "folder") { isImporting = true }
        }
      }
      .fileImporter(isPresented: $isImporting, allowedContentTypes: [markdownType, .plainText]) {
        if case .success(let url) = $0 {
          open(url)
        }
      }
      .dropDestination(for: URL.self) { urls, _ in
        guard let url = urls.first else { return false }
        open(url)
        return true
      }
      .task(id: fileURL) { await reloadWhileChanged() }
      .onAppear {
        if let url = LaunchArguments.fileURL { open(url) }
      }
  }

  private var content: some View {
    ZedMarkdownView(markdown: markdown, baseURL: fileURL?.deletingLastPathComponent())
  }

  private var markdownType: UTType {
    UTType(filenameExtension: "md") ?? .plainText
  }

  private func open(_ url: URL) {
    fileURL = url
    markdown = (try? String(contentsOf: url, encoding: .utf8)) ?? ""
  }

  /// Reloads the open file whenever its modification date changes, so editing it elsewhere updates
  /// the preview.
  private func reloadWhileChanged() async {
    guard let fileURL else { return }
    var lastModified = modificationDate(fileURL)
    while !Task.isCancelled {
      try? await Task.sleep(for: .milliseconds(500))
      let modified = modificationDate(fileURL)
      if modified != lastModified {
        lastModified = modified
        markdown = (try? String(contentsOf: fileURL, encoding: .utf8)) ?? markdown
      }
    }
  }

  private func modificationDate(_ url: URL) -> Date? {
    try? url.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate
  }
}

/// `--open <path>` opens a file at launch and `--dark` forces dark mode, e.g. for scripted checks.
enum LaunchArguments {
  static var fileURL: URL? {
    let arguments = CommandLine.arguments
    guard let index = arguments.firstIndex(of: "--open"), index + 1 < arguments.count else {
      return nil
    }
    return URL(fileURLWithPath: arguments[index + 1])
  }
}

enum SampleMarkdown {
  static let text = """
    # Zed Markdown Preview

    This window renders Markdown with **Zed's `markdown` crate**, drawn by GPUI into a SwiftUI view.
    Open any `.md` file with the toolbar button or drop it on the window.

    ## Lists

    - Headings, **bold**, *italic*, ~~strikethrough~~ and `inline code`
    - [Links](https://zed.dev) open in the default browser
    - Nested lists
      1. First
      2. Second

    - [x] Task lists
    - [ ] Still to do

    ## Code

    ```rust
    fn main() {
        let greeting = "Hello from GPUI";
        println!("{greeting}");
    }
    ```

    ```swift
    struct ContentView: View {
      var body: some View {
        ZedMarkdownView(markdown: "# Hello")
      }
    }
    ```

    ## Table

    | Layout | Drawing |
    | ------ | ------- |
    | GPUI (taffy) | Metal |

    > Block quotes are rendered as well.

    ---

    ![Zed app icon](https://raw.githubusercontent.com/zed-industries/zed/main/crates/zed/resources/app-icon.png)
    """
}
