import SwiftUI
import RetrofrontSwift

struct RuntimeMenuScreen: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPresented: Bool
    let dismissPlayer: () -> Void

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        GeometryReader { proxy in
            let metrics = runtime.frontend?.menuLayoutMetrics(width: UInt32(max(proxy.size.width, 1)), height: UInt32(max(proxy.size.height, 1)))
            ZStack {
                Color.black.opacity(0.72).ignoresSafeArea()
                RetroArchMenuBackground(skin: skin).ignoresSafeArea()
                RetroArchDriverMenuChrome(skin: skin, metrics: metrics, isPresented: $isPresented, dismissPlayer: dismissPlayer)
            }
        }
        .preferredColorScheme(.dark)
    }
}

struct RetroArchDriverMenuChrome: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let skin: RetroArchMenuSkin
    let metrics: MenuLayoutMetrics?
    @Binding var isPresented: Bool
    let dismissPlayer: () -> Void

    private var title: String { runtime.currentMenu?.title ?? "Quick Menu" }
    private var subtitle: String { runtime.loadedGameURL?.lastPathComponent ?? skin.displayName }
    private var entries: [MenuEntry] { runtime.currentMenu?.entries ?? [] }

    var body: some View {
        switch skin.layout {
        case .material:
            materialUI
        case .ozone:
            ozoneUI
        case .xmb:
            xmbUI
        case .rgui:
            rguiUI
        }
    }

    private var materialUI: some View {
        VStack(spacing: 0) {
            HStack(spacing: 16) {
                RetroArchMenuAssetImage(url: skin.assets.iconURL(named: "menu"), systemName: "line.3.horizontal")
                    .frame(width: 24, height: 24)
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.system(size: 20, weight: .medium)).foregroundColor(skin.palette.ink)
                    Text(subtitle).font(.caption).foregroundColor(skin.palette.secondary).lineLimit(1)
                }
                Spacer()
                closeButton
            }
            .padding(.horizontal, 16)
            .frame(height: CGFloat(metrics?.headerHeight ?? 64))
            .background(skin.palette.surface.opacity(0.98))

            ScrollView {
                VStack(spacing: 1) {
                    quickSections
                    ForEach(Array(entries.enumerated()), id: \.offset) { index, entry in
                        RetroArchMenuEntryRow(entry: entry, skin: skin, isSelected: index == 0, style: .material) {
                            activate(entry)
                        }
                    }
                }
                .padding(.vertical, 10)
            }

            HStack(spacing: 0) {
                materialTab("Main Menu", asset: "history", system: "house.fill")
                materialTab("Quick Menu", asset: "menu", system: "line.3.horizontal.circle.fill")
                materialTab("Settings", asset: "settings", system: "gearshape.fill")
            }
            .frame(height: CGFloat(metrics?.footerHeight ?? 58))
            .background(skin.palette.surface.opacity(0.98))
        }
    }

    private var ozoneUI: some View {
        HStack(spacing: 0) {
            VStack(spacing: 26) {
                driverRailIcon("history", "house.fill", selected: title == "Main Menu")
                driverRailIcon("load-content", "rectangle.stack.fill", selected: false)
                driverRailIcon("core", "cpu.fill", selected: title == "Core" || title == "Quick Menu")
                driverRailIcon("settings", "gearshape.fill", selected: title.contains("Settings"))
                Spacer()
                closeButton
            }
            .padding(.top, 38)
            .padding(.bottom, 20)
            .frame(width: CGFloat(metrics?.sidebarWidth ?? 92))
            .background(skin.palette.surface.opacity(0.84))

            VStack(alignment: .leading, spacing: 0) {
                HStack(alignment: .firstTextBaseline) {
                    VStack(alignment: .leading, spacing: 5) {
                        Text(title).font(.system(size: 30, weight: .semibold)).foregroundColor(skin.palette.ink)
                        Text(subtitle).font(.caption.weight(.medium)).foregroundColor(skin.palette.secondary).lineLimit(1)
                    }
                    Spacer()
                    Text(skin.displayName).font(.caption.bold()).foregroundColor(skin.palette.accent)
                }
                .padding(.horizontal, CGFloat(metrics?.horizontalPadding ?? 28))
                .padding(.top, CGFloat(metrics?.verticalPadding ?? 26))
                .padding(.bottom, 18)

                HStack(alignment: .top, spacing: 18) {
                    ScrollView {
                        VStack(spacing: 6) {
                            quickSections
                            ForEach(Array(entries.enumerated()), id: \.offset) { index, entry in
                                RetroArchMenuEntryRow(entry: entry, skin: skin, isSelected: index == 0, style: .ozone) {
                                    activate(entry)
                                }
                            }
                        }
                    }
                    .frame(maxWidth: .infinity)

                    VStack(alignment: .leading, spacing: 12) {
                        Text("Details").font(.headline).foregroundColor(skin.palette.ink)
                        Text(entries.first?.sublabel.isEmpty == false ? entries.first?.sublabel ?? "" : "Select a menu entry to open the RetroArch action.")
                            .font(.caption)
                            .foregroundColor(skin.palette.secondary)
                        Spacer()
                    }
                    .padding(18)
                    .frame(width: 240)
                    .frame(maxHeight: .infinity)
                    .background(skin.palette.surface.opacity(0.72))
                    .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                }
                .padding(.horizontal, CGFloat(metrics?.horizontalPadding ?? 28))
                .padding(.bottom, CGFloat(metrics?.verticalPadding ?? 24))
            }
        }
    }

    private var xmbUI: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 34) {
                driverRailIcon("history", "house.fill", selected: title == "Main Menu")
                driverRailIcon("load-content", "rectangle.stack.fill", selected: false)
                driverRailIcon("core", "cpu.fill", selected: title == "Core" || title == "Quick Menu")
                driverRailIcon("settings", "gearshape.fill", selected: title.contains("Settings"))
                Spacer()
                closeButton
            }
            .padding(.horizontal, 28)
            .padding(.top, 54)

            Text(title)
                .font(.system(size: 26, weight: .light))
                .foregroundColor(skin.palette.ink)
                .padding(.leading, 34)
                .padding(.top, 22)

            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    quickSections
                    ForEach(Array(entries.enumerated()), id: \.offset) { index, entry in
                        RetroArchMenuEntryRow(entry: entry, skin: skin, isSelected: index == 0, style: .xmb) {
                            activate(entry)
                        }
                    }
                }
                .padding(.leading, CGFloat(metrics?.horizontalPadding ?? 80))
                .padding(.trailing, CGFloat(metrics?.horizontalPadding ?? 32))
                .padding(.top, 20)
            }
        }
    }

    private var rguiUI: some View {
        VStack(spacing: 0) {
            Text("┌─ RETROARCH / \(title.uppercased()) ─────────────────────────────┐")
                .font(.system(size: 13, weight: .regular, design: .monospaced))
                .foregroundColor(skin.palette.accent)
                .lineLimit(1)
            ScrollView {
                VStack(spacing: 0) {
                    quickSections
                    ForEach(Array(entries.enumerated()), id: \.offset) { index, entry in
                        RetroArchMenuEntryRow(entry: entry, skin: skin, isSelected: index == 0, style: .rgui) {
                            activate(entry)
                        }
                    }
                }
                .padding(.vertical, 8)
            }
            Text("└────────────────────────────────────────────────────────────────┘")
                .font(.system(size: 13, weight: .regular, design: .monospaced))
                .foregroundColor(skin.palette.accent)
                .lineLimit(1)
        }
        .padding(12)
        .background(Color.black.opacity(0.82))
        .overlay(Rectangle().stroke(skin.palette.accent, lineWidth: 2))
        .padding(14)
    }

    @ViewBuilder
    private var quickSections: some View {
        EmptyView()
    }

    private var closeButton: some View {
        Button { isPresented = false } label: {
            Image(systemName: "xmark")
                .font(.system(size: 14, weight: .bold))
                .foregroundColor(skin.palette.ink)
                .frame(width: 38, height: 38)
                .background(Circle().fill(skin.palette.elevated.opacity(0.88)))
        }
        .buttonStyle(.plain)
    }

    private func materialTab(_ text: String, asset: String, system: String) -> some View {
        VStack(spacing: 3) {
            RetroArchMenuAssetImage(url: skin.assets.iconURL(named: asset), systemName: system)
                .frame(width: 22, height: 22)
            Text(text).font(.caption2.weight(.semibold)).lineLimit(1)
        }
        .foregroundColor(text == title ? skin.palette.accent : skin.palette.secondary)
        .frame(maxWidth: .infinity)
    }

    private func driverRailIcon(_ asset: String, _ system: String, selected: Bool) -> some View {
        RetroArchMenuAssetImage(url: skin.assets.iconURL(named: asset), systemName: system)
            .foregroundColor(selected ? skin.palette.accent : skin.palette.secondary)
            .frame(width: selected ? 34 : 28, height: selected ? 34 : 28)
            .shadow(color: selected ? skin.palette.accent.opacity(0.5) : .clear, radius: 12)
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
}

