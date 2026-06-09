import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers
import UIKit

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0
    @State private var isPlayViewActive = false
    @State private var isFilePickerPresented = false

    var body: some View {
        TabView(selection: $selectedTab) {
            LibraryView(isFilePickerPresented: $isFilePickerPresented)
                .tabItem { Label("Library", systemImage: "rectangle.stack.fill") }
                .tag(0)

            NowPlayingView(isPlayViewActive: $isPlayViewActive)
                .tabItem { Label("Play", systemImage: "play.fill") }
                .tag(1)

            SettingsView()
                .tabItem { Label("Settings", systemImage: "gearshape.fill") }
                .tag(2)
        }
        .accentColor(OneUI.accent)
        .preferredColorScheme(.dark)
        .toolbarBackground(OneUI.surface, for: .tabBar)
        .toolbarBackground(.visible, for: .tabBar)
        .background(OneUI.background.ignoresSafeArea())
        .fullScreenCover(isPresented: $isPlayViewActive) { PlayView() }
        .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
            if case .success(let url) = result { runtime.importFile(at: url) }
        }
        .sheet(isPresented: Binding(get: { runtime.pendingContentURL != nil }, set: { if !$0 { runtime.cancelCoreChoice() } })) {
            CoreChoiceSheet()
        }
        .onReceive(runtime.$launchToken) { token in
            if token > 0 { isPlayViewActive = true }
        }
    }
}

enum OneUI {
    static let background = Color(red: 0.030, green: 0.036, blue: 0.055)
    static let surface = Color(red: 0.075, green: 0.088, blue: 0.125)
    static let elevated = Color(red: 0.105, green: 0.122, blue: 0.170)
    static let ink = Color(red: 0.930, green: 0.950, blue: 0.990)
    static let secondary = Color(red: 0.650, green: 0.700, blue: 0.790)
    static let muted = Color(red: 0.430, green: 0.490, blue: 0.610)
    static let accent = Color(red: 0.250, green: 0.600, blue: 1.000)
    static let teal = Color(red: 0.130, green: 0.830, blue: 0.830)
    static let violet = Color(red: 0.620, green: 0.500, blue: 1.000)
    static let radius: CGFloat = 24
    static let compactRadius: CGFloat = 18
}

struct AppScreen<Content: View>: View {
    let title: String
    let subtitle: String
    @ViewBuilder var content: Content

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(title)
                            .font(.system(size: 34, weight: .bold, design: .default))
                            .foregroundColor(OneUI.ink)
                        Text(subtitle)
                            .font(.subheadline.weight(.medium))
                            .foregroundColor(OneUI.secondary)
                    }
                    .padding(.top, 16)

                    content
                }
                .padding(.horizontal, 18)
                .padding(.bottom, 28)
            }
            .background(OneUI.background)
            .navigationBarHidden(true)
        }
        .navigationViewStyle(.stack)
    }
}

struct LibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isFilePickerPresented: Bool

    var body: some View {
        AppScreen(title: "Library", subtitle: "ROM専用ライブラリ（coresフォルダは表示しません）") {
            HStack(spacing: 12) {
                PrimaryAction(title: "Import", subtitle: "ROMを追加", icon: "plus", tint: OneUI.accent) {
                    isFilePickerPresented = true
                }
                PrimaryAction(title: "Refresh", subtitle: "再スキャン", icon: "arrow.clockwise", tint: OneUI.teal) {
                    runtime.rescanLibrary()
                }
            }

            StatusPill(message: runtime.statusMessage)

            HStack(spacing: 10) {
                LibraryStatCard(title: "ROMs", value: "\(runtime.availableGames.count)", icon: "gamecontroller.fill", tint: OneUI.accent)
                LibraryStatCard(title: "Types", value: "\(runtime.libraryRomTypeCount)", icon: "doc.fill", tint: OneUI.violet)
            }

            ContentSection(title: "ROMs", count: runtime.availableGames.count) {
                if runtime.availableGames.isEmpty {
                    EmptyPanel(icon: "tray", title: "No ROMs", message: "Importから.gbaなどのROMだけを追加してください。coresやinfoはライブラリに表示されません。")
                } else {
                    VStack(spacing: 10) {
                        ForEach(runtime.availableGames, id: \.path) { game in
                            GameRow(
                                game: game,
                                details: runtime.romDetails(for: game),
                                compatibility: runtime.compatibleCoreSummary(for: game)
                            ) {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            }
                        }
                    }
                }
            }
        }
    }
}

