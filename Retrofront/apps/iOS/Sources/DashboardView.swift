import SwiftUI
import RetrofrontSwift

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            VStack(spacing: 0) {
                switch selectedTab {
                case 0:
                    LibraryView()
                case 1:
                    SettingsView()
                default:
                    EmptyView()
                }

                Spacer()

                OneUITabBar(selectedTab: $selectedTab)
            }
        }
        .fullScreenCover(isPresented: Binding(
            get: { runtime.frontendState == .gameLoaded },
            set: { _ in }
        )) {
            PlayView()
        }
        .sheet(isPresented: Binding(
            get: { !runtime.candidateCores.isEmpty },
            set: { _ in }
        )) {
            CoreSelectionSheet()
        }
    }
}

struct OneUIHeader: View {
    let title: String
    var subtitle: String? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.system(size: 34, weight: .bold))
                .foregroundColor(.white)
            if let subtitle = subtitle {
                Text(subtitle)
                    .font(.system(size: 16))
                    .foregroundColor(.gray)
            }
        }
        .padding(.horizontal, 24)
        .padding(.top, 60)
        .padding(.bottom, 20)
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

struct LibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        VStack(spacing: 0) {
            OneUIHeader(title: "Library", subtitle: "\(runtime.availableGames.count) Games Available")

            ScrollView {
                LazyVStack(spacing: 12) {
                    ForEach(runtime.availableGames) { game in
                        OneUICard {
                            Button {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            } label: {
                                HStack(spacing: 16) {
                                    Image(systemName: "gamecontroller.fill")
                                        .foregroundColor(.blue)
                                        .font(.title2)

                                    VStack(alignment: .leading) {
                                        Text(game.label)
                                            .font(.headline)
                                            .foregroundColor(.white)
                                        Text(URL(fileURLWithPath: game.path).pathExtension.uppercased())
                                            .font(.caption)
                                            .foregroundColor(.gray)
                                    }
                                    Spacer()
                                    Image(systemName: "play.circle.fill")
                                        .foregroundColor(.blue)
                                }
                                .padding(16)
                            }
                        }
                    }
                    if runtime.availableGames.isEmpty {
                        Text("No games found in 'roms' directory.")
                            .foregroundColor(.gray)
                            .padding(.top, 40)
                    }
                }
                .padding(.horizontal, 16)
            }
        }
    }
}

struct SettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var showingDetails = false
    @State private var detailTitle = ""

    var body: some View {
        VStack(spacing: 0) {
            OneUIHeader(title: "Settings")

            ScrollView {
                VStack(spacing: 24) {
                    OneUIGroup(title: "General") {
                        OneUIRow(icon: "archivebox.fill", title: "Extract Assets", color: .orange) {
                            runtime.menuAction(21) // ACTION_EXTRACT_ASSETS
                        }
                        OneUIRow(icon: "arrow.clockwise", title: "Refresh Games", color: .green) {
                            runtime.refreshGames()
                        }
                    }

                    OneUIGroup(title: "System") {
                        OneUIRow(icon: "cpu", title: "Cores", color: .blue) {
                             detailTitle = "Cores"
                             showingDetails = true
                        }
                        OneUIRow(icon: "folder.fill", title: "Directories", color: .gray) {
                             detailTitle = "Directories"
                             showingDetails = true
                        }
                    }

                    OneUIGroup(title: "Personalization") {
                        OneUIRow(icon: "paintbrush.fill", title: "Skins & Themes", color: .pink) {
                             detailTitle = "Skins"
                             showingDetails = true
                        }
                    }
                }
                .padding(.horizontal, 16)
            }
        }
        .sheet(isPresented: $showingDetails) {
            SettingsDetailView(title: detailTitle)
        }
    }
}

