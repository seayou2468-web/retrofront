import SwiftUI
import RetrofrontSwift

struct PrimaryAction: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let subtitle: String
    let icon: String
    let tint: Color
    let action: () -> Void

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.system(size: 17, weight: .bold))
                    .foregroundColor(.white)
                    .frame(width: 38, height: 38)
                    .background(Circle().fill(tint))
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(skin.rowFont.weight(.semibold)).foregroundColor(skin.palette.ink)
                    Text(subtitle).font(skin.subtitleFont.weight(.medium)).foregroundColor(skin.palette.secondary)
                }
                Spacer(minLength: 0)
            }
            .padding(14)
            .frame(maxWidth: .infinity, minHeight: 76)
            .background(skin.palette.surface)
            .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius == 0 ? 0 : RetroArchMenuMetrics.radius, style: .continuous))
            .shadow(color: .black.opacity(0.05), radius: 14, y: 8)
        }
        .buttonStyle(.plain)
    }
}

struct StatusPill: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let message: String

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        Label(message, systemImage: "info.circle.fill")
            .font(.footnote.weight(.medium))
            .foregroundColor(skin.palette.secondary)
            .lineLimit(2)
            .padding(.horizontal, 14)
            .padding(.vertical, 11)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Capsule().fill(skin.palette.surface))
    }
}

struct ContentSection<Content: View>: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let count: Int
    @ViewBuilder var content: Content

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        VStack(alignment: .leading, spacing: skin.rowSpacing) {
            HStack {
                Text(title)
                    .font(skin.titleFont.weight(.bold))
                    .foregroundColor(skin.palette.ink)
                Spacer()
                Text("\(count)")
                    .font(.caption.bold())
                    .foregroundColor(skin.palette.secondary)
                    .padding(.horizontal, 9)
                    .padding(.vertical, 5)
                    .background(Capsule().fill(skin.palette.surface))
            }
            content
        }
    }
}

struct GameRow: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let game: GameEntrySwift
    let details: String
    let compatibility: String
    let action: () -> Void

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        Button(action: action) {
            HStack(spacing: 14) {
                RoundedIcon(systemName: "play.fill", tint: skin.palette.accent)
                VStack(alignment: .leading, spacing: 4) {
                    Text(game.label)
                        .font(skin.rowFont)
                        .foregroundColor(skin.palette.ink)
                        .lineLimit(1)
                    Text(details)
                        .font(skin.subtitleFont)
                        .foregroundColor(skin.palette.secondary)
                        .lineLimit(1)
                    Text(compatibility)
                        .font(.caption2.weight(.semibold))
                        .foregroundColor(compatibility.hasPrefix("No") ? .orange : skin.palette.accent)
                        .lineLimit(1)
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.caption.bold())
                    .foregroundColor(RetroArchMenuPalette.driver("materialui").muted)
            }
            .padding(15)
            .background(skin.palette.surface)
            .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

struct CoreRow: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let core: CoreInfo

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        HStack(spacing: 14) {
            RoundedIcon(systemName: "cpu.fill", tint: RetroArchMenuPalette.violet)
            VStack(alignment: .leading, spacing: 4) {
                Text(core.displayName)
                    .font(skin.rowFont)
                    .foregroundColor(skin.palette.ink)
                    .lineLimit(1)
                Text([core.systemName, core.supportedExtensions.joined(separator: ", ")].filter { !$0.isEmpty }.joined(separator: " • "))
                    .font(skin.subtitleFont)
                    .foregroundColor(skin.palette.secondary)
                    .lineLimit(2)
            }
            Spacer(minLength: 0)
        }
        .padding(15)
        .background(skin.palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius, style: .continuous))
    }
}

struct LibraryStatCard: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let value: String
    let icon: String
    let tint: Color

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        HStack(spacing: 10) {
            RoundedIcon(systemName: icon, tint: tint)
            VStack(alignment: .leading, spacing: 2) {
                Text(value).font(.title3.bold()).foregroundColor(skin.palette.ink)
                Text(title).font(skin.subtitleFont.weight(.semibold)).foregroundColor(skin.palette.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
        .frame(maxWidth: .infinity)
        .background(skin.palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius, style: .continuous))
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
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let icon: String
    let title: String
    let message: String

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 28, weight: .semibold))
                .foregroundColor(RetroArchMenuPalette.driver("materialui").muted)
            Text(title)
                .font(.headline)
                .foregroundColor(skin.palette.ink)
            Text(message)
                .font(.subheadline)
                .multilineTextAlignment(.center)
                .foregroundColor(skin.palette.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(24)
        .background(skin.palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius == 0 ? 0 : RetroArchMenuMetrics.radius, style: .continuous))
    }
}
