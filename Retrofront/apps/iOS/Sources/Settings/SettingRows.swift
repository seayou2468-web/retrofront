import SwiftUI
import RetrofrontSwift

struct SettingsGroup<Content: View>: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    @ViewBuilder var content: Content

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        VStack(alignment: .leading, spacing: skin.rowSpacing) {
            Text(title)
                .font(skin.titleFont.weight(.bold))
                .foregroundColor(skin.palette.ink)
            VStack(spacing: 1) { content }
                .padding(6)
                .background(skin.palette.surface)
                .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius == 0 ? 0 : RetroArchMenuMetrics.radius, style: .continuous))
        }
    }
}

struct SettingPickerRow: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let subtitle: String
    let value: String
    let choices: [(label: String, value: String)]
    let onSelect: ((label: String, value: String)) -> Void

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        Menu {
            if choices.isEmpty {
                Text("No choices available")
            } else {
                ForEach(choices, id: \.value) { choice in
                    Button {
                        onSelect(choice)
                    } label: {
                        HStack {
                            Text(choice.label)
                            if choice.value == value || choice.label == value { Image(systemName: "checkmark") }
                        }
                    }
                }
            }
        } label: {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 3) {
                    Text(title).font(skin.rowFont.weight(.semibold)).foregroundColor(skin.palette.ink)
                    Text(subtitle).font(skin.subtitleFont).foregroundColor(skin.palette.secondary)
                }
                Spacer()
                Text(value)
                    .font(.caption.bold())
                    .foregroundColor(skin.palette.accent)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(Capsule().fill(skin.palette.accent.opacity(0.10)))
                Image(systemName: "chevron.down")
                    .font(.caption.bold())
                    .foregroundColor(skin.palette.secondary)
            }
            .padding(12)
            .background(skin.palette.elevated)
            .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}


struct SettingToggleRow: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let subtitle: String
    let isOn: Bool
    let onChange: (Bool) -> Void

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 3) {
                Text(title).font(skin.rowFont.weight(.semibold)).foregroundColor(skin.palette.ink)
                Text(subtitle).font(skin.subtitleFont).foregroundColor(skin.palette.secondary)
            }
            Spacer()
            Toggle("", isOn: Binding(
    get: { isOn },
    set: { newValue in
        onChange(newValue)
    }
))
                .labelsHidden()
                .toggleStyle(SwitchToggleStyle(tint: skin.palette.accent))
        }
        .padding(12)
        .background(skin.palette.elevated)
        .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius, style: .continuous))
    }
}

struct SettingInfoRow: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let value: String

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(skin.rowFont.weight(.semibold)).foregroundColor(skin.palette.ink)
            Text(value.isEmpty ? "Not set" : value)
                .font(.caption)
                .foregroundColor(skin.palette.secondary)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(skin.palette.elevated)
        .clipShape(RoundedRectangle(cornerRadius: skin.rowCornerRadius, style: .continuous))
    }
}
