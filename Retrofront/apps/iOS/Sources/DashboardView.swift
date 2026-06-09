import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0
    @State private var isPlayViewActive = false
    @State private var isFilePickerPresented = false

    var body: some View {
        ZStack {
            Color(white: 0.05).ignoresSafeArea()

            TabView(selection: $selectedTab) {
                ModernHomeView(isPlayViewActive: $isPlayViewActive, isFilePickerPresented: $isFilePickerPresented)
                    .tabItem {
                        Label("Home", systemImage: "house.fill")
                    }.tag(0)

                ModernLibraryView(isFilePickerPresented: $isFilePickerPresented)
                    .tabItem {
                        Label("Library", systemImage: "gamecontroller.fill")
                    }.tag(1)

                ModernSettingsView()
                    .tabItem {
                        Label("Settings", systemImage: "gearshape.2.fill")
                    }.tag(2)
            }
            .tint(.cyan)
        }
        .fullScreenCover(isPresented: $isPlayViewActive) {
            PlayView()
        }
        .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
            if case .success(let url) = result {
                runtime.importFile(at: url)
            }
        }
        .onReceive(runtime.$frontendState) { newState in
            if newState == .gameLoaded {
                isPlayViewActive = true
            }
        }
    }
}

struct ModernHomeView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool
    @Binding var isFilePickerPresented: Bool

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    Text("Continue Playing")
                        .font(.title2.bold())
                        .padding(.horizontal)

                    if let game = runtime.loadedGameURL {
                        Button {
                            isPlayViewActive = true
                        } label: {
                            HStack {
                                Image(systemName: "play.circle.fill")
                                    .font(.largeTitle)
                                VStack(alignment: .leading) {
                                    Text(game.lastPathComponent)
                                        .font(.headline)
                                    Text(runtime.systemInfo?.libraryName ?? "Unknown Core")
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                            }
                            .padding()
                            .background(Color(white: 0.15))
                            .cornerRadius(12)
                        }
                        .padding(.horizontal)
                        .buttonStyle(.plain)
                    } else {
                        Text("No game loaded")
                            .foregroundStyle(.secondary)
                            .padding()
                            .frame(maxWidth: .infinity)
                            .background(Color(white: 0.1))
                            .cornerRadius(12)
                            .padding(.horizontal)
                    }

                    Text("Quick Actions")
                        .font(.title2.bold())
                        .padding(.horizontal)
                        .padding(.top)

                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 15) {
                        NavigationLink {
                            CoreListView()
                        } label: {
                            ActionCard(title: "Load Core", icon: "cpu", color: .purple) {}
                        }

                        ActionCard(title: "Import ROM", icon: "plus.circle", color: .green) {
                            isFilePickerPresented = true
                        }
                    }
                    .padding(.horizontal)

                    if !runtime.availableGames.isEmpty {
                        Text("Recently Added")
                            .font(.title2.bold())
                            .padding(.horizontal)
                            .padding(.top)

                        ForEach(runtime.availableGames.prefix(5), id: \.path) { game in
                            Button {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            } label: {
                                HStack {
                                    Image(systemName: "doc.fill")
                                    Text(game.label)
                                    Spacer()
                                    Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
                                }
                                .padding()
                                .background(Color(white: 0.15))
                                .cornerRadius(10)
                            }
                            .padding(.horizontal)
                            .buttonStyle(.plain)
                        }
                    }
                }
                .padding(.vertical)
            }
            .navigationTitle("Retrofront")
            .background(Color(white: 0.05))
        }
    }
}

struct CoreListView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        List {
            ForEach(runtime.availableCores, id: \.path) { core in
                Button {
                    runtime.loadCore(core)
                    dismiss()
                } label: {
                    VStack(alignment: .leading) {
                        Text(core.displayName)
                            .font(.headline)
                        Text(core.systemName)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle("Select Core")
    }
}

struct ActionCard: View {
    let title: String
    let icon: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.system(size: 30))
                    .foregroundStyle(color)
                Text(title)
                    .font(.headline)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 20)
            .background(
                LinearGradient(colors: [color.opacity(0.22), Color.white.opacity(0.06)], startPoint: .topLeading, endPoint: .bottomTrailing)
            )
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(color.opacity(0.45), lineWidth: 1)
            )
            .shadow(color: color.opacity(0.18), radius: 16, x: 0, y: 8)
        }
        .buttonStyle(.plain)
    }
}

