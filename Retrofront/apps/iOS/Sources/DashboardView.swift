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
        ZStack {
            LinearGradient(
                colors: [Color(red: 0.02, green: 0.02, blue: 0.04), Color(red: 0.04, green: 0.10, blue: 0.14)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .ignoresSafeArea()

            Circle()
                .fill(.cyan.opacity(0.18))
                .blur(radius: 70)
                .frame(width: 260, height: 260)
                .offset(x: -130, y: -220)

            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 6) {
                    Text(runtime.currentMenu?.title.uppercased() ?? "RETROFRONT")
                        .font(.system(size: 34, weight: .black, design: .rounded))
                        .foregroundStyle(.white)
                    Text("Rust libretro menu engine • XMB/Ozone compatible model")
                        .font(.caption)
                        .foregroundStyle(.white.opacity(0.62))
                }
                .padding(.horizontal, 24)
                .padding(.top, 28)

                ScrollView {
                    LazyVStack(spacing: 12) {
                        if let menu = runtime.currentMenu {
                            ForEach(menu.entries, id: \.actionId) { entry in
                                RustMenuRow(entry: entry) {
                                    runtime.menuAction(entry.actionId)
                                }
                            }
                        } else {
                            Text("Menu unavailable")
                                .foregroundStyle(.white.opacity(0.7))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(24)
                        }
                    }
                    .padding(.horizontal, 18)
                    .padding(.bottom, 28)
                }

                Text("MoltenVK / OpenGL ES: bgfx requested, software copy fallback active until host handles are ready")
                    .font(.caption2)
                    .foregroundStyle(.cyan.opacity(0.75))
                    .padding(.horizontal, 24)
                    .padding(.bottom, 12)
            }
        }
    }
}

struct RustMenuRow: View {
    let entry: MenuEntry
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                MenuEntryIcon(kind: entry.kind)

                VStack(alignment: .leading, spacing: 4) {
                    Text(entry.label)
                        .font(.headline)
                        .foregroundStyle(.white)
                    if !entry.sublabel.isEmpty {
                        Text(entry.sublabel)
                            .font(.caption)
                            .foregroundStyle(.white.opacity(0.58))
                            .lineLimit(2)
                    }
                }

                Spacer(minLength: 12)

                if !entry.value.isEmpty {
                    Text(entry.value)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.cyan)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .frame(maxWidth: 110, alignment: .trailing)
                }

                if entry.kind == .submenu {
                    Image(systemName: "chevron.right")
                        .font(.caption.weight(.bold))
                        .foregroundStyle(.white.opacity(0.5))
                }
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .fill(.white.opacity(0.08))
                    .overlay(
                        RoundedRectangle(cornerRadius: 18, style: .continuous)
                            .stroke(.white.opacity(0.10), lineWidth: 1)
                    )
            )
        }
        .buttonStyle(.plain)
        .disabled(entry.actionId == 0)
    }
}

struct MenuEntryIcon: View {
    let kind: MenuEntryKind

    var body: some View {
        Image(systemName: iconName)
            .font(.system(size: 18, weight: .bold))
            .foregroundStyle(.black)
            .frame(width: 42, height: 42)
            .background(Circle().fill(iconColor))
    }

    private var iconName: String {
        switch kind {
        case .action:
            return "play.fill"
        case .submenu:
            return "rectangle.grid.2x2.fill"
        case .toggle:
            return "switch.2"
        case .setting:
            return "slider.horizontal.3"
        }
    }

    private var iconColor: Color {
        switch kind {
        case .action:
            return .green
        case .submenu:
            return .cyan
        case .toggle:
            return .orange
        case .setting:
            return .purple
        }
    }
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
