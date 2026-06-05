#if canImport(SwiftUI)
import SwiftUI
import GameController
import RetroFrontCore

struct GameDetailView: View {
    @EnvironmentObject private var model: FrontendViewModel
    let game: Game
    var compatibleCores: [LibretroCore] { model.cores.filter { $0.systemIDs.contains(game.systemID) || $0.supportedExtensions.contains(game.fileURL.pathExtension.lowercased()) } }
    var body: some View {
        List {
            Section { HStack(alignment: .top, spacing: 18) { ArtworkView(url: game.artworkURL, title: game.title).frame(width: 140, height: 200); VStack(alignment: .leading, spacing: 8) { Text(game.title).font(.title2.bold()); Text(SystemCatalog.system(id: game.systemID)?.name ?? game.systemID).foregroundStyle(.secondary); if let year = game.releaseYear { Text(String(year)) }; Button("Toggle Favorite", systemImage: game.favorite ? "star.fill" : "star") { Task { await model.toggleFavorite(game) } } } } }
            Section("Play") {
                ForEach(compatibleCores) { core in NavigationLink { PlayerView(game: game, core: core) } label: { Label(core.displayName, systemImage: "play.circle.fill") } }
                if compatibleCores.isEmpty { ContentUnavailableView("No compatible core", systemImage: "cpu", description: Text("Import a bundled libretro core that supports .\(game.fileURL.pathExtension).")) }
            }
            Section("Metadata") { LabeledContent("Path", value: game.fileURL.lastPathComponent); LabeledContent("CRC32", value: game.crc32 ?? "Unknown"); LabeledContent("Play time", value: game.playTime.formatted()) }
            Section("Cheats & patches") { NavigationLink("Cheat codes") { CheatsView(game: game) }; NavigationLink("Soft patches") { Text("Place IPS/BPS/UPS patches next to the ROM with the same base name.") } }
        }.navigationTitle(game.title).navigationBarTitleDisplayMode(.inline)
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
            FrameView(frame: session.currentFrame).ignoresSafeArea().background(.black)
            TouchOverlay(input: session.input).ignoresSafeArea()
            VStack { HStack { Button { showMenu = true } label: { Image(systemName: "line.3.horizontal.circle.fill").font(.largeTitle).symbolRenderingMode(.hierarchical) }.padding(); Spacer(); if session.fastForward { Label("FF", systemImage: "forward.fill").padding(8).background(.thinMaterial).clipShape(Capsule()) } }; Spacer() }
        }
        .navigationBarBackButtonHidden(true)
        .task { await session.start(game: game, core: core, settings: model.settings) }
        .onDisappear { Task { await session.stop() } }
        .sheet(isPresented: $showMenu) { PauseMenu(session: session, game: game, core: core) }
    }
}

@MainActor
final class PlayerSession: ObservableObject {
    @Published var currentFrame: VideoFrame?
    @Published var isRunning = false
    @Published var fastForward = false
    @Published var log: [String] = []
    let input = ControllerInputState()
    private let runtime = LibretroRuntime()
    private var task: Task<Void, Never>?

    func start(game: Game, core: LibretroCore, settings: FrontendSettings) async {
        guard !isRunning else { return }
        do {
            runtime.onVideoFrame = { [weak self] frame in Task { @MainActor in self?.currentFrame = frame } }
            runtime.inputState = { [weak input] port, device, index, id in input?.state(port: port, device: device, index: index, id: id) ?? 0 }
            try runtime.open(coreAt: core.path); try runtime.initialize(); try runtime.load(gameAt: game.fileURL, needsFullPath: core.requiresFullPath)
            isRunning = true
            task = Task.detached(priority: .userInitiated) { [runtime] in
                while !Task.isCancelled { runtime.runFrame(); try? await Task.sleep(nanoseconds: 16_666_667) }
            }
        } catch { log.append(error.localizedDescription) }
    }
    func stop() async { task?.cancel(); runtime.unloadGame(); runtime.close(); isRunning = false }
    func reset() { runtime.reset() }
    func saveState(to url: URL) throws { try runtime.serialize().write(to: url, options: .atomic) }
    func loadState(from url: URL) throws { try runtime.unserialize(Data(contentsOf: url)) }
}

