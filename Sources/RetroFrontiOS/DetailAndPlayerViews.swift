#if canImport(SwiftUI)
import SwiftUI
import GameController
import CoreGraphics
#if canImport(AVFoundation)
import AVFoundation
#endif
import RetroFrontCore

struct GameDetailView: View {
    @EnvironmentObject private var model: FrontendViewModel
    let game: Game
    var compatibleCores: [LibretroCore] { model.cores.filter { $0.systemIDs.contains(game.systemID) || $0.supportedExtensions.contains(game.fileURL.pathExtension.lowercased()) } }

    var body: some View {
        List {
            Section {
                HStack(alignment: .top, spacing: 18) {
                    ArtworkView(url: game.artworkURL, title: game.title).frame(width: 140, height: 200)
                    VStack(alignment: .leading, spacing: 8) {
                        Text(game.title).font(.title2.bold())
                        Text(SystemCatalog.system(id: game.systemID)?.name ?? game.systemID).foregroundStyle(.secondary)
                        if let year = game.releaseYear { Text(String(year)) }
                        Button("Toggle Favorite", systemImage: game.favorite ? "star.fill" : "star") { Task { await model.toggleFavorite(game) } }
                    }
                }
            }
            Section("Play") {
                ForEach(compatibleCores) { core in
                    NavigationLink { PlayerView(game: game, core: core) } label: { Label(core.displayName, systemImage: "play.circle.fill") }
                }
                if compatibleCores.isEmpty { ContentUnavailableView("No compatible core", systemImage: "cpu", description: Text("Import a bundled libretro core that supports .\(game.fileURL.pathExtension).")) }
            }
            Section("Metadata") {
                LabeledContent("Path", value: game.fileURL.lastPathComponent)
                LabeledContent("CRC32", value: game.crc32 ?? "Unknown")
                LabeledContent("Play time", value: game.playTime.formatted())
            }
            Section("Cheats & patches") {
                NavigationLink("Cheat codes") { CheatsView(game: game) }
                NavigationLink("Soft patches") { SoftPatchInfoView(game: game) }
            }
        }
        .navigationTitle(game.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}

struct PlayerView: View {
    @EnvironmentObject private var model: FrontendViewModel
    let game: Game
    let core: LibretroCore
    @StateObject private var session = PlayerSession()
    @State private var showMenu = false

    var body: some View {
        ZStack {
            FrameView(frame: session.currentFrame, settings: model.settings).ignoresSafeArea().background(.black)
            TouchOverlay(input: session.input, preset: model.settings.overlayPreset).ignoresSafeArea()
            VStack {
                HStack {
                    Button { showMenu = true } label: { Image(systemName: "line.3.horizontal.circle.fill").font(.largeTitle).symbolRenderingMode(.hierarchical) }.padding()
                    Spacer()
                    if session.fastForward { Label("FF", systemImage: "forward.fill").padding(8).background(.thinMaterial).clipShape(Capsule()) }
                }
                Spacer()
            }
        }
        .navigationBarBackButtonHidden(true)
        .task { await session.start(game: game, core: core, settings: model.settings, cheats: model.cheats.filter { $0.gameID == game.id }, store: model.store) }
        .onDisappear { Task { await session.stop(store: model.store, gameID: game.id, coreID: core.id) } }
        .sheet(isPresented: $showMenu) { PauseMenu(session: session, game: game, core: core, store: model.store) }
    }
}

@MainActor
final class PlayerSession: ObservableObject {
    @Published var currentFrame: VideoFrame?
    @Published var isRunning = false
    @Published var fastForward = false
    @Published var log: [String] = []
    @Published var stateSlot = 0
    let input = ControllerInputState()
    private let runtime = LibretroRuntime()
    private var task: Task<Void, Never>?
    private var controllerTask: Task<Void, Never>?
    private var rewindTask: Task<Void, Never>?
    private var rewindBuffer: [Data] = []
    private var startDate = Date()
    private var settings = FrontendSettings()
    private var directories: FrontendDirectories?
#if canImport(AVFoundation)
    private let audioOutput = PCMOutput()
#endif

    func start(game: Game, core: LibretroCore, settings: FrontendSettings, cheats: [CheatCode], store: LibraryStore?) async {
        guard !isRunning else { return }
        self.settings = settings
        self.directories = await store?.directories
        do {
            runtime.onVideoFrame = { [weak self] frame in Task { @MainActor in self?.currentFrame = frame } }
#if canImport(AVFoundation)
            let audioOutput = self.audioOutput
            runtime.onAudio = { data, frames in audioOutput.enqueue(data: data, frames: frames) }
#else
            runtime.onAudio = { _, _ in }
#endif
            runtime.inputState = { [weak input] port, device, index, id in input?.state(port: port, device: device, index: index, id: id) ?? 0 }
            try runtime.open(coreAt: core.path)
            if let directories { runtime.configureDirectories(system: directories.system, save: directories.saves, content: game.fileURL.deletingLastPathComponent()) }
            try runtime.initialize()
            let patchResult = try directories.map { try SoftPatchManager().preparedContentURL(for: game, workDirectory: $0.imports) }
            patchResult?.messages.forEach { log.append($0) }
            try runtime.load(gameAt: patchResult?.patchedURL ?? game.fileURL, needsFullPath: core.requiresFullPath)
            apply(cheats: cheats)
#if canImport(AVFoundation)
            audioOutput.start(sampleRate: max(runtime.avInfo().sampleRate, 44_100))
#endif
            isRunning = true
            startDate = Date()
            startControllerPolling()
            startRewindCapture()
            let frameInterval = max(1_000_000, UInt64(1_000_000_000 / max(runtime.avInfo().fps, 30)))
            task = Task.detached(priority: .userInitiated) { [weak self, runtime] in
                while !Task.isCancelled {
                    runtime.runFrame()
                    let speed = await MainActor.run { self?.fastForward == true ? max(self?.settings.fastForwardRate ?? 2, 1) : 1 }
                    try? await Task.sleep(nanoseconds: UInt64(Double(frameInterval) / speed))
                }
            }
        } catch { log.append(error.localizedDescription) }
    }

    func stop(store: LibraryStore?, gameID: UUID, coreID: UUID) async {
        task?.cancel(); controllerTask?.cancel(); rewindTask?.cancel()
#if canImport(AVFoundation)
        audioOutput.stop()
#endif
        if settings.autoSaveOnBackground, let directories, let state = try? quickSave(gameID: gameID, coreID: coreID, directories: directories, slot: 99) { try? await store?.add(saveState: state) }
        runtime.unloadGame(); runtime.close(); isRunning = false
        try? await store?.markPlayed(gameID: gameID, additionalPlayTime: Date().timeIntervalSince(startDate))
    }

    func reset() { runtime.reset() }
    func saveState(to url: URL) throws { try runtime.serialize().write(to: url, options: .atomic) }
    func loadState(from url: URL) throws { try runtime.unserialize(Data(contentsOf: url)) }
    func quickSave(gameID: UUID, coreID: UUID, directories: FrontendDirectories, slot: Int? = nil) throws -> SaveState {
        let targetSlot = slot ?? stateSlot
        let url = directories.states.appendingPathComponent("\(gameID.uuidString)-\(coreID.uuidString)-\(targetSlot).state")
        try saveState(to: url)
        return SaveState(id: UUID(), gameID: gameID, coreID: coreID, slot: targetSlot, createdAt: Date(), stateURL: url, thumbnailURL: nil, note: "Slot \(targetSlot)")
    }
    func rewind() { guard let state = rewindBuffer.popLast() else { log.append("No rewind state captured yet."); return }; do { try runtime.unserialize(state) } catch { log.append(error.localizedDescription) } }
    func apply(cheats: [CheatCode]) { runtime.resetCheats(); for (index, cheat) in cheats.filter(\.enabled).enumerated() { runtime.setCheat(index: index, enabled: true, code: cheat.code) } }

    private func startRewindCapture() {
        guard settings.rewindEnabled else { return }
        rewindTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 1_000_000_000)
                await MainActor.run {
                    guard let self, self.isRunning, let state = try? self.runtime.serialize() else { return }
                    self.rewindBuffer.append(state)
                    let maxCount = max(1, self.settings.rewindBufferSeconds)
                    if self.rewindBuffer.count > maxCount { self.rewindBuffer.removeFirst(self.rewindBuffer.count - maxCount) }
                }
            }
        }
    }

    private func startControllerPolling() {
        GCController.startWirelessControllerDiscovery(completionHandler: nil)
        controllerTask = Task { [weak input] in
            while !Task.isCancelled {
                if let pad = GCController.controllers().first?.extendedGamepad {
                    input?.set(4, pressed: pad.dpad.up.isPressed); input?.set(5, pressed: pad.dpad.down.isPressed); input?.set(6, pressed: pad.dpad.left.isPressed); input?.set(7, pressed: pad.dpad.right.isPressed)
                    input?.set(0, pressed: pad.buttonA.isPressed); input?.set(1, pressed: pad.buttonB.isPressed); input?.set(9, pressed: pad.buttonY.isPressed); input?.set(10, pressed: pad.buttonX.isPressed)
                    input?.set(8, pressed: pad.buttonMenu.isPressed); input?.set(11, pressed: pad.leftShoulder.isPressed); input?.set(12, pressed: pad.rightShoulder.isPressed)
                }
                try? await Task.sleep(nanoseconds: 8_000_000)
            }
        }
    }
}

