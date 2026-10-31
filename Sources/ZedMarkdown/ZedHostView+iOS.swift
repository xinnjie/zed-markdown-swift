#if os(iOS)
  import CZedGPUI
  import UIKit

  /// A `UIView` that GPUI draws Zed content into.
  ///
  /// GPUI needs the host view to be in a window, so the Zed view is created when this view enters a
  /// window and destroyed before it leaves.
  @MainActor
  class ZedHostView: UIView {
    private(set) var handle: OpaquePointer?

    override init(frame: CGRect) {
      super.init(frame: frame)
      registerForTraitChanges([UITraitUserInterfaceStyle.self]) { (view: Self, _) in
        view.applyAppearance()
      }
    }

    required init?(coder: NSCoder) {
      fatalError("init(coder:) is not supported")
    }

    override func willMove(toWindow newWindow: UIWindow?) {
      super.willMove(toWindow: newWindow)
      if newWindow !== window {
        destroyZedView()
      }
    }

    override func didMoveToWindow() {
      super.didMoveToWindow()
      guard window != nil, handle == nil else { return }
      applyAppearance()
      handle = makeZedView()
      if let handle {
        didCreate(handle)
      }
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
      zed_gpui_set_dark_appearance(traitCollection.userInterfaceStyle == .dark)
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
#endif
