import SwiftUI
import UIKit

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