struct ModernLibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isFilePickerPresented: Bool

    var body: some View {
        NavigationStack {
            List {
                Section("My Games") {
                    if runtime.availableGames.isEmpty {
                        Text("No games found in Roms folder").foregroundStyle(.secondary)
                    } else {
                        ForEach(runtime.availableGames, id: \.path) { game in
                            Button {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            } label: {
                                Text(game.label)
                            }
                        }
                    }
                }

                Section("Cores") {
                    ForEach(runtime.availableCores, id: \.path) { core in
                        Button {
                            runtime.loadCore(core)
                        } label: {
                            VStack(alignment: .leading) {
                                Text(core.displayName)
                                    .font(.headline)
                                Text(core.systemName)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }

                Section("Import") {
                    Button {
                        isFilePickerPresented = true
                    } label: {
                        Label("Add Content or Core", systemImage: "square.and.arrow.down")
                    }
                    Text(".dylib files are copied into the configured Cores directory; other files go to the content directory.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("Library")
            .background(Color(white: 0.05))
            .scrollContentBackground(.hidden)
        }
    }
}

struct ModernSettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        NavigationStack {
            RustMenuView()
                .navigationTitle("")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .navigationBarLeading) {
                        Button {
                            runtime.menuPop()
                        } label: {
                            Label("Back", systemImage: "chevron.left")
                        }
                    }
                }
        }
    }
}

struct RustMenuView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        GeometryReader { geometry in
            if let scene = runtime.currentMenuRenderScene(width: geometry.size.width, height: geometry.size.height) {
                RustMenuSceneView(scene: scene, action: runtime.menuAction)
            } else {
                Text("Menu unavailable")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
    }
}

private struct RustMenuSceneView: View {
    let scene: MenuRenderScene
    let action: (UInt32) -> Void

    var body: some View {
        ZStack(alignment: .topLeading) {
            argb(scene.backgroundColor).ignoresSafeArea()
            ForEach(Array(scene.nodes.enumerated()), id: \.offset) { _, node in
                RustMenuRenderNodeView(node: node, action: action)
            }
        }
    }
}

private struct RustMenuRenderNodeView: View {
    let node: MenuRenderNode
    let action: (UInt32) -> Void

    var body: some View {
        switch node.kind {
        case .panel:
            Rectangle()
                .fill(argb(node.backgroundColor))
                .frame(width: CGFloat(node.width), height: CGFloat(node.height))
                .position(x: CGFloat(node.x + node.width / 2), y: CGFloat(node.y + node.height / 2))
        case .separator:
            Rectangle()
                .fill(argb(node.backgroundColor))
                .frame(width: CGFloat(node.width), height: max(CGFloat(node.height), 1))
                .position(x: CGFloat(node.x + node.width / 2), y: CGFloat(node.y))
        case .text:
            Text(node.text)
                .font(font)
                .foregroundStyle(argb(node.foregroundColor))
                .position(x: CGFloat(node.x), y: CGFloat(node.y))
        case .entry:
            Button { if node.actionId != 0 { action(node.actionId) } } label: {
                Text(node.text)
                    .font(font)
                    .foregroundStyle(argb(node.foregroundColor))
                    .frame(width: CGFloat(node.width), height: CGFloat(node.height), alignment: .leading)
                    .padding(.horizontal, 8)
                    .background(entryBackground)
            }
            .buttonStyle(.plain)
            .disabled(node.actionId == 0)
            .position(x: CGFloat(node.x + node.width / 2), y: CGFloat(node.y + node.height / 2))
        case .icon:
            Button { if node.actionId != 0 { action(node.actionId) } } label: {
                Image(systemName: "square.grid.3x3.fill")
                    .font(.system(size: CGFloat(max(node.fontSize, 12)), weight: .bold))
                    .foregroundStyle(argb(node.foregroundColor))
                    .frame(width: CGFloat(node.width), height: CGFloat(node.height))
                    .background(Circle().fill(argb(node.backgroundColor)))
            }
            .buttonStyle(.plain)
            .disabled(node.actionId == 0)
            .position(x: CGFloat(node.x + node.width / 2), y: CGFloat(node.y + node.height / 2))
        }
    }

    private var font: Font {
        if node.flags & 4 != 0 {
            return .system(size: CGFloat(max(node.fontSize, 10)), weight: .regular, design: .monospaced)
        }
        return .system(size: CGFloat(max(node.fontSize, 10)), weight: node.flags & 1 != 0 ? .bold : .regular)
    }

    @ViewBuilder
    private var entryBackground: some View {
        if node.backgroundColor == 0 {
            Color.clear
        } else if node.flags & 2 != 0 {
            RoundedRectangle(cornerRadius: 3).fill(argb(node.backgroundColor)).shadow(color: .black.opacity(0.12), radius: 2, y: 1)
        } else {
            Rectangle().fill(argb(node.backgroundColor))
        }
    }
}

private func argb(_ value: UInt32) -> Color {
    let a = Double((value >> 24) & 0xff) / 255.0
    let r = Double((value >> 16) & 0xff) / 255.0
    let g = Double((value >> 8) & 0xff) / 255.0
    let b = Double(value & 0xff) / 255.0
    return Color(red: r, green: g, blue: b).opacity(a)
}

struct PlayView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack {
            ZStack {
                Color.black
                if let image = runtime.displayImage {
                    Image(uiImage: image)
                        .resizable()
                        .interpolation(.none)
                        .scaledToFit()
                } else {
                    VStack {
                        Image(systemName: "gamecontroller").font(.system(size: 50))
                        Text("No Video").font(.headline)
                    }.foregroundStyle(.white)
                }
            }
            .aspectRatio(runtime.aspectRatio, contentMode: .fit)
            .cornerRadius(12)
            .padding()

            Spacer()

            VirtualController()

            HStack(spacing: 40) {
                Button {
                    runtime.toggleRunning()
                } label: {
                    Image(systemName: runtime.isRunning ? "pause.circle.fill" : "play.circle.fill")
                        .font(.system(size: 60))
                }

                Button {
                    runtime.stop()
                    dismiss()
                } label: {
                    Image(systemName: "stop.circle.fill")
                        .font(.system(size: 60))
                        .foregroundStyle(.red)
                }
            }
            .padding(.bottom, 30)
        }
        .navigationTitle(runtime.loadedGameURL?.lastPathComponent ?? "Play")
        .background(Color.black)
        .onAppear {
            runtime.play()
        }
        .onDisappear {
            runtime.stop()
        }
    }
}

