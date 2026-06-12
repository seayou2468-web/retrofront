import SwiftUI

enum RetroArchMenuMetrics {
    static let radius: CGFloat = 24
    static let compactRadius: CGFloat = 18
}

enum RetroArchMenuBackgrounds {
    static var ashChrome: some View {
        ZStack {
            Color(red: 0.055, green: 0.060, blue: 0.070)
            LinearGradient(colors: [Color.white.opacity(0.10), Color.black.opacity(0.35)], startPoint: .top, endPoint: .bottom)
            RadialGradient(colors: [Color(red: 0.62, green: 0.68, blue: 0.78).opacity(0.22), .clear], center: .topLeading, startRadius: 20, endRadius: 440)
            RadialGradient(colors: [Color(red: 0.22, green: 0.26, blue: 0.32).opacity(0.42), .clear], center: .bottomTrailing, startRadius: 60, endRadius: 380)
        }
    }

    static var aurora: some View {
        ZStack {
            RetroArchMenuPalette.driver("materialui").background
            RadialGradient(colors: [RetroArchMenuPalette.violet.opacity(0.36), .clear], center: .topLeading, startRadius: 20, endRadius: 420)
            RadialGradient(colors: [RetroArchMenuPalette.teal.opacity(0.28), .clear], center: .bottomTrailing, startRadius: 40, endRadius: 360)
        }
    }
}

struct RetroArchMenuPalette {
    let background: Color
    let surface: Color
    let elevated: Color
    let ink: Color
    let secondary: Color
    let accent: Color

    var muted: Color { secondary.opacity(0.70) }

    static let teal = Color(red: 0.130, green: 0.830, blue: 0.830)
    static let violet = Color(red: 0.620, green: 0.500, blue: 1.000)
    static let pink = Color(red: 0.930, green: 0.280, blue: 0.520)
    static let amber = Color(red: 0.950, green: 0.650, blue: 0.120)

    static func driver(_ ident: String) -> RetroArchMenuPalette {
        switch ident.lowercased() {
        case "ozone":
            return .init(background: Color(red: 0.07, green: 0.08, blue: 0.10), surface: Color(red: 0.10, green: 0.11, blue: 0.14), elevated: Color(red: 0.16, green: 0.18, blue: 0.22), ink: .white, secondary: Color(red: 0.70, green: 0.74, blue: 0.80), accent: Color(red: 0.28, green: 0.58, blue: 0.95))
        case "xmb":
            return .init(background: Color(red: 0.02, green: 0.06, blue: 0.16), surface: Color(red: 0.05, green: 0.11, blue: 0.24), elevated: Color(red: 0.09, green: 0.18, blue: 0.36), ink: .white, secondary: Color(red: 0.72, green: 0.82, blue: 0.96), accent: Color(red: 0.58, green: 0.78, blue: 1.00))
        case "rgui":
            return .init(background: .black, surface: Color(red: 0.02, green: 0.08, blue: 0.08), elevated: Color(red: 0.00, green: 0.18, blue: 0.16), ink: Color(red: 0.72, green: 1.0, blue: 0.82), secondary: Color(red: 0.42, green: 0.82, blue: 0.62), accent: Color(red: 0.95, green: 0.90, blue: 0.45))
        default:
            return .init(background: Color(red: 0.05, green: 0.05, blue: 0.06), surface: Color(red: 0.10, green: 0.10, blue: 0.12), elevated: Color(red: 0.16, green: 0.16, blue: 0.18), ink: .white, secondary: Color(red: 0.74, green: 0.74, blue: 0.78), accent: Color(red: 0.00, green: 0.74, blue: 0.83))
        }
    }
}

struct AppScreen<Content: View>: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let title: String
    let subtitle: String
    @ViewBuilder var content: Content

    var body: some View {
        let skin = RetroArchMenuSkin.current(runtime: runtime)
        NavigationStack {
            ZStack(alignment: .topLeading) {
                RetroArchMenuBackground(skin: skin).ignoresSafeArea()
                if skin.showsSidebarRail { ozoneRail(skin) }
                if skin.showsXmbRibbon { xmbRibbon(skin) }

                ScrollView {
                    VStack(alignment: .leading, spacing: skin.layout == .rgui ? 8 : 18) {
                        header(skin)
                        content
                    }
                    .padding(.horizontal, max(18, skin.horizontalPadding))
                    .padding(.leading, skin.showsSidebarRail ? 82 : 0)
                    .padding(.top, skin.showsMaterialBar ? 56 : 16)
                    .padding(.bottom, 28)
                }

                if skin.showsMaterialBar { materialBar(skin) }
            }
            .toolbar(.hidden, for: .navigationBar)
        }
    }

    private func header(_ skin: RetroArchMenuSkin) -> some View {
        VStack(alignment: .leading, spacing: skin.layout == .material ? 2 : 4) {
            Text(title)
                .font(skin.titleFont)
                .foregroundColor(skin.palette.ink)
                .textCase(skin.layout == .rgui ? .uppercase : nil)
            Text(subtitle)
                .font(skin.subtitleFont.weight(.medium))
                .foregroundColor(skin.palette.secondary)
        }
        .padding(.top, skin.layout == .material ? 10 : 0)
        .padding(.bottom, skin.layout == .material ? 10 : 0)
    }

    private func materialBar(_ skin: RetroArchMenuSkin) -> some View {
        HStack(spacing: 12) {
            Image(systemName: "line.3.horizontal")
            Text(title).font(.headline)
            Spacer()
            Text(skin.displayName).font(.caption.weight(.semibold))
        }
        .foregroundColor(skin.palette.ink)
        .padding(.horizontal, 16)
        .frame(height: 56)
        .background(skin.palette.surface.opacity(0.98))
    }

    private func ozoneRail(_ skin: RetroArchMenuSkin) -> some View {
        VStack(spacing: 22) {
            RetroArchMenuAssetImage(url: skin.assets.iconURL(named: "history"), systemName: "house.fill")
            RetroArchMenuAssetImage(url: skin.assets.iconURL(named: "load-content"), systemName: "rectangle.stack.fill")
            RetroArchMenuAssetImage(url: skin.assets.iconURL(named: "core"), systemName: "cpu.fill")
            RetroArchMenuAssetImage(url: skin.assets.iconURL(named: "settings"), systemName: "gearshape.fill")
            Spacer()
        }
        .font(.title3)
        .foregroundColor(skin.palette.secondary)
        .frame(width: 88)
        .padding(.top, 42)
    }

    private func xmbRibbon(_ skin: RetroArchMenuSkin) -> some View {
        HStack(spacing: 30) {
            ForEach([(systemName: "house.fill", assetName: "history"), (systemName: "rectangle.stack.fill", assetName: "load-content"), (systemName: "cpu.fill", assetName: "core"), (systemName: "gearshape.fill", assetName: "settings")], id: \.systemName) { icon in
                RetroArchMenuAssetImage(url: skin.assets.iconURL(named: icon.assetName), systemName: icon.systemName)
                    .frame(width: 30, height: 30)
                    .foregroundColor(skin.palette.ink.opacity(0.82))
            }
        }
        .padding(.top, 72)
        .padding(.leading, 26)
    }
}
