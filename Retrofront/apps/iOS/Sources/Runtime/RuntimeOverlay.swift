import Foundation
import RetrofrontSwift
import UIKit

extension EmulatorRuntimeModel {
  public func setJoypadButton(_ button: JoypadButton, pressed: Bool) {
    try? frontend?.setJoypadButton(button, pressed: pressed)
  }

  public func setOverlayTouch(slot: Int, location: CGPoint, in size: CGSize, active: Bool) {
    guard size.width > 0, size.height > 0 else { return }
    let x = Float(max(0, min(1, location.x / size.width)))
    let y = Float(max(0, min(1, location.y / size.height)))
    try? frontend?.setOverlayTouch(slot: slot, x: x, y: y, active: active)
    if frontend?.consumeOverlayMenuToggle() == true {
      menuAction(8)
      menuToken &+= 1
    }
  }

  public func clearOverlayTouches() {
    frontend?.clearOverlayTouches()
  }

  func loadConfiguredOverlay(_ frontend: Retrofront) {
    let enabled = frontend.setting("input_overlay_enable") != "false"
    if let path = frontend.setting("input_overlay"), FileManager.default.fileExists(atPath: path) {
      try? frontend.loadOverlay(at: path)
    }
    frontend.setOverlayEnabled(enabled)
    overlayInfo = frontend.overlayInfo()
  }

  public func overlayRenderDescs() -> [OverlayRenderDesc] {
    frontend?.overlayRenderDescs() ?? []
  }

  public func setOverlayOrientation(for size: CGSize) {
    guard size.width > 0, size.height > 0 else { return }
    do {
      try frontend?.setOverlayOrientation(portrait: size.height > size.width)
      overlayInfo = frontend?.overlayInfo()
    } catch {
      // Some overlays do not define portrait/landscape variants; keep the current overlay.
    }
  }

  public func refreshOverlayChoices() {
    guard let frontend else { return }
    let overlayDir = URL(fileURLWithPath: frontend.setting("overlay_directory") ?? storageLayout.overlaysDirectory.path)
    let fm = FileManager.default
    guard let enumerator = fm.enumerator(at: overlayDir, includingPropertiesForKeys: nil) else {
      availableOverlays = []
      return
    }
    var choices: [OverlayChoice] = []
    for case let url as URL in enumerator where url.pathExtension.lowercased() == "cfg" {
      let relative = url.path.replacingOccurrences(of: overlayDir.path + "/", with: "")
      let isGamepad = relative.contains("gamepads/")
      let label = relative.replacingOccurrences(of: ".cfg", with: "")
      choices.append(OverlayChoice(id: url.path, path: url.path, label: isGamepad ? label : "Other / \(label)"))
    }
    availableOverlays = choices.sorted { left, right in
      let leftGamepad = left.label.hasPrefix("gamepads/")
      let rightGamepad = right.label.hasPrefix("gamepads/")
      if leftGamepad != rightGamepad { return leftGamepad }
      return left.label.localizedStandardCompare(right.label) == .orderedAscending
    }
  }

  func applyVideoSettings(_ frontend: Retrofront) {
    var config = frontend.gfxVideoConfig() ?? GfxVideoConfig(baseWidth: 0, baseHeight: 0)
    config = GfxVideoConfig(
      baseWidth: config.baseWidth,
      baseHeight: config.baseHeight,
      maxWidth: config.maxWidth,
      maxHeight: config.maxHeight,
      aspectRatio: config.aspectRatio,
      outputWidth: config.outputWidth,
      outputHeight: config.outputHeight,
      scaleMode: scaleModeFromSetting(frontend.setting("video_scale_mode")),
      filterMode: filterModeFromSetting(frontend.setting("video_filter_mode")),
      rotationQuarters: config.rotationQuarters,
      vsync: frontend.setting("video_vsync") != "false")
    try? frontend.setGfxVideoConfig(config)
  }
}
