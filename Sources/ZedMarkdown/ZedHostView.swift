import AppKit
import CZedGPUI

/// An `NSView` that GPUI draws Zed content into.
///
/// GPUI needs the host view to be in a window, so the Zed view is created when this view enters a
/// window and destroyed before it leaves.
@MainActor
class ZedHostView: NSView {
  private(set) var handle: OpaquePointer?

  override func viewWillMove(toWindow newWindow: NSWindow?) {
    super.viewWillMove(toWindow: newWindow)
    if newWindow !== window {
      destroyZedView()
    }
  }

  override func viewDidMoveToWindow() {
    super.viewDidMoveToWindow()
    guard window != nil, handle == nil else { return }
    applyAppearance()
    handle = makeZedView()
    if let handle {
      didCreate(handle)
    }
  }

  override func viewDidChangeEffectiveAppearance() {
    super.viewDidChangeEffectiveAppearance()
    applyAppearance()
  }

  /// Creates the Zed view inside `self`. Subclasses return the C handle.
  func makeZedView() -> OpaquePointer? {
    nil
  }

  /// Called after `makeZedView` succeeds, to apply settings that are not creation parameters.
  func didCreate(_ handle: OpaquePointer) {}

  var hostPointer: UnsafeMutableRawPointer {
    Unmanaged.passUnretained(self).toOpaque()
  }

  private func applyAppearance() {
    let dark = effectiveAppearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua
    zed_gpui_set_dark_appearance(dark)
  }

  private func destroyZedView() {
    guard let handle else { return }
    self.handle = nil
    zed_gpui_view_destroy(handle)
  }

  isolated deinit {
    destroyZedView()
  }
}
