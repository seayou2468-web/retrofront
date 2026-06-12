import SwiftUI

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