final class ControllerInputState: ObservableObject {
    @Published var pressed: Set<Int> = []
    func set(_ id: Int, pressed: Bool) { if pressed { self.pressed.insert(id) } else { self.pressed.remove(id) } }
    func state(port: UInt32, device: UInt32, index: UInt32, id: UInt32) -> Int16 { pressed.contains(Int(id)) ? 1 : 0 }
}

struct FrameView: View {
    let frame: VideoFrame?
    let settings: FrontendSettings
    var body: some View {
        GeometryReader { proxy in
            if let frame, let image = frame.cgImage(shaderPreset: settings.shaderPreset) {
                Image(decorative: image, scale: 1).resizable().modifier(AspectModifier(settings: settings)).frame(width: proxy.size.width, height: proxy.size.height).background(.black)
            } else { ProgressView("Loading core…").frame(width: proxy.size.width, height: proxy.size.height).tint(.white) }
        }
    }
}

struct AspectModifier: ViewModifier {
    let settings: FrontendSettings
    @ViewBuilder
    func body(content: Content) -> some View {
        switch settings.aspectRatio {
        case .stretch, .fullscreen: content.scaledToFill()
        default: content.scaledToFit()
        }
    }
}

extension VideoFrame {
    func cgImage(shaderPreset: String) -> CGImage? {
        let converted = shaderedRGBA(preset: shaderPreset)
        let provider = CGDataProvider(data: converted as CFData)
        return provider.flatMap { CGImage(width: width, height: height, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: width * 4, space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue), provider: $0, decode: nil, shouldInterpolate: false, intent: .defaultIntent) }
    }

    private func shaderedRGBA(preset: String) -> Data {
        var data = rgba8888
        guard !preset.localizedCaseInsensitiveContains("No shader") else { return data }
        data.withUnsafeMutableBytes { raw in
            guard let pixels = raw.bindMemory(to: UInt8.self).baseAddress else { return }
            for y in 0..<height {
                for x in 0..<width {
                    let offset = (y * width + x) * 4
                    if preset.localizedCaseInsensitiveContains("Scanlines") || preset.localizedCaseInsensitiveContains("CRT") {
                        if y % 2 == 1 { pixels[offset] = UInt8(Double(pixels[offset]) * 0.72); pixels[offset + 1] = UInt8(Double(pixels[offset + 1]) * 0.72); pixels[offset + 2] = UInt8(Double(pixels[offset + 2]) * 0.72) }
                    }
                    if preset.localizedCaseInsensitiveContains("LCD") {
                        if x % 3 == 0 { pixels[offset + 1] = UInt8(Double(pixels[offset + 1]) * 0.84); pixels[offset + 2] = UInt8(Double(pixels[offset + 2]) * 0.84) }
                        if x % 3 == 1 { pixels[offset] = UInt8(Double(pixels[offset]) * 0.84); pixels[offset + 2] = UInt8(Double(pixels[offset + 2]) * 0.84) }
                        if x % 3 == 2 { pixels[offset] = UInt8(Double(pixels[offset]) * 0.84); pixels[offset + 1] = UInt8(Double(pixels[offset + 1]) * 0.84) }
                    }
                }
            }
        }
        return data
    }
}