enum RetroArchMenuEntryRowStyle {
    case material
    case ozone
    case xmb
    case rgui
}

struct RetroArchMenuEntryRow: View {
    let entry: MenuEntry
    let skin: RetroArchMenuSkin
    let isSelected: Bool
    let style: RetroArchMenuEntryRowStyle
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            switch style {
            case .material:
                materialRow
            case .ozone:
                ozoneRow
            case .xmb:
                xmbRow
            case .rgui:
                rguiRow
            }
        }
        .buttonStyle(.plain)
    }

    private var materialRow: some View {
        HStack(spacing: 16) {
            icon.frame(width: 24, height: 24).foregroundColor(skin.palette.secondary)
            rowText(titleFont: skin.rowFont, subtitleFont: skin.subtitleFont)
            trailing
        }
        .padding(.horizontal, 18)
        .frame(minHeight: 56)
        .background(isSelected ? skin.palette.elevated : skin.palette.surface.opacity(0.94))
    }

    private var ozoneRow: some View {
        HStack(spacing: 14) {
            Rectangle().fill(isSelected ? skin.palette.accent : .clear).frame(width: 4)
            icon.frame(width: 26, height: 26).foregroundColor(isSelected ? skin.palette.ink : skin.palette.secondary)
            rowText(titleFont: skin.rowFont.weight(isSelected ? .semibold : .regular), subtitleFont: skin.subtitleFont)
            trailing
        }
        .padding(.trailing, 14)
        .frame(minHeight: 50)
        .background(isSelected ? skin.palette.elevated.opacity(0.92) : Color.clear)
        .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
    }

    private var xmbRow: some View {
        HStack(spacing: 18) {
            icon.frame(width: isSelected ? 42 : 34, height: isSelected ? 42 : 34).foregroundColor(skin.palette.ink)
            rowText(titleFont: .system(size: isSelected ? 22 : 18, weight: .light), subtitleFont: .caption)
            trailing
        }
        .padding(.horizontal, 8)
        .frame(minHeight: isSelected ? 64 : 50)
        .shadow(color: .black.opacity(0.55), radius: 2, x: 1, y: 1)
    }

    private var rguiRow: some View {
        HStack(spacing: 8) {
            Text(isSelected ? "▶" : " ")
            Text(entry.label.uppercased()).lineLimit(1)
            Spacer(minLength: 8)
            if !entry.value.isEmpty { Text(entry.value.uppercased()).lineLimit(1) }
            if entry.kind == .submenu { Text("›") }
        }
        .font(.system(size: 15, weight: .regular, design: .monospaced))
        .foregroundColor(isSelected ? skin.palette.background : skin.palette.ink)
        .padding(.horizontal, 10)
        .frame(height: 25)
        .background(isSelected ? skin.palette.accent : Color.clear)
    }

    private var icon: some View {
        RetroArchMenuAssetImage(url: skin.iconURL(for: entry.actionId), systemName: skin.iconName(for: entry.actionId))
    }

    private func rowText(titleFont: Font, subtitleFont: Font) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(entry.label)
                .font(titleFont)
                .foregroundColor(skin.palette.ink)
                .lineLimit(1)
            if !entry.sublabel.isEmpty {
                Text(entry.sublabel)
                    .font(subtitleFont)
                    .foregroundColor(skin.palette.secondary)
                    .lineLimit(2)
            }
            if !entry.value.isEmpty && style != .rgui {
                Text(entry.value)
                    .font(.caption2.bold())
                    .foregroundColor(skin.palette.accent)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private var trailing: some View {
        if entry.kind == .submenu {
            Image(systemName: "chevron.right")
                .font(.caption.bold())
                .foregroundColor(skin.palette.secondary)
        }
    }
}