struct SettingsDetailView: View {
    let title: String
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ZStack {
            Color(white: 0.05).ignoresSafeArea()
            VStack(spacing: 0) {
                OneUIHeader(title: title)

                ScrollView {
                    VStack(spacing: 12) {
                        if title == "Cores" {
                            ForEach(runtime.availableCores) { core in
                                OneUICard {
                                    HStack {
                                        VStack(alignment: .leading) {
                                            Text(core.displayName).foregroundColor(.white).font(.headline)
                                            Text(core.systemName).foregroundColor(.gray).font(.subheadline)
                                        }
                                        Spacer()
                                    }.padding(16)
                                }
                            }
                        } else if title == "Skins" {
                             OneUICard {
                                 VStack(alignment: .leading) {
                                     Text("Active Theme").font(.caption2.bold()).foregroundColor(.pink)
                                     Text("OneUI Dark").foregroundColor(.white).font(.headline)
                                 }.padding(16).frame(maxWidth: .infinity, alignment: .leading)
                             }
                             OneUICard {
                                 VStack(alignment: .leading) {
                                     Text("Icon Pack").font(.caption2.bold()).foregroundColor(.pink)
                                     Text("Modern (SF Symbols)").foregroundColor(.white).font(.headline)
                                 }.padding(16).frame(maxWidth: .infinity, alignment: .leading)
                             }
                        } else {
                            ForEach(runtime.settings.filter { $0.key.contains("directory") }, id: \.key) { setting in
                                OneUICard {
                                    VStack(alignment: .leading) {
                                        Text(setting.key.replacingOccurrences(of: "_", with: " ").uppercased())
                                            .font(.caption2.bold()).foregroundColor(.blue)
                                        Text(setting.value).foregroundColor(.white).font(.system(size: 14, design: .monospaced))
                                    }.padding(16)
                                }
                            }
                        }
                    }.padding(16)
                }

                Button("Close") { dismiss() }
                    .padding()
                    .foregroundColor(.blue)
            }
        }
    }
}

struct OneUICard<Content: View>: View {
    let content: Content
    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        content
            .background(Color(white: 0.12))
            .cornerRadius(24)
    }
}

struct OneUIGroup<Content: View>: View {
    let title: String
    let content: Content
    init(title: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.uppercased())
                .font(.caption2.bold())
                .foregroundColor(.gray)
                .padding(.leading, 16)

            VStack(spacing: 0) {
                content
            }
            .background(Color(white: 0.12))
            .cornerRadius(24)
        }
    }
}

struct OneUIRow: View {
    let icon: String
    let title: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                Image(systemName: icon)
                    .frame(width: 32, height: 32)
                    .background(color.opacity(0.2))
                    .foregroundColor(color)
                    .cornerRadius(8)

                Text(title)
                    .foregroundColor(.white)

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.caption2)
                    .foregroundColor(.gray)
            }
            .padding(16)
        }
    }
}

struct OneUITabBar: View {
    @Binding var selectedTab: Int

    var body: some View {
        HStack {
            TabButton(icon: "square.grid.2x2.fill", label: "Library", index: 0, selection: $selectedTab)
            Spacer()
            TabButton(icon: "gearshape.fill", label: "Settings", index: 1, selection: $selectedTab)
        }
        .padding(.horizontal, 60)
        .padding(.vertical, 12)
        .background(Color.black)
    }
}

struct TabButton: View {
    let icon: String
    let label: String
    let index: Int
    @Binding var selection: Int

    var body: some View {
        Button {
            selection = index
        } label: {
            VStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.system(size: 20))
                Text(label)
                    .font(.caption2)
            }
            .foregroundColor(selection == index ? .blue : .gray)
        }
    }
}

struct CoreSelectionSheet: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ZStack {
            Color(white: 0.05).ignoresSafeArea()
            VStack(spacing: 0) {
                OneUIHeader(title: "Select Core", subtitle: "Multiple cores support this game")

                ScrollView {
                    VStack(spacing: 12) {
                        ForEach(runtime.candidateCores) { core in
                            OneUICard {
                                Button {
                                    runtime.selectCoreAndLaunch(core)
                                    dismiss()
                                } label: {
                                    HStack {
                                        VStack(alignment: .leading) {
                                            Text(core.displayName)
                                                .font(.headline)
                                                .foregroundColor(.white)
                                            Text(core.systemName)
                                                .font(.subheadline)
                                                .foregroundColor(.gray)
                                        }
                                        Spacer()
                                        Image(systemName: "cpu.fill")
                                            .foregroundColor(.blue)
                                    }
                                    .padding(16)
                                }
                            }
                        }
                    }
                    .padding(16)
                }
            }
        }
    }
}