struct NowPlayingView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool

    var body: some View {
        AppScreen(title: "Play", subtitle: "現在のゲームセッション") {
            VStack(alignment: .leading, spacing: 18) {
                ZStack {
                    RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous)
                        .fill(Color.black)
                        .aspectRatio(16.0 / 10.0, contentMode: .fit)
                    if let image = runtime.displayImage {
                        Image(uiImage: image)
                            .resizable()
                            .interpolation(.none)
                            .scaledToFit()
                            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
                    } else {
                        VStack(spacing: 10) {
                            Image(systemName: "display")
                                .font(.system(size: 34, weight: .semibold))
                            Text(runtime.loadedGameURL == nil ? "No game loaded" : "Ready to render")
                                .font(.headline)
                        }
                        .foregroundColor(.white.opacity(0.78))
                    }
                }

                VStack(alignment: .leading, spacing: 6) {
                    Text(runtime.loadedGameURL?.lastPathComponent ?? "ゲーム未選択")
                        .font(.title3.bold())
                        .foregroundColor(OneUI.ink)
                    Text(runtime.systemInfo?.libraryName ?? "Libraryからゲームを選択すると起動します。")
                        .font(.subheadline)
                        .foregroundColor(OneUI.secondary)
                }

                Button {
                    if runtime.loadedGameURL != nil { isPlayViewActive = true }
                } label: {
                    Label(runtime.loadedGameURL == nil ? "Select a game from Library" : "Open Player", systemImage: "play.circle.fill")
                        .font(.headline)
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 15)
                        .background(Capsule().fill(runtime.loadedGameURL == nil ? OneUI.muted : OneUI.accent))
                }
                .buttonStyle(.plain)
                .disabled(runtime.loadedGameURL == nil)
            }
            .padding(18)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
            .shadow(color: .black.opacity(0.05), radius: 18, y: 10)
        }
    }
}

struct SettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        AppScreen(title: "Settings", subtitle: "実際に保存・反映されるアプリ設定") {
            SettingsGroup(title: "Video") {
                SettingChoiceRow(title: "Scale", subtitle: "画面比率の扱い", value: runtime.videoScaleModeLabel) {
                    runtime.cycleVideoScaleMode()
                }
                SettingChoiceRow(title: "Filter", subtitle: "ピクセル描画品質", value: runtime.videoFilterLabel) {
                    runtime.toggleVideoFilter()
                }
                SettingToggleRow(title: "VSync", subtitle: "表示更新に同期", isOn: runtime.vsyncEnabled) {
                    runtime.setVsyncEnabled($0)
                }
            }

            SettingsGroup(title: "Audio") {
                SettingToggleRow(title: "Audio Output", subtitle: "音声出力を有効化", isOn: runtime.audioEnabledSetting) {
                    runtime.setAudioEnabled($0)
                }
                SettingToggleRow(title: "Audio Sync", subtitle: "音声同期", isOn: runtime.audioSyncSetting) {
                    runtime.setAudioSync($0)
                }
                SettingChoiceRow(title: "Latency", subtitle: "出力遅延", value: runtime.audioLatencyLabel) {
                    runtime.cycleAudioLatency()
                }
            }

