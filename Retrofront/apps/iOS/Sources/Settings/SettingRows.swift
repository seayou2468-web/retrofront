import SwiftUI

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

struct SettingPickerRow: View {
    let title: String
    let subtitle: String
    let value: String
    let choices: [(label: String, value: String)]
    let onSelect: ((label: String, value: String)) -> Void

    var body: some View {
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
                Image(systemName: "chevron.down")
                    .font(.caption.bold())
                    .foregroundColor(OneUI.secondary)
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
            Toggle("", isOn: Binding(
    get: { isOn },
    set: { newValue in
        onChange(newValue)
    }
))
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
