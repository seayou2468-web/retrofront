import Foundation
import SwiftUI
import UIKit
import RetrofrontSwift

struct RetroArchMenuAssets {
    let root: URL
    let driverRoot: URL
    let iconDirectory: URL?
    let fontCandidates: [URL]
    let backgroundCandidates: [URL]

    static func resolve(driver: String, assetsRootPath: String, native: MenuResolvedAssets? = nil) -> RetroArchMenuAssets {
        let root = URL(fileURLWithPath: native?.rootDirectory.isEmpty == false ? native!.rootDirectory : (assetsRootPath.isEmpty ? "assets" : assetsRootPath), isDirectory: true)
        let normalized = driver.lowercased()
        let directoryName = normalized
        let driverRoot = URL(fileURLWithPath: native?.driverDirectory.isEmpty == false ? native!.driverDirectory : root.appendingPathComponent(directoryName, isDirectory: true).path, isDirectory: true)
        let iconDirectory: URL?
        if let native, !native.iconDirectory.isEmpty {
            iconDirectory = URL(fileURLWithPath: native.iconDirectory, isDirectory: true)
        } else {
            switch normalized {
            case "ozone": iconDirectory = driverRoot.appendingPathComponent("png", isDirectory: true)
            case "xmb": iconDirectory = driverRoot.appendingPathComponent("png", isDirectory: true)
            case "rgui": iconDirectory = driverRoot.appendingPathComponent("png", isDirectory: true)
            case "materialui": iconDirectory = driverRoot
            default: iconDirectory = nil
            }
        }
        var fontCandidates = [
            driverRoot.appendingPathComponent("font.ttf"),
            driverRoot.appendingPathComponent("regular.ttf"),
            driverRoot.appendingPathComponent("bold.ttf"),
            root.appendingPathComponent("xmb/monochrome/font.ttf"),
            root.appendingPathComponent("ozone/regular.ttf")
        ]
        if let native, !native.fontPath.isEmpty { fontCandidates.insert(URL(fileURLWithPath: native.fontPath), at: 0) }
        var backgroundCandidates = [
            driverRoot.appendingPathComponent("bg.png"),
            driverRoot.appendingPathComponent("wallpaper.png"),
            driverRoot.appendingPathComponent("png/retroarch.png"),
            driverRoot.appendingPathComponent("monochrome/png/retroarch.png"),
            root.appendingPathComponent("xmb/monochrome/png/retroarch.png"),
            root.appendingPathComponent("ozone/png/retroarch.png")
        ]
        if let native, !native.backgroundPath.isEmpty { backgroundCandidates.insert(URL(fileURLWithPath: native.backgroundPath), at: 0) }
        return RetroArchMenuAssets(root: root, driverRoot: driverRoot, iconDirectory: iconDirectory, fontCandidates: fontCandidates, backgroundCandidates: backgroundCandidates)
    }

    var hasDriverAssets: Bool {
        FileManager.default.fileExists(atPath: driverRoot.path)
    }

    func firstExisting(_ urls: [URL]) -> URL? {
        urls.first { FileManager.default.fileExists(atPath: $0.path) }
    }

    func iconURL(named name: String) -> URL? {
        guard let iconDirectory else { return nil }
        let candidates = [
            iconDirectory.appendingPathComponent("\(name).png"),
            iconDirectory.appendingPathComponent("\(name).svg"),
            driverRoot.appendingPathComponent("png/\(name).png"),
            driverRoot.appendingPathComponent("monochrome/png/\(name).png"),
            driverRoot.appendingPathComponent("\(name).png")
        ]
        return firstExisting(candidates)
    }

    var backgroundURL: URL? { firstExisting(backgroundCandidates) }
}

struct RetroArchMenuAssetImage: View {
    let url: URL?
    let systemName: String
    let renderingMode: Image.TemplateRenderingMode

    init(url: URL?, systemName: String, renderingMode: Image.TemplateRenderingMode = .template) {
        self.url = url
        self.systemName = systemName
        self.renderingMode = renderingMode
    }

    var body: some View {
        Group {
            if let url,
               url.pathExtension.lowercased() == "png",
               let image = UIImage(contentsOfFile: url.path) {
                Image(uiImage: image).renderingMode(renderingMode).resizable().scaledToFit()
            } else {
                Color.clear
            }
        }
        .accessibilityHidden(true)
    }
}

struct RetroArchMenuSkin {
    enum Layout {
        case material
        case ozone
        case xmb
        case rgui
    }

    let driver: String
    let displayName: String
    let layout: Layout
    let palette: RetroArchMenuPalette
    let assets: RetroArchMenuAssets
    let rowCornerRadius: CGFloat
    let rowSpacing: CGFloat
    let horizontalPadding: CGFloat
    let titleFont: Font
    let rowFont: Font
    let subtitleFont: Font
    let usesMonospacedRows: Bool
    let showsSidebarRail: Bool
    let showsXmbRibbon: Bool
    let showsMaterialBar: Bool

    @MainActor
    static func current(runtime: EmulatorRuntimeModel) -> RetroArchMenuSkin {
        let rawDriver = runtime.settingValue("menu_driver")
        let driver = rawDriver.isEmpty ? "materialui" : rawDriver.lowercased()
        let assetsRoot = runtime.settingValue("menu_assets_directory").isEmpty ? runtime.settingValue("assets_directory") : runtime.settingValue("menu_assets_directory")
        return RetroArchMenuSkin(driver: driver, assetsRootPath: assetsRoot, nativeAssets: runtime.frontend?.menuResolvedAssets())
    }