struct PlayView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss
    @State private var showQuickMenu = false

    var body: some View {
        GeometryReader { proxy in
            ZStack {
                Color.black.ignoresSafeArea()

                VStack {
                    ZStack {
                        if let image = runtime.displayImage {
                            Image(uiImage: image)
                                .resizable()
                                .interpolation(.none)
                                .scaledToFit()
                        } else {
                            ProgressView()
                                .tint(.white)
                        }
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)

                    if proxy.size.height > proxy.size.width {
                        VirtualController()
                            .padding(.bottom, 20)
                    }
                }

                VStack {
                    HStack {
                        Button {
                            showQuickMenu = true
                        } label: {
                            Image(systemName: "line.3.horizontal")
                                .font(.title)
                                .padding()
                                .background(Circle().fill(.black.opacity(0.5)))
                                .foregroundColor(.white)
                        }
                        Spacer()
                    }
                    Spacer()
                }
                .padding()
            }
        }
        .onAppear { runtime.play() }
        .onDisappear { runtime.stop() }
        .sheet(isPresented: $showQuickMenu) {
            QuickMenuView(isPresented: $showQuickMenu)
        }
    }
}

struct QuickMenuView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPresented: Bool
    @State private var showingOptions = false

    var body: some View {
        ZStack {
            Color(white: 0.05).ignoresSafeArea()
            VStack(spacing: 0) {
                OneUIHeader(title: "Quick Menu")

                ScrollView {
                    VStack(spacing: 20) {
                        OneUIGroup(title: "Game Control") {
                            OneUIRow(icon: "play.fill", title: "Resume", color: .blue) {
                                isPresented = false
                            }
                            OneUIRow(icon: "arrow.counterclockwise", title: "Restart", color: .orange) {
                                runtime.menuAction(9) // ACTION_RESTART_CONTENT
                                isPresented = false
                            }
                            OneUIRow(icon: "xmark.circle.fill", title: "Close Content", color: .red) {
                                runtime.stop()
                                runtime.unloadGame()
                                isPresented = false
                            }
                        }

                        OneUIGroup(title: "Settings") {
                            OneUIRow(icon: "slider.horizontal.3", title: "Core Options", color: .purple) {
                                showingOptions = true
                            }
                        }
                    }
                    .padding(16)
                }
            }
        }
        .sheet(isPresented: $showingOptions) {
            CoreOptionsView()
        }
    }
}

struct CoreOptionsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ZStack {
            Color(white: 0.05).ignoresSafeArea()
            VStack(spacing: 0) {
                OneUIHeader(title: "Core Options")
                ScrollView {
                    VStack(spacing: 12) {
                        ForEach(runtime.coreOptions, id: \.key) { opt in
                            OneUICard {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(opt.desc).foregroundColor(.white).font(.headline)
                                    Text(opt.value).foregroundColor(.blue).font(.subheadline)
                                }.padding(16).frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }
                    }.padding(16)
                }
                Button("Done") { dismiss() }.padding()
            }
        }
    }
}

struct VirtualController: View {
    var body: some View {
        VStack {
            Spacer()
            HStack {
                Dpad_OneUI()
                Spacer()
                ActionButtons_OneUI()
            }
            .padding(.horizontal, 30)
            .padding(.bottom, 50)
        }
    }
}

struct Dpad_OneUI: View {
    var body: some View {
        VStack(spacing: 8) {
            DPadButton(icon: "chevron.up", button: .up)
            HStack(spacing: 8) {
                DPadButton(icon: "chevron.left", button: .left)
                Circle().frame(width: 48, height: 48).foregroundColor(.white.opacity(0.05))
                DPadButton(icon: "chevron.right", button: .right)
            }
            DPadButton(icon: "chevron.down", button: .down)
        }
    }
}

struct ActionButtons_OneUI: View {
    var body: some View {
        VStack(spacing: 12) {
            HStack(spacing: 12) {
                ActionButton(label: "Y", color: .green, button: .y)
                ActionButton(label: "X", color: .blue, button: .x)
            }
            HStack(spacing: 12) {
                ActionButton(label: "B", color: .red, button: .b)
                ActionButton(label: "A", color: .yellow, button: .a)
            }
        }
    }
}

struct DPadButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let icon: String
    let button: JoypadButton

    var body: some View {
        Image(systemName: icon)
            .font(.title2.bold())
            .frame(width: 56, height: 56)
            .background(Circle().fill(Color.white.opacity(0.12)))
            .foregroundColor(.white)
            .simultaneousGesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                    .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
            )
    }
}

struct ActionButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let label: String
    let color: Color
    let button: JoypadButton

    var body: some View {
        Text(label)
            .font(.title.bold())
            .frame(width: 64, height: 64)
            .background(Circle().fill(color.opacity(0.2)))
            .overlay(Circle().stroke(color, lineWidth: 3))
            .foregroundColor(color)
            .simultaneousGesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                    .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
            )
    }
}
