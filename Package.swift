// swift-tools-version: 6.4

import Foundation
import PackageDescription

// Zed's GPUI Markdown preview as a SwiftUI view, plus a demo app.
// Build the Rust static library first (`mise run build:rust`, or `mise run build:rust:ios` for the
// simulator); it links from `target/<configuration>` on macOS and `target/ios/<configuration>` on iOS.

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
  platforms: [.macOS("15.4"), .iOS("17.0")],
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
          ["-L", rustTarget.appendingPathComponent("debug").path],
          .when(platforms: [.macOS], configuration: .debug)),
        .unsafeFlags(
          ["-L", rustTarget.appendingPathComponent("release").path],
          .when(platforms: [.macOS], configuration: .release)),
        .unsafeFlags(
          ["-L", rustTarget.appendingPathComponent("ios/debug").path],
          .when(platforms: [.iOS], configuration: .debug)),
        .unsafeFlags(
          ["-L", rustTarget.appendingPathComponent("ios/release").path],
          .when(platforms: [.iOS], configuration: .release)),
        // System libraries the Rust static library links against
        // (`cargo rustc -- --print native-static-libs`).
        .linkedLibrary("iconv"),
        .linkedLibrary("objc"),
        .linkedFramework("AppKit", .when(platforms: [.macOS])),
        .linkedFramework("ApplicationServices", .when(platforms: [.macOS])),
        .linkedFramework("Carbon", .when(platforms: [.macOS])),
        .linkedFramework("CoreServices", .when(platforms: [.macOS])),
        .linkedFramework("IOKit", .when(platforms: [.macOS])),
        .linkedFramework("UIKit", .when(platforms: [.iOS])),
        .linkedFramework("CoreFoundation"),
        .linkedFramework("CoreGraphics"),
        .linkedFramework("CoreText"),
        .linkedFramework("CoreVideo"),
        .linkedFramework("Foundation"),
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