            SettingsGroup(title: "Controller") {
                SettingToggleRow(title: "Touch Overlay", subtitle: "画面上コントローラー", isOn: runtime.overlayEnabledSetting) {
                    runtime.setOverlayEnabledSetting($0)
                }
                SettingChoiceRow(title: "Overlay Set", subtitle: "RetroArch overlay .cfg", value: runtime.overlaySelectionLabel) {
                    runtime.cycleOverlaySelection()
                }
                SettingChoiceRow(title: "Overlay Opacity", subtitle: "タッチ操作の透明度", value: runtime.overlayOpacityLabel) {
                    runtime.cycleOverlayOpacity()
                }
                SettingToggleRow(title: "Haptics", subtitle: "タッチ操作の振動フィードバック", isOn: runtime.hapticsEnabledSetting) {
                    runtime.setHapticsEnabled($0)
                }
            }

            SettingsGroup(title: "Library") {
                SettingChoiceRow(title: "Sort", subtitle: "ROMの並び順", value: runtime.librarySortLabel) {
                    runtime.cycleLibrarySort()
                }
                SettingToggleRow(title: "Core Badges", subtitle: "互換コア数をROMに表示", isOn: runtime.libraryCoreBadgesEnabled) {
                    runtime.setLibraryCoreBadgesEnabled($0)
                }
                SettingToggleRow(title: "File Details", subtitle: "拡張子とサイズを表示", isOn: runtime.libraryFileDetailsEnabled) {
                    runtime.setLibraryFileDetailsEnabled($0)
                }
                SettingToggleRow(title: "Auto Scan", subtitle: "起動時にROMを再スキャン", isOn: runtime.libraryAutoScanEnabled) {
                    runtime.setLibraryAutoScanEnabled($0)
                }
            }

            SettingsGroup(title: "Loaded Core") {
                SettingInfoRow(title: "Current Core", value: runtime.systemInfo?.libraryName ?? runtime.corePath ?? "Not loaded")
                if runtime.coreOptions.isEmpty {
                    SettingInfoRow(title: "Core Options", value: "Load a core to edit its options")
                } else {
                    ForEach(runtime.coreOptions, id: \.key) { option in
                        SettingChoiceRow(title: option.desc.isEmpty ? option.key : option.desc, subtitle: option.info.isEmpty ? option.key : option.info, value: option.value) {
                            guard let current = option.values.firstIndex(where: { $0.value == option.value }) else {
                                if let next = option.values.first { runtime.setOption(key: option.key, value: next.value) }
                                return
                            }
                            let next = option.values[(current + 1) % option.values.count]
                            runtime.setOption(key: option.key, value: next.value)
                        }
                    }
                }
            }

            SettingsGroup(title: "Storage") {
                SettingInfoRow(title: "Content Folder", value: runtime.settingValue("content_directory"))
                SettingInfoRow(title: "Core Folder", value: runtime.settingValue("libretro_directory"))
                SettingInfoRow(title: "Info Folder", value: runtime.settingValue("libretro_info_path"))
                SettingInfoRow(title: "Saves", value: runtime.settingValue("savefile_directory"))
                SettingInfoRow(title: "States", value: runtime.settingValue("savestate_directory"))
                SettingInfoRow(title: "System/BIOS", value: runtime.settingValue("system_directory"))
                SettingInfoRow(title: "Screenshots", value: runtime.settingValue("screenshot_directory"))
                Button {
                    runtime.installBundledAssets()
                } label: {
                    Label("Install bundled assets", systemImage: "arrow.down.doc.fill")
                        .font(.subheadline.bold())
                        .foregroundColor(OneUI.accent)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(16)
                        .background(OneUI.elevated)
                        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
                }
                .buttonStyle(.plain)
            }
        }
    }
}

