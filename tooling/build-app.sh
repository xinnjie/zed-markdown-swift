#!/usr/bin/env bash
# Wraps the ZedMarkdownPreview executable in a temporary .app bundle so SwiftUI gets a real app.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
product=ZedMarkdownPreview
cd "$root"

swift build --product "$product"
bin_path="$(swift build --show-bin-path)"
app_path="$(mktemp -d "${TMPDIR:-/tmp}/zed-markdown-swift.XXXXXX")/$product.app"
mkdir -p "$app_path/Contents/MacOS"
cp "$bin_path/$product" "$app_path/Contents/MacOS/$product"
cp "$root/Demo/Info.plist" "$app_path/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleExecutable $product" "$app_path/Contents/Info.plist"
echo "Built temporary app: $app_path"
