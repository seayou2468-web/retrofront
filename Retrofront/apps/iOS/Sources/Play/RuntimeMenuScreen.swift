import SwiftUI
import RetrofrontSwift

struct RuntimeMenuScreen: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPresented: Bool
    let dismissPlayer: () -> Void

    var body: some View {
        ZStack {
            Color.black.opacity(0.72).ignoresSafeArea()
            OneUI.ashChromeBackground.ignoresSafeArea()

            VStack(spacing: 0) {
                QuickMenuHeader(title: runtime.currentMenu?.title ?? "Quick Menu", subtitle: runtime.loadedGameURL?.lastPathComponent ?? "No game loaded", onBack: runtime.menuPop) {
                    isPresented = false
                }

                ScrollView {
                    VStack(alignment: .leading, spacing: 18) {
                        if runtime.currentMenu?.title == "Quick Menu" {
                            QuickActionGrid(isPresented: $isPresented, dismissPlayer: dismissPlayer)
                        }

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
            QuickMenuCard(title: "Save State", subtitle: "Slot \(runtime.stateSlotLabel)に保存", icon: "square.and.arrow.down.fill", tint: OneUI.teal) {
                runtime.saveState()
            }
            QuickMenuCard(title: "Load State", subtitle: "Slot \(runtime.stateSlotLabel)から復元", icon: "arrow.counterclockwise.circle.fill", tint: OneUI.violet) {
                runtime.loadState()
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
        case 14, 27, 28, 29, 30, 31, 32: return "externaldrive.fill"
        case 22, 211: return "display"
        case 19, 25, 213: return "gamecontroller.fill"
        case 23, 212: return "speaker.wave.2.fill"
        case 26, 12: return "xmark.circle.fill"
        case 13: return "sparkles"
        case 15: return "camera.fill"
        case 16, 36: return "star.fill"
        case 17: return "bolt.fill"
        case 18: return "doc.badge.gearshape.fill"
        case 24: return "opticaldisc.fill"
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

