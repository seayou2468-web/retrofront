import SwiftUI
import RetrofrontSwift

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
                        .background(Capsule().fill(RetroArchMenuPalette.driver("materialui").accent))
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