final class ControllerInputState: ObservableObject {
    @Published var pressed: Set<Int> = []
    func set(_ id: Int, pressed: Bool) { if pressed { self.pressed.insert(id) } else { self.pressed.remove(id) } }
    func state(port: UInt32, device: UInt32, index: UInt32, id: UInt32) -> Int16 { pressed.contains(Int(id)) ? 1 : 0 }
}

struct FrameView: View {
    let frame: VideoFrame?
    var body: some View {
        GeometryReader { proxy in
            if let frame {
                Canvas { ctx, size in
                    let rect = CGRect(origin: .zero, size: size)
                    ctx.fill(Path(rect), with: .color(.black))
                    let text = Text("Running \(frame.width)x\(frame.height)").foregroundColor(.white)
                    ctx.draw(text, at: CGPoint(x: size.width / 2, y: size.height / 2))
                }
            } else { ProgressView("Loading core…").frame(width: proxy.size.width, height: proxy.size.height).tint(.white) }
        }
    }
}

struct TouchOverlay: View {
    @ObservedObject var input: ControllerInputState
    var body: some View {
        VStack { Spacer(); HStack(alignment: .bottom) { DPad(input: input); Spacer(); FaceButtons(input: input) }.padding(28) }
    }
}

struct DPad: View {
    @ObservedObject var input: ControllerInputState
    var body: some View {
        VStack {
            PadButton("▲", id: 4, input: input)
            HStack {
                PadButton("◀", id: 6, input: input)
                PadButton("●", id: 8, input: input)
                PadButton("▶", id: 7, input: input)
            }
            PadButton("▼", id: 5, input: input)
        }
    }
}

struct FaceButtons: View {
    @ObservedObject var input: ControllerInputState
    var body: some View {
        VStack {
            PadButton("Y", id: 9, input: input)
            HStack {
                PadButton("X", id: 10, input: input)
                PadButton("A", id: 0, input: input)
            }
            PadButton("B", id: 1, input: input)
        }
    }
}

struct PadButton: View {
    let title: String
    let id: Int
    @ObservedObject var input: ControllerInputState
    init(_ title: String, id: Int, input: ControllerInputState) {
        self.title = title; self.id = id; self.input = input
    }
    var body: some View {
        Text(title)
            .font(.headline)
            .frame(width: 58, height: 58)
            .background(.ultraThinMaterial)
            .clipShape(Circle())
            .gesture(DragGesture(minimumDistance: 0).onChanged { _ in input.set(id, pressed: true) }.onEnded { _ in input.set(id, pressed: false) })
    }
}

struct PauseMenu: View {
    @ObservedObject var session: PlayerSession
    let game: Game; let core: LibretroCore
    var body: some View {
        NavigationStack {
            List {
                Section {
                    Button("Resume", systemImage: "play.fill") {}
                    Button("Reset", systemImage: "arrow.counterclockwise") { session.reset() }
                    Toggle("Fast forward", isOn: $session.fastForward)
                }
                Section("Save States") {
                    Button("Quick Save", systemImage: "square.and.arrow.down") {}
                    Button("Quick Load", systemImage: "square.and.arrow.up") {}
                }
                Section("Video") {
                    Label("Shaders", systemImage: "camera.filters")
                    Label("Aspect ratio", systemImage: "rectangle.inset.filled")
                }
                Section("Session") {
                    Label("Netplay room", systemImage: "network")
                    Label("RetroAchievements", systemImage: "trophy")
                }
                if !session.log.isEmpty {
                    Section("Log") { ForEach(session.log, id: \.self) { Text($0) } }
                }
            }.navigationTitle("Paused")
        }
    }
}
#endif
