import SwiftUI
import UniformTypeIdentifiers
import UIKit
import RetrofrontSwift

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
                        .aspectRatio(runtime.aspectRatio, contentMode: .fit)
                        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
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

