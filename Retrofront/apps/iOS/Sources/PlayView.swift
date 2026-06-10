import SwiftUI
import UIKit

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
            .onAppear { runtime.setOverlayOrientation(for: outer.size) }
            .onChange(of: outer.size) { _, newSize in runtime.setOverlayOrientation(for: newSize) }
        }
        .background(Color.black.ignoresSafeArea())
        .statusBar(hidden: true)
        .onReceive(runtime.$menuToken) { token in if token > 0 { isMenuPresented = true } }
        .fullScreenCover(isPresented: $isMenuPresented) { RuntimeMenuScreen(isPresented: $isMenuPresented) { presentationMode.wrappedValue.dismiss() } }
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
            .onChange(of: geometry.size) { _, newSize in runtime.setOverlayOrientation(for: newSize) }
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

struct RuntimeMenuScreen: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPresented: Bool
    let dismissPlayer: () -> Void

    var body: some View {
        ZStack {
            Color.black.opacity(0.72).ignoresSafeArea()
            LinearGradient(colors: [OneUI.background.opacity(0.92), OneUI.surface.opacity(0.84)], startPoint: .topLeading, endPoint: .bottomTrailing)
                .ignoresSafeArea()

            VStack(spacing: 0) {
                QuickMenuHeader(title: runtime.currentMenu?.title ?? "Quick Menu", subtitle: runtime.loadedGameURL?.lastPathComponent ?? "No game loaded", onBack: runtime.menuPop) {
                    isPresented = false
                }

                ScrollView {
                    VStack(alignment: .leading, spacing: 18) {
                        QuickActionGrid(isPresented: $isPresented, dismissPlayer: dismissPlayer)

                        if runtime.currentMenu?.title == "Core", !runtime.coreOptions.isEmpty {
                            QuickCoreOptionsSection()
                        }

                        RuntimeMenuEntries(isPresented: $isPresented, dismissPlayer: dismissPlayer)
                    }
                    .padding(.horizontal, 18)
                    .padding(.bottom, 32)
                }
            }
        }
        .preferredColorScheme(.dark)
    }
}

struct QuickMenuHeader: View {
    let title: String
    let subtitle: String
    let onBack: () -> Void
    let onClose: () -> Void

    var body: some View {
        HStack(spacing: 14) {
            RoundedIcon(systemName: "line.3.horizontal.circle.fill", tint: OneUI.accent)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.title2.bold())
                    .foregroundColor(OneUI.ink)
                Text(subtitle)
                    .font(.caption.weight(.medium))
                    .foregroundColor(OneUI.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            Spacer()
            Button(action: onBack) {
                Image(systemName: "chevron.left")
                    .font(.system(size: 15, weight: .bold))
                    .foregroundColor(OneUI.ink)
                    .frame(width: 42, height: 42)
                    .background(Circle().fill(Color.white.opacity(0.12)))
            }
            .buttonStyle(.plain)
            Button(action: onClose) {
                Image(systemName: "xmark")
                    .font(.system(size: 15, weight: .bold))
                    .foregroundColor(OneUI.ink)
                    .frame(width: 42, height: 42)
                    .background(Circle().fill(Color.white.opacity(0.12)))
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 18)
        .padding(.top, 18)
        .padding(.bottom, 14)
        .background(.ultraThinMaterial)
    }
}

struct QuickActionGrid: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPresented: Bool
    let dismissPlayer: () -> Void

    var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
            QuickMenuCard(title: runtime.isRunning ? "Pause" : "Resume", subtitle: "実行状態を切替", icon: runtime.isRunning ? "pause.fill" : "play.fill", tint: OneUI.accent) {
                runtime.toggleRunning()
            }
            QuickMenuCard(title: "Restart", subtitle: "現在のゲームをリセット", icon: "restart.circle.fill", tint: .orange) {
                runtime.resetContent()
            }
            QuickMenuCard(title: "Save State", subtitle: "Slot 0に保存", icon: "square.and.arrow.down.fill", tint: OneUI.teal) {
                runtime.saveState(slot: 0)
            }
            QuickMenuCard(title: "Load State", subtitle: "Slot 0から復元", icon: "arrow.counterclockwise.circle.fill", tint: OneUI.violet) {
                runtime.loadState(slot: 0)
            }
            QuickMenuCard(title: "Save SRAM", subtitle: "実セーブを即書き込み", icon: "sdcard.fill", tint: .cyan) {
                runtime.saveSRAMNow()
            }
            QuickMenuCard(title: "Exit Game", subtitle: "保存して終了", icon: "xmark.circle.fill", tint: .red) {
                runtime.closeContent()
                isPresented = false
                dismissPlayer()
            }
        }
    }
}

