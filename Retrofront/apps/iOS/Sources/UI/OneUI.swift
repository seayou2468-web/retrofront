import SwiftUI

enum OneUI {
    static let background = Color(red: 0.030, green: 0.036, blue: 0.055)
    static let surface = Color(red: 0.075, green: 0.088, blue: 0.125)
    static let elevated = Color(red: 0.105, green: 0.122, blue: 0.170)
    static let ink = Color(red: 0.930, green: 0.950, blue: 0.990)
    static let secondary = Color(red: 0.650, green: 0.700, blue: 0.790)
    static let muted = Color(red: 0.430, green: 0.490, blue: 0.610)
    static let accent = Color(red: 0.250, green: 0.600, blue: 1.000)
    static let teal = Color(red: 0.130, green: 0.830, blue: 0.830)
    static let violet = Color(red: 0.620, green: 0.500, blue: 1.000)
    static let pink = Color(red: 0.930, green: 0.280, blue: 0.520)
    static let amber = Color(red: 0.950, green: 0.650, blue: 0.120)
    static let radius: CGFloat = 24
    static let compactRadius: CGFloat = 18

    static var ashChromeBackground: some View {
        ZStack {
            Color(red: 0.055, green: 0.060, blue: 0.070)
            LinearGradient(colors: [Color.white.opacity(0.10), Color.black.opacity(0.35)], startPoint: .top, endPoint: .bottom)
            RadialGradient(colors: [Color(red: 0.62, green: 0.68, blue: 0.78).opacity(0.22), .clear], center: .topLeading, startRadius: 20, endRadius: 440)
            RadialGradient(colors: [Color(red: 0.22, green: 0.26, blue: 0.32).opacity(0.42), .clear], center: .bottomTrailing, startRadius: 60, endRadius: 380)
        }
    }

    static var auroraBackground: some View {
        ZStack {
            background
            RadialGradient(colors: [violet.opacity(0.36), .clear], center: .topLeading, startRadius: 20, endRadius: 420)
            RadialGradient(colors: [teal.opacity(0.28), .clear], center: .bottomTrailing, startRadius: 40, endRadius: 360)
        }
    }
}

struct AppScreen<Content: View>: View {
    let title: String
    let subtitle: String
    @ViewBuilder var content: Content

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(title)
                            .font(.system(size: 34, weight: .bold, design: .default))
                            .foregroundColor(OneUI.ink)
                        Text(subtitle)
                            .font(.subheadline.weight(.medium))
                            .foregroundColor(OneUI.secondary)
                    }
                    .padding(.top, 16)

                    content
                }
                .padding(.horizontal, 18)
                .padding(.bottom, 28)
            }
            .background(OneUI.auroraBackground.ignoresSafeArea())
            .toolbar(.hidden, for: .navigationBar)
        }
    }
}

