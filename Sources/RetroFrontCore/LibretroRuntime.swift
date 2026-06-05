import Foundation
import CLibretroHost

public struct CoreSystemInfo: Sendable, Hashable {
    public var name: String
    public var version: String
    public var validExtensions: [String]
    public var needsFullPath: Bool
}

public struct CoreAVInfo: Sendable, Hashable {
    public var baseWidth: Int
    public var baseHeight: Int
    public var maxWidth: Int
    public var maxHeight: Int
    public var aspectRatio: Float
    public var fps: Double
    public var sampleRate: Double
}

public enum LibretroPixelFormat: UInt32, Codable, Sendable {
    case rgb1555 = 0
    case xrgb8888 = 1
    case rgb565 = 2
}

public struct VideoFrame: Sendable {
    public var width: Int
    public var height: Int
    public var pitch: Int
    public var pixelFormat: UInt32
    public var bytes: Data

    public init(width: Int, height: Int, pitch: Int, pixelFormat: UInt32, bytes: Data) {
        self.width = width
        self.height = height
        self.pitch = pitch
        self.pixelFormat = pixelFormat
        self.bytes = bytes
    }

    public var rgba8888: Data {
        var output = Data(count: max(width * height * 4, 0))
        guard width > 0, height > 0, pitch > 0 else { return output }
        output.withUnsafeMutableBytes { dstRaw in
            guard let dst = dstRaw.bindMemory(to: UInt8.self).baseAddress else { return }
            bytes.withUnsafeBytes { srcRaw in
                guard let src = srcRaw.bindMemory(to: UInt8.self).baseAddress else { return }
                for y in 0..<height {
                    let row = src + y * pitch
                    for x in 0..<width {
                        let out = dst + (y * width + x) * 4
                        switch LibretroPixelFormat(rawValue: pixelFormat) ?? .rgb1555 {
                        case .rgb1555:
                            let value = UInt16(row[x * 2]) | (UInt16(row[x * 2 + 1]) << 8)
                            out[0] = UInt8(((value >> 10) & 0x1F) * 255 / 31)
                            out[1] = UInt8(((value >> 5) & 0x1F) * 255 / 31)
                            out[2] = UInt8((value & 0x1F) * 255 / 31)
                            out[3] = 255
                        case .xrgb8888:
                            out[0] = row[x * 4 + 2]
                            out[1] = row[x * 4 + 1]
                            out[2] = row[x * 4]
                            out[3] = 255
                        case .rgb565:
                            let value = UInt16(row[x * 2]) | (UInt16(row[x * 2 + 1]) << 8)
                            out[0] = UInt8(((value >> 11) & 0x1F) * 255 / 31)
                            out[1] = UInt8(((value >> 5) & 0x3F) * 255 / 63)
                            out[2] = UInt8((value & 0x1F) * 255 / 31)
                            out[3] = 255
                        }
                    }
                }
            }
        }
        return output
    }
}

public final class LibretroRuntime: @unchecked Sendable {
    private var handle: OpaquePointer?
    private var retainedSelf: Unmanaged<LibretroRuntime>?
    public var onVideoFrame: (@Sendable (VideoFrame) -> Void)?
    public var onAudio: (@Sendable (Data, Int) -> Void)?
    public var inputState: (@Sendable (_ port: UInt32, _ device: UInt32, _ index: UInt32, _ id: UInt32) -> Int16)?
    public private(set) var log: [String] = []

    public init() {}
    deinit { close() }

    public func open(coreAt url: URL) throws {
        close()
        retainedSelf = Unmanaged.passUnretained(self)
        handle = rf_core_open(url.path, { message, context in
            guard let context else { return }
            let runtime = Unmanaged<LibretroRuntime>.fromOpaque(context).takeUnretainedValue()
            runtime.log.append(message.map { String(cString: $0) } ?? "")
        }, retainedSelf?.toOpaque())
        guard rf_core_is_open(handle) else { throw RuntimeError.coreOpenFailed(lastError) }
        rf_core_set_callbacks(handle, { frame, context in
            guard let frame, let context, let data = frame.pointee.data else { return }
            let runtime = Unmanaged<LibretroRuntime>.fromOpaque(context).takeUnretainedValue()
            let byteCount = Int(frame.pointee.pitch) * Int(frame.pointee.height)
            runtime.onVideoFrame?(VideoFrame(width: Int(frame.pointee.width), height: Int(frame.pointee.height), pitch: Int(frame.pointee.pitch), pixelFormat: UInt32(frame.pointee.pixel_format), bytes: Data(bytes: data, count: byteCount)))
        }, { samples, frames, context in
            guard let samples, let context else { return }
            let runtime = Unmanaged<LibretroRuntime>.fromOpaque(context).takeUnretainedValue()
            runtime.onAudio?(Data(bytes: samples, count: Int(frames) * MemoryLayout<Int16>.size * 2), Int(frames))
        }, { port, device, index, id, context in
            guard let context else { return 0 }
            let runtime = Unmanaged<LibretroRuntime>.fromOpaque(context).takeUnretainedValue()
            return runtime.inputState?(port, device, index, id) ?? 0
        }, retainedSelf?.toOpaque())
    }