struct TouchOverlay: View {
    @ObservedObject var input: ControllerInputState
    let preset: String
    var body: some View { VStack { Spacer(); HStack(alignment: .bottom) { DPad(input: input); Spacer(); FaceButtons(input: input) }.padding(28).opacity(preset == "Minimal" ? 0.55 : 0.82) } }
}

struct DPad: View { @ObservedObject var input: ControllerInputState; var body: some View { VStack { PadButton("▲", id: 4, input: input); HStack { PadButton("◀", id: 6, input: input); PadButton("●", id: 8, input: input); PadButton("▶", id: 7, input: input) }; PadButton("▼", id: 5, input: input) } } }
struct FaceButtons: View { @ObservedObject var input: ControllerInputState; var body: some View { VStack { PadButton("Y", id: 9, input: input); HStack { PadButton("X", id: 10, input: input); PadButton("A", id: 0, input: input) }; PadButton("B", id: 1, input: input); HStack { PadButton("L", id: 11, input: input); PadButton("R", id: 12, input: input) } } } }

struct PadButton: View {
    let title: String; let id: Int; @ObservedObject var input: ControllerInputState
    init(_ title: String, id: Int, input: ControllerInputState) { self.title = title; self.id = id; self.input = input }
    var body: some View { Text(title).font(.headline).frame(width: 58, height: 58).background(.ultraThinMaterial).clipShape(Circle()).gesture(DragGesture(minimumDistance: 0).onChanged { _ in input.set(id, pressed: true) }.onEnded { _ in input.set(id, pressed: false) }) }
}