struct VirtualController: View {
    var body: some View {
        VStack {
            HStack {
                Dpad()
                Spacer()
                ActionButtons()
            }
            .padding(40)
        }
    }
}

struct Dpad: View {
    var body: some View {
        VStack(spacing: 5) {
            DPadButton(icon: "chevron.up", button: .up)
            HStack(spacing: 5) {
                DPadButton(icon: "chevron.left", button: .left)
                Circle().frame(width: 40, height: 40).opacity(0.1)
                DPadButton(icon: "chevron.right", button: .right)
            }
            DPadButton(icon: "chevron.down", button: .down)
        }
    }
}

struct DPadButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let icon: String
    let button: JoypadButton

    var body: some View {
        Button {
            // Visual feedback
        } label: {
            Image(systemName: icon)
                .frame(width: 44, height: 44)
                .background(Circle().fill(.white.opacity(0.1)))
        }
        .simultaneousGesture(
            DragGesture(minimumDistance: 0)
                .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
        )
    }
}

struct ActionButtons: View {
    var body: some View {
        VStack(spacing: 10) {
            HStack(spacing: 10) {
                ActionButton(label: "Y", color: .green, button: .y)
                ActionButton(label: "X", color: .blue, button: .x)
            }
            HStack(spacing: 10) {
                ActionButton(label: "B", color: .red, button: .b)
                ActionButton(label: "A", color: .yellow, button: .a)
            }
        }
    }
}

struct ActionButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let label: String
    let color: Color
    let button: JoypadButton

    var body: some View {
        Button {
            // Visual feedback
        } label: {
            Text(label)
                .font(.headline)
                .frame(width: 50, height: 50)
                .background(Circle().fill(color.opacity(0.3)))
                .overlay(Circle().stroke(color, lineWidth: 2))
        }
        .simultaneousGesture(
            DragGesture(minimumDistance: 0)
                .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
        )
    }
}
