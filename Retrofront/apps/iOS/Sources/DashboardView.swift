import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0
    @State private var isPlayViewActive = false
    @State private var isFilePickerPresented = false

    var body: some View {
        TabView(selection: $selectedTab) {
            OneHomeView(isPlayViewActive: $isPlayViewActive, isFilePickerPresented: $isFilePickerPresented)
                .tabItem { Label("Home", systemImage: "house.fill") }
                .tag(0)
            OneLibraryView(isFilePickerPresented: $isFilePickerPresented)
                .tabItem { Label("Library", systemImage: "rectangle.stack.fill") }
                .tag(1)
            OneSettingsView()
                .tabItem { Label("Settings", systemImage: "gearshape.fill") }
                .tag(2)
        }
        .tint(.cyan)
        .background(OneUI.background.ignoresSafeArea())
        .fullScreenCover(isPresented: $isPlayViewActive) { PlayView() }
        .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
            if case .success(let url) = result { runtime.importFile(at: url) }
        }
        .sheet(isPresented: Binding(get: { runtime.pendingContentURL != nil }, set: { if !$0 { runtime.cancelCoreChoice() } })) {
            CoreChoiceSheet()
                .presentationDetents([.medium, .large])
                .presentationDragIndicator(.visible)
        }
        .onReceive(runtime.$frontendState) { if $0 == .gameLoaded { isPlayViewActive = true } }
    }
}

enum OneUI {
    static let background = Color(red: 0.965, green: 0.973, blue: 0.988)
    static let card = Color.white
    static let ink = Color(red: 0.055, green: 0.075, blue: 0.11)
    static let subink = Color(red: 0.40, green: 0.44, blue: 0.50)
    static let blue = Color(red: 0.08, green: 0.44, blue: 1.0)
    static let cyan = Color(red: 0.0, green: 0.72, blue: 0.92)
    static let radius: CGFloat = 28
}

struct OneScreen<Content: View>: View {
    let title: String
    @ViewBuilder var content: Content
    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    Text(title)
                        .font(.system(size: 42, weight: .black, design: .rounded))
                        .foregroundStyle(OneUI.ink)
                        .padding(.top, 18)
                    content
                }
                .padding(.horizontal, 22)
                .padding(.bottom, 30)
            }
            .background(OneUI.background)
        }
    }
}

struct OneHomeView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool
    @Binding var isFilePickerPresented: Bool

    var body: some View {
        OneScreen(title: "Retrofront") {
            OneHeroCard(runtime: runtime, isPlayViewActive: $isPlayViewActive)
            HStack(spacing: 12) {
                OneActionButton(title: "Import Content", subtitle: "Copies to RetroArch downloads", icon: "square.and.arrow.down.fill", tint: OneUI.blue) { isFilePickerPresented = true }
                OneActionButton(title: "Install Assets", subtitle: "Extract bundled assets.zip", icon: "archivebox.fill", tint: .purple) { runtime.installBundledAssets() }
            }
            OneStatusCard(message: runtime.statusMessage)
            if !runtime.availableGames.isEmpty {
                OneSectionHeader(title: "Recent content", subtitle: "Detected using core .info extensions")
                ForEach(runtime.availableGames.prefix(6), id: \.path) { game in
                    OneContentRow(title: game.label, subtitle: game.path, icon: "play.rectangle.fill") {
                        runtime.loadGame(at: URL(fileURLWithPath: game.path))
                    }
                }
            }
        }
    }
}