struct CoreChoiceSheet: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.presentationMode) private var presentationMode

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 12) {
                    ForEach(runtime.pendingCoreChoices, id: \.path) { core in
                        Button {
                            runtime.launchPendingContent(with: core)
                            presentationMode.wrappedValue.dismiss()
                        } label: {
                            CoreRow(core: core)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(18)
            }
            .background(OneUI.background)
            .navigationBarTitle("Select Core", displayMode: .inline)
            .navigationBarItems(leading: Button("Cancel") { runtime.cancelCoreChoice() })
        }
        .navigationViewStyle(.stack)
    }
}

struct PrimaryAction: View {
    let title: String
    let subtitle: String
    let icon: String
    let tint: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.system(size: 17, weight: .bold))
                    .foregroundColor(.white)
                    .frame(width: 38, height: 38)
                    .background(Circle().fill(tint))
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.headline).foregroundColor(OneUI.ink)
                    Text(subtitle).font(.caption.weight(.medium)).foregroundColor(OneUI.secondary)
                }
                Spacer(minLength: 0)
            }
            .padding(14)
            .frame(maxWidth: .infinity, minHeight: 76)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
            .shadow(color: .black.opacity(0.05), radius: 14, y: 8)
        }
        .buttonStyle(.plain)
    }
}

struct StatusPill: View {
    let message: String

    var body: some View {
        Label(message, systemImage: "info.circle.fill")
            .font(.footnote.weight(.medium))
            .foregroundColor(OneUI.secondary)
            .lineLimit(2)
            .padding(.horizontal, 14)
            .padding(.vertical, 11)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Capsule().fill(OneUI.surface))
    }
}

struct ContentSection<Content: View>: View {
    let title: String
    let count: Int
    @ViewBuilder var content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(title)
                    .font(.title3.bold())
                    .foregroundColor(OneUI.ink)
                Spacer()
                Text("\(count)")
                    .font(.caption.bold())
                    .foregroundColor(OneUI.secondary)
                    .padding(.horizontal, 9)
                    .padding(.vertical, 5)
                    .background(Capsule().fill(OneUI.surface))
            }
            content
        }
    }
}

struct GameRow: View {
    let game: GameEntrySwift
    let details: String
    let compatibility: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                RoundedIcon(systemName: "play.fill", tint: OneUI.accent)
                VStack(alignment: .leading, spacing: 4) {
                    Text(game.label)
                        .font(.headline)
                        .foregroundColor(OneUI.ink)
                        .lineLimit(1)
                    Text(details)
                        .font(.caption)
                        .foregroundColor(OneUI.secondary)
                        .lineLimit(1)
                    Text(compatibility)
                        .font(.caption2.weight(.semibold))
                        .foregroundColor(compatibility.hasPrefix("No") ? .orange : OneUI.teal)
                        .lineLimit(1)
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.caption.bold())
                    .foregroundColor(OneUI.muted)
            }
            .padding(15)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

struct CoreRow: View {
    let core: CoreInfo

    var body: some View {
        HStack(spacing: 14) {
            RoundedIcon(systemName: "cpu.fill", tint: OneUI.violet)
            VStack(alignment: .leading, spacing: 4) {
                Text(core.displayName)
                    .font(.headline)
                    .foregroundColor(OneUI.ink)
                    .lineLimit(1)
                Text([core.systemName, core.supportedExtensions.joined(separator: ", ")].filter { !$0.isEmpty }.joined(separator: " • "))
                    .font(.caption)
                    .foregroundColor(OneUI.secondary)
                    .lineLimit(2)
            }
            Spacer(minLength: 0)
        }
        .padding(15)
        .background(OneUI.surface)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
    }
}

struct LibraryStatCard: View {
    let title: String
    let value: String
    let icon: String
    let tint: Color

