import CRetrofrontCore
import Foundation

public enum RetrofrontError: Error, Equatable, CustomStringConvertible {
  case coreUnavailable
  case operationFailed(String)

  public var description: String {
    switch self {
    case .coreUnavailable:
      return "Retrofront Rust core is unavailable"
    case .operationFailed(let message):
      return message
    }
  }
}

public enum FrontendState: UInt32, Equatable, Sendable {
  case empty = 0
  case coreLoaded = 1
  case gameLoaded = 2
}

public struct LibretroSystemInfo: Equatable, Sendable {
  public let libraryName: String
  public let libraryVersion: String
  public let validExtensions: [String]
  public let needsFullPath: Bool
  public let blocksExtraction: Bool
}

public enum GfxBackend: UInt32, Equatable, Sendable {
  case software = 0
  case openGL = 1
  case vulkan = 2
}

public struct VideoFrame: Equatable, Sendable {
  public let width: UInt32
  public let height: UInt32
  public let sourcePitch: UInt64
  public let pixelFormat: UInt32
  public let frameNumber: UInt64
  public let rgba: [UInt8]
}

public enum FrontendEvent: Equatable, Sendable {
  case videoFrame(width: UInt32, height: UInt32, pitch: UInt64)
  case audioBatch(frames: UInt64)
  case audioSample(left: Int16, right: Int16)
  case environmentCommand(command: UInt32, handled: Bool)
  case inputPoll
}

public final class Retrofront: @unchecked Sendable {
  private let handle: OpaquePointer

  public init() throws {
    guard let handle = rf_frontend_create() else {
      throw RetrofrontError.coreUnavailable
    }
    self.handle = handle
  }

  deinit {
    rf_frontend_destroy(handle)
  }

  public var state: FrontendState {
    FrontendState(rawValue: rf_frontend_state(handle)) ?? .empty
  }

  @discardableResult
  public func loadCore(at path: String) throws -> LibretroSystemInfo {
    let ok = path.withCString { rf_frontend_load_core(handle, $0) }
    guard ok else { throw lastError() }
    return try systemInfo()
  }

  public func loadGame(at path: String, metadata: String? = nil) throws {
    let ok = path.withCString { cPath in
      if let metadata {
        return metadata.withCString { cMeta in
          rf_frontend_load_game(handle, cPath, cMeta)
        }
      }
      return rf_frontend_load_game(handle, cPath, nil)
    }
    guard ok else { throw lastError() }
  }

  public func runFrame() throws -> [FrontendEvent] {
    guard rf_frontend_run_frame(handle) else {
      throw lastError()
    }
    return drainEvents()
  }

  public func unloadGame() {
    rf_frontend_unload_game(handle)
  }

  public func setGfxBackend(_ backend: GfxBackend) throws {
    guard rf_frontend_set_gfx_backend(handle, backend.rawValue) else {
      throw lastError()
    }
  }

  public func latestVideoFrame() -> VideoFrame? {
    var info = RfVideoFrameInfo()
    guard rf_frontend_video_frame_info(handle, &info) else { return nil }
    var rgba = [UInt8](repeating: 0, count: Int(info.rgba_len))
    let copied = rgba.withUnsafeMutableBufferPointer { buffer in
      rf_frontend_copy_video_frame_rgba(handle, buffer.baseAddress, UInt(buffer.count))
    }
    guard UInt64(copied) == info.rgba_len else { return nil }
    return VideoFrame(
      width: info.width,
      height: info.height,
      sourcePitch: info.pitch,
      pixelFormat: info.pixel_format,
      frameNumber: info.frame_number,
      rgba: rgba)
  }

  public static func openGLShaderSources() -> (vertex: String, fragment: String) {
    var vertex: UnsafePointer<CChar>?
    var fragment: UnsafePointer<CChar>?
    rf_frontend_opengl_shader_sources(&vertex, &fragment)
    return (
      vertex.map(String.init(cString:)) ?? "",
      fragment.map(String.init(cString:)) ?? "")
  }

  public func systemInfo() throws -> LibretroSystemInfo {
    var raw = RfSystemInfo()
    guard rf_frontend_system_info(handle, &raw) else {
      throw lastError()
    }
    let extensions = String(cString: raw.valid_extensions).split(separator: "|").map(String.init)
    return LibretroSystemInfo(
      libraryName: String(cString: raw.library_name),
      libraryVersion: String(cString: raw.library_version),
      validExtensions: extensions,
      needsFullPath: raw.need_fullpath,
      blocksExtraction: raw.block_extract
    )
  }

  public func drainEvents() -> [FrontendEvent] {
    var events: [FrontendEvent] = []
    var raw = RfEvent()
    while rf_frontend_next_event(handle, &raw) {
      if let event = FrontendEvent(raw) {
        events.append(event)
      }
    }
    return events
  }

  private func lastError() -> RetrofrontError {
    guard let pointer = rf_frontend_last_error(handle) else {
      return .operationFailed("unknown Retrofront core error")
    }
    let message = String(cString: pointer)
    return .operationFailed(message.isEmpty ? "unknown Retrofront core error" : message)
  }
}

extension FrontendEvent {
  fileprivate init?(_ raw: RfEvent) {
    switch raw.kind {
    case 1:
      self = .videoFrame(width: UInt32(raw.a), height: UInt32(raw.b), pitch: raw.c)
    case 2:
      self = .audioBatch(frames: raw.a)
    case 3:
      self = .audioSample(
        left: Int16(bitPattern: UInt16(truncatingIfNeeded: raw.a)),
        right: Int16(bitPattern: UInt16(truncatingIfNeeded: raw.b)))
    case 4:
      self = .environmentCommand(command: UInt32(raw.a), handled: raw.b != 0)
    case 5:
      self = .inputPoll
    default:
      return nil
    }
  }
}