    init(driver: String, assetsRootPath: String, nativeAssets: MenuResolvedAssets? = nil) {
        let normalized = driver.lowercased()
        self.driver = normalized
        self.palette = RetroArchMenuPalette.driver(normalized)
        self.assets = .resolve(driver: normalized, assetsRootPath: assetsRootPath, native: nativeAssets)
        switch normalized {
        case "ozone":
            displayName = "Ozone"
            layout = .ozone
            rowCornerRadius = 6
            rowSpacing = 6
            horizontalPadding = 22
            titleFont = .system(size: 28, weight: .semibold)
            rowFont = .system(size: 16, weight: .medium)
            subtitleFont = .caption
            usesMonospacedRows = false
            showsSidebarRail = true
            showsXmbRibbon = false
            showsMaterialBar = false
        case "xmb":
            displayName = "XMB"
            layout = .xmb
            rowCornerRadius = 2
            rowSpacing = 14
            horizontalPadding = 26
            titleFont = .system(size: 26, weight: .light)
            rowFont = .system(size: 18, weight: .regular)
            subtitleFont = .caption
            usesMonospacedRows = false
            showsSidebarRail = false
            showsXmbRibbon = true
            showsMaterialBar = false
        case "rgui":
            displayName = "RGUI"
            layout = .rgui
            rowCornerRadius = 0
            rowSpacing = 0
            horizontalPadding = 12
            titleFont = .system(size: 20, weight: .bold, design: .monospaced)
            rowFont = .system(size: 15, weight: .regular, design: .monospaced)
            subtitleFont = .system(size: 11, weight: .regular, design: .monospaced)
            usesMonospacedRows = true
            showsSidebarRail = false
            showsXmbRibbon = false
            showsMaterialBar = false
        default:
            displayName = "Material UI"
            layout = .material
            rowCornerRadius = 2
            rowSpacing = 1
            horizontalPadding = 0
            titleFont = .system(size: 22, weight: .medium)
            rowFont = .system(size: 16, weight: .regular)
            subtitleFont = .caption
            usesMonospacedRows = false
            showsSidebarRail = false
            showsXmbRibbon = false
            showsMaterialBar = true
        }
    }

    func assetIconName(for actionId: UInt32) -> String {
        switch actionId {
        case 1, 20, 21, 222: return "core"
        case 2, 36, 37: return "load-content"
        case 3: return "network"
        case 4, 210...221, 260...274: return "settings"
        case 8: return "menu"
        case 9: return "restart"
        case 10: return "resume"
        case 12, 26: return "close"
        case 13: return "shader-options"
        case 14, 27, 28, 29, 30, 38, 39: return "savestate"
        case 15: return "take-screenshot"
        case 16: return "add-favorite"
        case 17: return "cheat-options"
        case 19, 25, 213: return "input"
        case 22, 211: return "video"
        case 23, 212: return "audio"
        default: return "default"
        }
    }

    func iconURL(for actionId: UInt32) -> URL? {
        assets.iconURL(named: assetIconName(for: actionId))
    }

    func iconName(for actionId: UInt32) -> String {
        switch actionId {
        case 1, 20, 21, 222: return "cpu.fill"
        case 2, 36, 37: return "rectangle.stack.fill"
        case 3: return "icloud.and.arrow.down.fill"
        case 4, 210...221, 260...274: return "gearshape.fill"
        case 8: return "line.3.horizontal.circle.fill"
        case 9: return "restart.circle.fill"
        case 10: return "play.fill"
        case 12, 26: return "xmark.circle.fill"
        case 13: return "sparkles"
        case 14, 27, 28, 29, 30, 38, 39: return "sdcard.fill"
        case 15: return "camera.fill"
        case 16: return "heart.fill"
        case 17: return "wand.and.stars"
        case 19, 25, 213: return "gamecontroller.fill"
        case 22, 211: return "display"
        case 23, 212: return "speaker.wave.2.fill"
        default: return "circle.grid.2x2.fill"
        }
    }
}

struct RetroArchMenuBackground: View {
    let skin: RetroArchMenuSkin

    var body: some View {
        ZStack(alignment: .topLeading) {
            skin.palette.background
            if let backgroundURL = skin.assets.backgroundURL,
               backgroundURL.pathExtension.lowercased() == "png",
               let image = UIImage(contentsOfFile: backgroundURL.path) {
                Image(uiImage: image)
                    .resizable()
                    .scaledToFill()
                    .opacity(skin.layout == .xmb ? 0.36 : 0.18)
            }
            switch skin.layout {
            case .material:
                LinearGradient(colors: [skin.palette.surface, skin.palette.background], startPoint: .top, endPoint: .bottom)
            case .ozone:
                LinearGradient(colors: [skin.palette.background, skin.palette.surface.opacity(0.95)], startPoint: .leading, endPoint: .trailing)
                Rectangle().fill(skin.palette.elevated.opacity(0.88)).frame(width: 88)
            case .xmb:
                RadialGradient(colors: [skin.palette.accent.opacity(0.45), .clear], center: .topLeading, startRadius: 10, endRadius: 520)
                LinearGradient(colors: [.white.opacity(0.10), .clear, .black.opacity(0.35)], startPoint: .top, endPoint: .bottom)
            case .rgui:
                skin.palette.background
                Rectangle().strokeBorder(skin.palette.accent.opacity(0.7), lineWidth: 2).padding(6)
            }
        }
    }
}
