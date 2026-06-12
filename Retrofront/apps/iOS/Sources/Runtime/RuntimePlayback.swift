import Foundation
import RetrofrontSwift
import UIKit

extension EmulatorRuntimeModel {
  public func toggleRunning() { isRunning ? stop() : play() }

  public func play() {
    guard frontendState == .gameLoaded, !isRunning else { return }
    isRunning = true
    runTask = Task.detached(priority: .userInitiated) { [weak self] in
      while !Task.isCancelled {
        guard let self = self else { break }
        let shouldStop = await MainActor.run { self.runOneFrame() }
        if shouldStop { break }
        try? await Task.sleep(nanoseconds: 16_666_667)
      }
    }
  }

  public func stop() {
    isRunning = false
    runTask?.cancel()
    runTask = nil
    try? frontend?.saveSRAM()
  }

  public func resetContent() {
    do {
      try frontend?.reset()
      statusMessage = "Game reset"
    } catch {
      statusMessage = "Reset failed: \(error)"
    }
    refresh()
  }

  public func saveSRAMNow() {
    do {
      try frontend?.saveSRAM()
      statusMessage = "SRAM saved"
    } catch {
      statusMessage = "SRAM save failed: \(error)"
    }
    refresh()
  }

  public var activeStateSlot: UInt32 {
    UInt32(max(0, min(999, Int(settingValue("state_slot")) ?? 0)))
  }

  public var stateSlotLabel: String {
    settingValue("state_slot") == "-1" ? "Auto" : String(activeStateSlot)
  }

  public func saveState(slot: UInt32? = nil) {
    let targetSlot = slot ?? activeStateSlot
    do {
      try frontend?.saveState(slot: targetSlot)
      statusMessage = "State saved to slot \(targetSlot)"
    } catch {
      statusMessage = "Save state failed: \(error)"
    }
    refresh()
  }

  public func loadState(slot: UInt32? = nil) {
    let targetSlot = slot ?? activeStateSlot
    do {
      try frontend?.loadState(slot: targetSlot)
      statusMessage = "State loaded from slot \(targetSlot)"
    } catch {
      statusMessage = "Load state failed: \(error)"
    }
    refresh()
  }

  public func closeContent() {
    stop()
    frontend?.unloadGame()
    loadedGameURL = nil
    displayImage = nil
    frontendState = frontend?.state ?? .empty
    refresh()
    statusMessage = "Game exited"
  }

  @discardableResult
  func runOneFrame() -> Bool {
    guard let frontend else { return true }
    do {
      _ = try frontend.runFrame()
      if let info = frontend.latestVideoFrameInfo() {
        if pixelBuffer == nil || pixelBuffer?.count != Int(info.rgbaLen) {
          pixelBuffer = Data(count: Int(info.rgbaLen))
        }
        let copied = pixelBuffer?.withUnsafeMutableBytes { buffer -> Int in
          guard let base = buffer.baseAddress else { return 0 }
          return frontend.copyLatestVideoFrame(to: base, length: buffer.count)
        } ?? 0
        if copied == Int(info.rgbaLen), let data = pixelBuffer {
          displayImage = Self.imageFromData(data, width: Int(info.width), height: Int(info.height))
        }
      }
      return false
    } catch {
      Task { @MainActor in
        self.stop()
        self.statusMessage = "Run error: \(error)"
      }
      return true
    }
  }

  public func refresh() {
    guard let frontend else { return }
    frontendState = frontend.state
    coreOptions = frontend.coreOptions()
    settings = frontend.settings()
    if let config = frontend.gfxVideoConfig() {
      if config.aspectRatio > 0 { aspectRatio = Double(config.aspectRatio) }
      else if config.baseHeight > 0 { aspectRatio = Double(config.baseWidth) / Double(config.baseHeight) }
    }
    refreshMenu()
    overlayInfo = frontend.overlayInfo()
  }

  public func refreshMenu() { currentMenu = frontend?.currentMenuList() }
  static func imageFromData(_ data: Data, width: Int, height: Int) -> UIImage? {
    guard width > 0, height > 0 else { return nil }
    guard let provider = CGDataProvider(data: data as CFData) else { return nil }
    guard let cgImage = CGImage(
      width: width,
      height: height,
      bitsPerComponent: 8,
      bitsPerPixel: 32,
      bytesPerRow: width * 4,
      space: CGColorSpaceCreateDeviceRGB(),
      bitmapInfo: CGBitmapInfo.byteOrder32Big.union(CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue)),
      provider: provider,
      decode: nil,
      shouldInterpolate: false,
      intent: .defaultIntent
    ) else { return nil }
    return UIImage(cgImage: cgImage)
  }
}