struct OneHeroCard: View {
    let runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool
    var body: some View {
        Button { if runtime.loadedGameURL != nil { isPlayViewActive = true } } label: {
            VStack(alignment: .leading, spacing: 20) {
                HStack {
                    Image(systemName: "gamecontroller.fill")
                        .font(.system(size: 32, weight: .bold))
                        .foregroundStyle(.white)
                        .frame(width: 62, height: 62)
                        .background(Circle().fill(.white.opacity(0.18)))
                    Spacer()
                    Text(runtime.frontendState == .gameLoaded ? "READY" : "ONE UI")
                        .font(.caption.weight(.black))
                        .padding(.horizontal, 12)
                        .padding(.vertical, 7)
                        .background(Capsule().fill(.white.opacity(0.2)))
                }
                VStack(alignment: .leading, spacing: 6) {
                    Text(runtime.loadedGameURL?.lastPathComponent ?? "Pick content and Retrofront selects the core")
                        .font(.system(size: 26, weight: .black, design: .rounded))
                        .foregroundStyle(.white)
                        .lineLimit(2)
                    Text(runtime.systemInfo?.libraryName ?? "Build-time dylibs are discovered from Frameworks/dylibs and matched with .info metadata.")
                        .font(.subheadline.weight(.medium))
                        .foregroundStyle(.white.opacity(0.78))
                }
            }
            .padding(24)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(LinearGradient(colors: [OneUI.blue, OneUI.cyan], startPoint: .topLeading, endPoint: .bottomTrailing))
            .clipShape(RoundedRectangle(cornerRadius: 34, style: .continuous))
            .shadow(color: OneUI.blue.opacity(0.28), radius: 24, y: 14)
        }
        .buttonStyle(.plain)
    }
}

struct OneLibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isFilePickerPresented: Bool
    var body: some View {
        OneScreen(title: "Library") {
            OneSectionHeader(title: "Content", subtitle: "No hard-coded extensions; scanner uses loaded .info data")
            OneActionButton(title: "Import Content", subtitle: "RetroArch iOS-style import to downloads", icon: "plus.app.fill", tint: .green) { isFilePickerPresented = true }
            if runtime.availableGames.isEmpty {
                OneEmptyCard(text: "No content found yet")
            } else {
                ForEach(runtime.availableGames, id: \.path) { game in
                    OneContentRow(title: game.label, subtitle: game.path, icon: "doc.fill") { runtime.loadGame(at: URL(fileURLWithPath: game.path)) }
                }
            }
            OneSectionHeader(title: "Bundled cores", subtitle: "App-store safe: add cores only at build time in dylibs/")
            ForEach(runtime.availableCores, id: \.path) { core in
                OneCoreRow(core: core)
            }
        }
    }
}

struct OneSettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    var body: some View {
        OneScreen(title: "Settings") {
            OneActionButton(title: "Install / Refresh Assets", subtitle: "Extract assets.zip into the RetroArch assets directory", icon: "paintpalette.fill", tint: .purple) { runtime.installBundledAssets() }
            OneSectionHeader(title: "RetroArch menu", subtitle: "Rust settings, font/assets paths, downloads and directories")
            RustMenuView()
                .frame(minHeight: 460)
                .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
        }
    }
}

struct CoreChoiceSheet: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    var body: some View {
        NavigationStack {
            List(runtime.pendingCoreChoices, id: \.path) { core in
                Button { runtime.launchPendingContent(with: core) } label: { OneCoreRow(core: core) }
            }
            .navigationTitle("Select Core")
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Cancel") { runtime.cancelCoreChoice() } } }
        }
    }
}

struct OneActionButton: View {
    let title: String
    let subtitle: String
    let icon: String
    let tint: Color
    let action: () -> Void
    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 12) {
                Image(systemName: icon).font(.title2.bold()).foregroundStyle(tint)
                    .frame(width: 46, height: 46).background(Circle().fill(tint.opacity(0.12)))
                Text(title).font(.headline).foregroundStyle(OneUI.ink)
                Text(subtitle).font(.caption).foregroundStyle(OneUI.subink).lineLimit(2)
            }
            .padding(18).frame(maxWidth: .infinity, alignment: .leading)
            .background(OneUI.card)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
            .shadow(color: .black.opacity(0.06), radius: 18, y: 8)
        }.buttonStyle(.plain)
    }
}

