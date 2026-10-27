// swift-tools-version: 6.4

import Foundation
import PackageDescription

// Zed's GPUI Markdown preview as a SwiftUI view, plus a demo app.
// Build the Rust static library first (`mise run build:rust`); it links from `target/<configuration>`.

let rustTarget = URL(fileURLWithPath: #filePath)
  .deletingLastPathComponent()
  .appendingPathComponent("target")

let swiftSettings: [SwiftSetting] = [
  .enableUpcomingFeature("NonisolatedNonsendingByDefault"),
  .enableUpcomingFeature("MemberImportVisibility"),
  .enableUpcomingFeature("InternalImportsByDefault"),
]

let package = Package(
  name: "ZedMarkdown",
  platforms: [.macOS("15.4")],
  products: [
    .library(name: "ZedMarkdown", targets: ["ZedMarkdown"]),
    .executable(name: "ZedMarkdownPreview", targets: ["ZedMarkdownPreview"]),
  ],
  targets: [
    .systemLibrary(name: "CZedGPUI", path: "c-module"),
    .target(
      name: "ZedMarkdown",
      dependencies: ["CZedGPUI"],
      path: "Sources/ZedMarkdown",
      swiftSettings: swiftSettings,
      linkerSettings: [
        .unsafeFlags(
          ["-L", rustTarget.appendingPathComponent("debug").path], .when(configuration: .debug)),
        .unsafeFlags(
          ["-L", rustTarget.appendingPathComponent("release").path], .when(configuration: .release)),
        // System libraries the Rust static library links against
        // (`cargo rustc -- --print native-static-libs`).
        .linkedLibrary("iconv"),
        .linkedLibrary("objc"),
        .linkedFramework("AppKit"),
        .linkedFramework("ApplicationServices"),
        .linkedFramework("Carbon"),
        .linkedFramework("CoreFoundation"),
        .linkedFramework("CoreGraphics"),
        .linkedFramework("CoreServices"),
        .linkedFramework("CoreText"),
        .linkedFramework("CoreVideo"),
        .linkedFramework("Foundation"),
        .linkedFramework("IOKit"),
        .linkedFramework("Metal"),
        .linkedFramework("QuartzCore"),
        .linkedFramework("Security"),
        .linkedFramework("SystemConfiguration"),
        .linkedFramework("UserNotifications"),
      ]
    ),
    .executableTarget(
      name: "ZedMarkdownPreview",
      dependencies: ["ZedMarkdown"],
      path: "Demo/ZedMarkdownPreview",
      swiftSettings: swiftSettings
    ),
  ],
  swiftLanguageModes: [.v6]
)