    var body: some View {
        HStack(spacing: 10) {
            RoundedIcon(systemName: icon, tint: tint)
            VStack(alignment: .leading, spacing: 2) {
                Text(value).font(.title3.bold()).foregroundColor(OneUI.ink)
                Text(title).font(.caption.weight(.semibold)).foregroundColor(OneUI.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
        .frame(maxWidth: .infinity)
        .background(OneUI.surface)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
    }
}

struct RoundedIcon: View {
    let systemName: String
    let tint: Color

    var body: some View {
        Image(systemName: systemName)
            .font(.system(size: 16, weight: .bold))
            .foregroundColor(tint)
            .frame(width: 42, height: 42)
            .background(RoundedRectangle(cornerRadius: 14, style: .continuous).fill(tint.opacity(0.12)))
    }
}

struct EmptyPanel: View {
    let icon: String
    let title: String
    let message: String

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 28, weight: .semibold))
                .foregroundColor(OneUI.muted)
            Text(title)
                .font(.headline)
                .foregroundColor(OneUI.ink)
            Text(message)
                .font(.subheadline)
                .multilineTextAlignment(.center)
                .foregroundColor(OneUI.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(24)
        .background(OneUI.surface)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
    }
}

struct SettingsGroup<Content: View>: View {
    let title: String
    @ViewBuilder var content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title)
                .font(.title3.bold())
                .foregroundColor(OneUI.ink)
            VStack(spacing: 1) { content }
                .padding(6)
                .background(OneUI.surface)
                .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
        }
    }
}

struct SettingChoiceRow: View {
    let title: String
    let subtitle: String
    let value: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 3) {
                    Text(title).font(.subheadline.bold()).foregroundColor(OneUI.ink)
                    Text(subtitle).font(.caption).foregroundColor(OneUI.secondary)
                }
                Spacer()
                Text(value)
                    .font(.caption.bold())
                    .foregroundColor(OneUI.accent)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(Capsule().fill(OneUI.accent.opacity(0.10)))
            }
            .padding(12)
            .background(OneUI.elevated)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

struct SettingToggleRow: View {
    let title: String
    let subtitle: String
    let isOn: Bool
    let onChange: (Bool) -> Void

    var body: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 3) {
                Text(title).font(.subheadline.bold()).foregroundColor(OneUI.ink)
                Text(subtitle).font(.caption).foregroundColor(OneUI.secondary)
            }
            Spacer()
            Toggle("", isOn: Binding(get: { isOn }, set: onChange))
                .labelsHidden()
                .toggleStyle(SwitchToggleStyle(tint: OneUI.accent))
        }
        .padding(12)
        .background(OneUI.elevated)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
    }
}

struct SettingInfoRow: View {
    let title: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.subheadline.bold()).foregroundColor(OneUI.ink)
            Text(value.isEmpty ? "Not set" : value)
                .font(.caption)
                .foregroundColor(OneUI.secondary)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(OneUI.elevated)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
    }
}

struct PlayView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.presentationMode) private var presentationMode

    @State private var isMenuPresented = false

    var body: some View {
        GeometryReader { outer in
            ZStack(alignment: .bottom) {
                Color.black.ignoresSafeArea()
                if let image = runtime.displayImage {
                    Image(uiImage: image)
                        .resizable()
                        .interpolation(.none)
                        .scaledToFit()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    VStack(spacing: 10) {
                        Image(systemName: "gamecontroller")
                            .font(.system(size: 42, weight: .semibold))
                        Text("Starting video…")
                            .font(.headline)
                    }
                    .foregroundColor(.white.opacity(0.78))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
                if runtime.overlayEnabledSetting, runtime.overlayInfo?.enabled == true {
                    RetroArchOverlayView()
                    PlayerUtilityBar { presentationMode.wrappedValue.dismiss() }
                        .padding(.horizontal, 10)
                        .padding(.bottom, 14)
                } else {
                    PlayerControls { presentationMode.wrappedValue.dismiss() }
                        .padding(.horizontal, 10)
                        .background(LinearGradient(colors: [.clear, .black.opacity(0.78)], startPoint: .top, endPoint: .bottom).ignoresSafeArea(edges: .bottom))
                }
            }
            .contentShape(Rectangle())
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in runtime.setOverlayTouch(slot: 0, location: value.location, in: outer.size, active: true) }
                    .onEnded { _ in runtime.setOverlayTouch(slot: 0, location: .zero, in: outer.size, active: false) }
            )
        }
        .background(Color.black.ignoresSafeArea())
        .statusBar(hidden: true)
        .onReceive(runtime.$menuToken) { token in if token > 0 { isMenuPresented = true } }
        .sheet(isPresented: $isMenuPresented) { RuntimeMenuSheet() }
        .onAppear {
            runtime.setOverlayOrientation(for: UIScreen.main.bounds.size)
            runtime.play()
        }
        .onDisappear {
            runtime.clearOverlayTouches()
            runtime.stop()
        }
    }
}