struct PauseMenu: View {
    @ObservedObject var session: PlayerSession
    let game: Game; let core: LibretroCore; let store: LibraryStore?
    @Environment(\.dismiss) private var dismiss
    var body: some View {
        NavigationStack { List {
            Section { Button("Resume", systemImage: "play.fill") { dismiss() }; Button("Reset", systemImage: "arrow.counterclockwise") { session.reset() }; Toggle("Fast forward", isOn: $session.fastForward); Button("Rewind one second", systemImage: "gobackward") { session.rewind() } }
            Section("Save States") { Stepper("Slot \(session.stateSlot)", value: $session.stateSlot, in: 0...9); Button("Quick Save", systemImage: "square.and.arrow.down") { quickSave() }; Button("Quick Load", systemImage: "square.and.arrow.up") { quickLoad() } }
            Section("Video") { Label("Shader preset: \(session.currentFrame == nil ? "loading" : "active")", systemImage: "camera.filters"); Label("Aspect ratio and scaling come from Settings", systemImage: "rectangle.inset.filled") }
            Section("Session") { Label("Netplay room configuration is available in Settings", systemImage: "network"); Label("RetroAchievements account is available in Settings", systemImage: "trophy") }
            if !session.log.isEmpty { Section("Log") { ForEach(session.log, id: \.self) { Text($0) } } }
        }.navigationTitle("Paused") }
    }
    private func quickSave() { Task { guard let directories = await store?.directories else { return }; do { let state = try session.quickSave(gameID: game.id, coreID: core.id, directories: directories); try await store?.add(saveState: state) } catch { session.log.append(error.localizedDescription) } } }
    private func quickLoad() { Task { let states = await store?.saveStates ?? []; guard let state = states.first(where: { $0.gameID == game.id && $0.coreID == core.id && $0.slot == session.stateSlot }) else { session.log.append("No state in slot \(session.stateSlot)."); return }; do { try session.loadState(from: state.stateURL) } catch { session.log.append(error.localizedDescription) } } }
}

struct SoftPatchInfoView: View {
    let game: Game
    var body: some View {
        List {
            Section("Supported soft patches") { Text("IPS patches are applied automatically when a .ips file with the same base name sits next to the ROM. BPS/UPS can still be handled by cores that provide native soft-patch support.") }
            Section("Expected file") { Text(game.fileURL.deletingPathExtension().lastPathComponent + ".ips") }
        }.navigationTitle("Soft patches")
    }
}

#if canImport(AVFoundation)
final class PCMOutput: @unchecked Sendable {
    private let engine = AVAudioEngine()
    private let player = AVAudioPlayerNode()
    private var format = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: 44_100, channels: 2, interleaved: false)!
    private let lock = NSLock()
    private var attached = false

    func start(sampleRate: Double) {
        lock.lock(); defer { lock.unlock() }
        format = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: sampleRate, channels: 2, interleaved: false)!
        if !attached { engine.attach(player); attached = true }
        engine.connect(player, to: engine.mainMixerNode, format: format)
        if !engine.isRunning { try? engine.start() }
        if !player.isPlaying { player.play() }
    }

    func enqueue(data: Data, frames: Int) {
        lock.lock(); defer { lock.unlock() }
        guard engine.isRunning, frames > 0, let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(frames)) else { return }
        buffer.frameLength = AVAudioFrameCount(frames)
        data.withUnsafeBytes { raw in
            guard let source = raw.bindMemory(to: Int16.self).baseAddress, let left = buffer.floatChannelData?[0], let right = buffer.floatChannelData?[1] else { return }
            for frame in 0..<frames {
                left[frame] = Float(source[frame * 2]) / Float(Int16.max)
                right[frame] = Float(source[frame * 2 + 1]) / Float(Int16.max)
            }
        }
        player.scheduleBuffer(buffer, completionHandler: nil)
    }

    func stop() {
        lock.lock(); defer { lock.unlock() }
        player.stop()
        engine.stop()
    }
}
#endif
#endif