    public func configureDirectories(system: URL?, save: URL?, content: URL?) {
        rf_core_set_directories(handle, system?.path, save?.path, content?.path)
    }

    public func setCoreOption(key: String, value: String) {
        rf_core_set_variable(handle, key, value)
    }

    public func initialize() throws { guard rf_core_init(handle) else { throw RuntimeError.initializationFailed(lastError) } }

    public func discoveredOptions() -> [CoreOption] {
        let count = rf_core_variable_count(handle)
        guard count > 0 else { return [] }
        return (0..<count).compactMap { index in
            let variable = rf_core_get_variable(handle, index)
            guard let keyPointer = variable.key else { return nil }
            let key = String(cString: keyPointer)
            let title = variable.description.map(String.init(cString:)) ?? key
            let values = (variable.values.map(String.init(cString:)) ?? "").split(separator: "|").map(String.init)
            let defaultValue = variable.value.map(String.init(cString:)) ?? values.first ?? ""
            return CoreOption(key: key, title: title, values: values, defaultValue: defaultValue)
        }
    }
    public func systemInfo() -> CoreSystemInfo {
        let info = rf_core_get_system_info(handle)
        return CoreSystemInfo(name: info.library_name.map(String.init(cString:)) ?? "Unknown", version: info.library_version.map(String.init(cString:)) ?? "", validExtensions: (info.valid_extensions.map(String.init(cString:)) ?? "").split(separator: "|").map(String.init), needsFullPath: info.need_fullpath)
    }
    public func avInfo() -> CoreAVInfo {
        let info = rf_core_get_av_info(handle)
        return CoreAVInfo(baseWidth: Int(info.geometry_base_width), baseHeight: Int(info.geometry_base_height), maxWidth: Int(info.geometry_max_width), maxHeight: Int(info.geometry_max_height), aspectRatio: info.geometry_aspect_ratio, fps: info.timing_fps, sampleRate: info.timing_sample_rate)
    }
    public func load(gameAt url: URL, needsFullPath: Bool) throws {
        let data = needsFullPath ? nil : try? Data(contentsOf: url)
        let ok: Bool = if let data { data.withUnsafeBytes { rf_core_load_game(handle, url.path, $0.baseAddress, data.count) } } else { rf_core_load_game(handle, url.path, nil, 0) }
        guard ok else { throw RuntimeError.gameLoadFailed(lastError) }
    }
    public func runFrame() { rf_core_run(handle) }
    public func reset() { rf_core_reset(handle) }
    public func resetCheats() { rf_core_cheat_reset(handle) }
    public func setCheat(index: Int, enabled: Bool, code: String) { rf_core_set_cheat(handle, UInt32(index), enabled, code) }
    public func serialize() throws -> Data {
        let size = rf_core_serialize_size(handle)
        guard size > 0 else { throw RuntimeError.serializationUnavailable }
        var data = Data(count: size)
        let ok = data.withUnsafeMutableBytes { rf_core_serialize(handle, $0.baseAddress, size) }
        guard ok else { throw RuntimeError.serializationFailed(lastError) }
        return data
    }
    public func unserialize(_ data: Data) throws {
        let ok = data.withUnsafeBytes { rf_core_unserialize(handle, $0.baseAddress, data.count) }
        guard ok else { throw RuntimeError.serializationFailed(lastError) }
    }
    public func unloadGame() { rf_core_unload_game(handle) }
    public func close() { if handle != nil { rf_core_deinit(handle); rf_core_close(handle); handle = nil } }
    private var lastError: String { String(cString: rf_core_last_error(handle)) }

    public enum RuntimeError: Error, LocalizedError, Sendable {
        case coreOpenFailed(String), initializationFailed(String), gameLoadFailed(String), serializationUnavailable, serializationFailed(String)
        public var errorDescription: String? {
            switch self {
            case .coreOpenFailed(let message): "Could not open libretro core: \(message)"
            case .initializationFailed(let message): "Could not initialize libretro core: \(message)"
            case .gameLoadFailed(let message): "Could not load game: \(message)"
            case .serializationUnavailable: "This core does not expose save-state serialization."
            case .serializationFailed(let message): "Save-state operation failed: \(message)"
            }
        }
    }
}