struct RetroArchOverlayView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        GeometryReader { geometry in
            ZStack(alignment: .topLeading) {
                ForEach(Array(runtime.overlayRenderDescs().enumerated()), id: \.offset) { _, desc in
                    if let image = UIImage(contentsOfFile: desc.imagePath) {
                        Image(uiImage: image)
                            .resizable()
                            .interpolation(.none)
                            .opacity(Double(desc.alpha))
                            .frame(width: CGFloat(desc.w) * geometry.size.width, height: CGFloat(desc.h) * geometry.size.height)
                            .position(
                                x: CGFloat(desc.x + desc.w * 0.5) * geometry.size.width,
                                y: CGFloat(desc.y + desc.h * 0.5) * geometry.size.height)
                    }
                }
            }
            .frame(width: geometry.size.width, height: geometry.size.height)
            .allowsHitTesting(false)
            .onAppear { runtime.setOverlayOrientation(for: geometry.size) }
            .onChange(of: geometry.size) { newSize in runtime.setOverlayOrientation(for: newSize) }
        }
    }
}

struct PlayerUtilityBar: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let dismiss: () -> Void

    var body: some View {
        HStack(spacing: 14) {
            Button { runtime.openQuickMenu() } label: {
                Image(systemName: "line.3.horizontal")
                    .font(.system(size: 18, weight: .bold))
                    .foregroundColor(.white)
                    .frame(width: 48, height: 42)
                    .background(Capsule().fill(Color.black.opacity(0.38)))
            }
            .buttonStyle(.plain)
            Button { runtime.toggleRunning() } label: {
                Image(systemName: runtime.isRunning ? "pause.fill" : "play.fill")
                    .font(.system(size: 18, weight: .bold))
                    .foregroundColor(.white)
                    .frame(width: 58, height: 42)
                    .background(Capsule().fill(OneUI.accent.opacity(0.9)))
            }
            .buttonStyle(.plain)
            Button {
                runtime.stop()
                dismiss()
            } label: {
                Image(systemName: "xmark")
                    .font(.system(size: 16, weight: .bold))
                    .foregroundColor(.white)
                    .frame(width: 48, height: 42)
                    .background(Capsule().fill(Color.black.opacity(0.38)))
            }
            .buttonStyle(.plain)
        }
    }
}

struct RuntimeMenuSheet: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.presentationMode) private var presentationMode

    var body: some View {
        NavigationView {
            List {
                ForEach(runtime.currentMenu?.entries ?? [], id: \.actionId) { entry in
                    Button {
                        runtime.menuAction(entry.actionId)
                    } label: {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(entry.label).foregroundColor(OneUI.ink)
                            if !entry.sublabel.isEmpty { Text(entry.sublabel).font(.caption).foregroundColor(OneUI.secondary) }
                        }
                    }
                    .listRowBackground(OneUI.surface)
                }
            }
            .scrollContentBackground(.hidden)
            .background(OneUI.background)
            .navigationBarTitle(runtime.currentMenu?.title ?? "Menu", displayMode: .inline)
            .navigationBarItems(leading: Button("Back") { runtime.menuPop() }, trailing: Button("Done") { presentationMode.wrappedValue.dismiss() })
        }
        .preferredColorScheme(.dark)
    }
}