struct OneContentRow: View {
    let title: String; let subtitle: String; let icon: String; let action: () -> Void
    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                Image(systemName: icon).foregroundStyle(.white).frame(width: 44, height: 44).background(Circle().fill(OneUI.blue))
                VStack(alignment: .leading, spacing: 3) { Text(title).font(.headline).foregroundStyle(OneUI.ink); Text(subtitle).font(.caption).foregroundStyle(OneUI.subink).lineLimit(1).truncationMode(.middle) }
                Spacer(); Image(systemName: "chevron.right").foregroundStyle(OneUI.subink)
            }.padding(16).background(OneUI.card).clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous))
        }.buttonStyle(.plain)
    }
}

struct OneCoreRow: View {
    let core: CoreInfo
    var body: some View {
        HStack(spacing: 14) {
            Image(systemName: "cpu.fill").foregroundStyle(.white).frame(width: 44, height: 44).background(Circle().fill(.purple))
            VStack(alignment: .leading, spacing: 3) {
                Text(core.displayName).font(.headline).foregroundStyle(OneUI.ink)
                Text([core.systemName, core.supportedExtensions.joined(separator: ", ")].filter { !$0.isEmpty }.joined(separator: " • "))
                    .font(.caption).foregroundStyle(OneUI.subink).lineLimit(2)
            }
        }.padding(16).background(OneUI.card).clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous))
    }
}

struct OneSectionHeader: View { let title: String; let subtitle: String; var body: some View { VStack(alignment: .leading, spacing: 3) { Text(title).font(.title3.bold()).foregroundStyle(OneUI.ink); Text(subtitle).font(.caption).foregroundStyle(OneUI.subink) }.padding(.top, 8) } }
struct OneEmptyCard: View { let text: String; var body: some View { Text(text).font(.headline).foregroundStyle(OneUI.subink).frame(maxWidth: .infinity).padding(28).background(OneUI.card).clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous)) } }
struct OneStatusCard: View { let message: String; var body: some View { Label(message, systemImage: "info.circle.fill").font(.subheadline.weight(.medium)).foregroundStyle(OneUI.subink).padding(16).frame(maxWidth: .infinity, alignment: .leading).background(OneUI.card).clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous)) } }

struct RustMenuView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    var body: some View {
        VStack(spacing: 10) {
            if let menu = runtime.currentMenu {
                ForEach(menu.entries, id: \.actionId) { entry in RustMenuRow(entry: entry) { runtime.menuAction(entry.actionId) } }
            } else { OneEmptyCard(text: "Menu unavailable") }
        }.padding(12).background(OneUI.card)
    }
}

struct RustMenuRow: View {
    let entry: MenuEntry; let action: () -> Void
    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: entry.kind == .submenu ? "folder.fill" : "slider.horizontal.3").foregroundStyle(OneUI.blue).frame(width: 36, height: 36).background(Circle().fill(OneUI.blue.opacity(0.12)))
                VStack(alignment: .leading) { Text(entry.label).font(.subheadline.bold()).foregroundStyle(OneUI.ink); if !entry.sublabel.isEmpty { Text(entry.sublabel).font(.caption2).foregroundStyle(OneUI.subink).lineLimit(2) } }
                Spacer(); if !entry.value.isEmpty { Text(entry.value).font(.caption2.bold()).foregroundStyle(OneUI.blue).lineLimit(1) }
            }.padding(10).background(OneUI.background).clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
        }.buttonStyle(.plain).disabled(entry.actionId == 0)
    }
}
struct PlayView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack {
            GeometryReader { proxy in
                ZStack(alignment: .topLeading) {
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
                    if let overlay = runtime.overlayInfo, overlay.enabled {
                        Text(overlay.activeName.isEmpty ? "Overlay" : overlay.activeName)
                            .font(.caption2.weight(.bold))
                            .foregroundStyle(.white.opacity(0.75))
                            .padding(8)
                    }
                }
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: 0)
                        .onChanged { value in runtime.setOverlayTouch(slot: 0, location: value.location, in: proxy.size, active: true) }
                        .onEnded { _ in runtime.setOverlayTouch(slot: 0, location: .zero, in: proxy.size, active: false) }
                )
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
            runtime.clearOverlayTouches()
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
