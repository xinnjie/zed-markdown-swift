#!/usr/bin/env bash
# Builds the demo as an iOS simulator app bundle (no Xcode project) and installs it into a booted
# simulator. Run tooling/build-rust-ios.sh first.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

product=ZedMarkdownPreview
sdk="$(xcrun --sdk iphonesimulator --show-sdk-path)"
triple=arm64-apple-ios17.0-simulator
rust_lib="$root/target/ios/debug"
out="$root/.build/ios-sim"
app="$out/$product.app"

rm -rf "$out"
mkdir -p "$out/modules" "$app"

swiftc_common=(-sdk "$sdk" -target "$triple" -swift-version 6
  -enable-upcoming-feature NonisolatedNonsendingByDefault
  -enable-upcoming-feature MemberImportVisibility
  -enable-upcoming-feature InternalImportsByDefault
  -Xcc -fmodule-map-file="$root/c-module/module.modulemap")

# The library module (ZedMarkdown) first, then the app against it.
xcrun swiftc "${swiftc_common[@]}" -module-name ZedMarkdown -parse-as-library \
  -emit-module -emit-module-path "$out/modules/ZedMarkdown.swiftmodule" \
  -c Sources/ZedMarkdown/*.swift -o "$out/ZedMarkdown.o" -wmo

xcrun swiftc "${swiftc_common[@]}" -module-name "$product" -parse-as-library \
  -I "$out/modules" \
  "$out/ZedMarkdown.o" Demo/ZedMarkdownPreview/*.swift \
  -L "$rust_lib" -lzed_gpui_c_api -liconv -lobjc \
  -framework UIKit -framework CoreFoundation -framework CoreGraphics -framework CoreText \
  -framework CoreVideo -framework Foundation -framework Metal -framework QuartzCore \
  -framework Security -framework SystemConfiguration -framework UserNotifications \
  -o "$app/$product"

cp "$root/Demo/Info-iOS.plist" "$app/Info.plist"
codesign --force --sign - "$app"
echo "Built simulator app: $app"