struct PlayerControls: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let dismiss: () -> Void

    var body: some View {
        VStack(spacing: 14) {
            HStack {
                ShoulderButton(label: "L", button: .l)
                Spacer()
                ShoulderButton(label: "R", button: .r)
            }
            .padding(.horizontal, 26)

            HStack(alignment: .center) {
                Dpad()
                Spacer()
                VStack(spacing: 18) {
                    HStack(spacing: 18) {
                        ActionButton(label: "Y", button: .y)
                        ActionButton(label: "X", button: .x)
                    }
                    HStack(spacing: 18) {
                        ActionButton(label: "B", button: .b)
                        ActionButton(label: "A", button: .a)
                    }
                }
            }
            .padding(.horizontal, 28)

            HStack(spacing: 18) {
                UtilityButton(label: "Select", button: .select)
                Button { runtime.openQuickMenu() } label: {
                    Image(systemName: "line.3.horizontal")
                        .font(.system(size: 18, weight: .bold))
                        .foregroundColor(.white)
                        .frame(width: 44, height: 42)
                        .background(Capsule().fill(Color.white.opacity(0.14)))
                }
                .buttonStyle(.plain)
                Button { runtime.toggleRunning() } label: {
                    Image(systemName: runtime.isRunning ? "pause.fill" : "play.fill")
                        .font(.system(size: 18, weight: .bold))
                        .foregroundColor(.white)
                        .frame(width: 58, height: 42)
                        .background(Capsule().fill(OneUI.accent))
                }
                .buttonStyle(.plain)
                UtilityButton(label: "Start", button: .start)
                Button {
                    runtime.stop()
                    dismiss()
                } label: {
                    Image(systemName: "xmark")
                        .font(.system(size: 16, weight: .bold))
                        .foregroundColor(.white)
                        .frame(width: 44, height: 42)
                        .background(Capsule().fill(Color.white.opacity(0.14)))
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.bottom, 18)
    }
}

struct Dpad: View {
    var body: some View {
        VStack(spacing: 8) {
            ControllerButton(icon: "chevron.up", label: nil, button: .up, size: 52)
            HStack(spacing: 8) {
                ControllerButton(icon: "chevron.left", label: nil, button: .left, size: 52)
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill(Color.white.opacity(0.08))
                    .frame(width: 52, height: 52)
                ControllerButton(icon: "chevron.right", label: nil, button: .right, size: 52)
            }
            ControllerButton(icon: "chevron.down", label: nil, button: .down, size: 52)
        }
    }
}

struct ShoulderButton: View {
    let label: String
    let button: JoypadButton

    var body: some View {
        ControllerButton(icon: nil, label: label, button: button, size: 78, height: 40)
    }
}

struct UtilityButton: View {
    let label: String
    let button: JoypadButton

    var body: some View {
        ControllerButton(icon: nil, label: label, button: button, size: 76, height: 42)
    }
}

struct ActionButton: View {
    let label: String
    let button: JoypadButton

    var body: some View {
        ControllerButton(icon: nil, label: label, button: button, size: 58)
    }
}

struct ControllerButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let icon: String?
    let label: String?
    let button: JoypadButton
    let size: CGFloat
    var height: CGFloat? = nil

    var body: some View {
        Button {} label: {
            Group {
                if let icon = icon {
                    Image(systemName: icon)
                } else {
                    Text(label ?? "")
                }
            }
            .font(.system(size: 16, weight: .bold))
            .foregroundColor(.white)
            .frame(width: size, height: height ?? size)
            .background(RoundedRectangle(cornerRadius: 18, style: .continuous).fill(Color.white.opacity(0.13)))
            .overlay(RoundedRectangle(cornerRadius: 18, style: .continuous).stroke(Color.white.opacity(0.16), lineWidth: 1))
        }
        .buttonStyle(.plain)
        .simultaneousGesture(
            DragGesture(minimumDistance: 0)
                .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
        )
    }
}
