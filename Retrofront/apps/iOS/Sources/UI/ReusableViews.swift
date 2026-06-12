import SwiftUI
import RetrofrontSwift

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