struct QuickMenuCard: View {
    let title: String
    let subtitle: String
    let icon: String
    let tint: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 10) {
                RoundedIcon(systemName: icon, tint: tint)
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.headline).foregroundColor(OneUI.ink)
                    Text(subtitle).font(.caption).foregroundColor(OneUI.secondary)
                }
                Spacer(minLength: 0)
            }
            .frame(maxWidth: .infinity, minHeight: 122, alignment: .topLeading)
            .padding(14)
            .background(OneUI.elevated.opacity(0.88))
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

struct QuickCoreOptionsSection: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        SettingsGroup(title: "Core Options") {
            ForEach(runtime.coreOptions, id: \.key) { option in
                SettingPickerRow(
                    title: option.desc.isEmpty ? option.key : option.desc,
                    subtitle: option.info.isEmpty ? option.key : option.info,
                    value: option.value,
                    choices: option.values.map { (label: $0.label.isEmpty ? $0.value : $0.label, value: $0.value) }
                ) { choice in
                    runtime.setOption(key: option.key, value: choice.value)
                }
            }
        }
    }
}

struct RuntimeMenuEntries: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPresented: Bool
    let dismissPlayer: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("RetroArch Menu")
                .font(.title3.bold())
                .foregroundColor(OneUI.ink)
            VStack(spacing: 8) {
                ForEach(runtime.currentMenu?.entries ?? [], id: \.actionId) { entry in
                    Button {
                        activate(entry)
                    } label: {
                        HStack(spacing: 12) {
                            Image(systemName: icon(for: entry))
                                .font(.system(size: 15, weight: .bold))
                                .foregroundColor(tint(for: entry))
                                .frame(width: 34, height: 34)
                                .background(Circle().fill(tint(for: entry).opacity(0.13)))
                            VStack(alignment: .leading, spacing: 3) {
                                Text(entry.label).font(.subheadline.bold()).foregroundColor(OneUI.ink)
                                if !entry.sublabel.isEmpty { Text(entry.sublabel).font(.caption).foregroundColor(OneUI.secondary).lineLimit(2) }
                                if !entry.value.isEmpty { Text(entry.value).font(.caption2.bold()).foregroundColor(OneUI.accent).lineLimit(1).truncationMode(.middle) }
                            }
                            Spacer()
                            if entry.kind == .submenu { Image(systemName: "chevron.right").font(.caption.bold()).foregroundColor(OneUI.muted) }
                        }
                        .padding(12)
                        .background(OneUI.surface.opacity(0.9))
                        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }

    private func activate(_ entry: MenuEntry) {
        if entry.actionId == 10 {
            isPresented = false
            return
        }
        runtime.menuAction(entry.actionId)
        if entry.actionId == 12 || entry.actionId == 26 {
            isPresented = false
            dismissPlayer()
        }
    }

    private func icon(for entry: MenuEntry) -> String {
        switch entry.actionId {
        case 9: return "restart.circle.fill"
        case 11, 21, 222: return "slider.horizontal.3"
        case 14, 27, 28, 29: return "externaldrive.fill"
        case 22, 211: return "display"
        case 19, 25, 213: return "gamecontroller.fill"
        case 23, 212: return "speaker.wave.2.fill"
        case 26, 12: return "xmark.circle.fill"
        default: return entry.kind == .submenu ? "folder.fill" : "circle.fill"
        }
    }

    private func tint(for entry: MenuEntry) -> Color {
        if entry.actionId == 26 || entry.actionId == 12 { return .red }
        if entry.kind == .submenu { return OneUI.accent }
        if entry.kind == .setting { return OneUI.teal }
        return OneUI.violet
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
