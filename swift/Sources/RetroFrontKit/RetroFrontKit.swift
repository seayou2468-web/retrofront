import Foundation

public enum RetroFrontPlatform: String, Sendable {
    case linux
    case iOSDevice
}

public struct SkinTheme: Equatable, Sendable {
    public var name: String
    public var background: [Double]
    public var foreground: [Double]
    public var accent: [Double]

    public init(name: String = "RetroFront XMB", background: [Double] = [0.03, 0.04, 0.08, 1.0], foreground: [Double] = [0.92, 0.94, 1.0, 1.0], accent: [Double] = [0.15, 0.55, 1.0, 1.0]) {
        self.name = name
        self.background = background
        self.foreground = foreground
        self.accent = accent
    }
}

public enum MenuItem: String, CaseIterable, Sendable {
    case loadCore = "Load Core"
    case loadContent = "Load Content"
    case quickMenu = "Quick Menu"
    case settings = "Settings"
    case history = "History"
    case quit = "Quit"
}

public struct MenuModel: Equatable, Sendable {
    public var theme: SkinTheme
    public private(set) var items: [MenuItem]
    public private(set) var selectedIndex: Int

    public init(theme: SkinTheme = SkinTheme()) {
        self.theme = theme
        self.items = MenuItem.allCases
        self.selectedIndex = 0
    }

    public mutating func moveSelection(by delta: Int) {
        guard !items.isEmpty else { return }
        selectedIndex = (selectedIndex + delta) % items.count
        if selectedIndex < 0 { selectedIndex += items.count }
    }

    public var selectedItem: MenuItem { items[selectedIndex] }
}

public struct FrontendConfiguration: Equatable, Sendable {
    public var systemDirectory: String
    public var saveDirectory: String
    public var stateDirectory: String
    public var menuDriver: String

    public init(systemDirectory: String = "system", saveDirectory: String = "saves", stateDirectory: String = "states", menuDriver: String = "xmb") {
        self.systemDirectory = systemDirectory
        self.saveDirectory = saveDirectory
        self.stateDirectory = stateDirectory
        self.menuDriver = menuDriver
    }
}

public final class RetroFrontRuntime: @unchecked Sendable {
    public let platform: RetroFrontPlatform
    public private(set) var configuration: FrontendConfiguration
    public private(set) var menu: MenuModel
    public private(set) var controllerSkin: ControllerSkin

    public init(platform: RetroFrontPlatform, configuration: FrontendConfiguration = FrontendConfiguration(), menu: MenuModel = MenuModel(), controllerSkin: ControllerSkin = ControllerSkin()) {
        self.platform = platform
        self.configuration = configuration
        self.menu = menu
        self.controllerSkin = controllerSkin
    }

    public func validateRuntimePolicy() throws {
        #if targetEnvironment(simulator)
        if platform == .iOSDevice { throw RuntimePolicyError.iOSSimulatorUnsupported }
        #endif
    }
}

public enum RuntimePolicyError: Error, Equatable, CustomStringConvertible {
    case iOSSimulatorUnsupported

    public var description: String {
        switch self {
        case .iOSSimulatorUnsupported:
            return "RetroFront iOS builds are device-only because libretro cores need the real device ABI and entitlements."
        }
    }
}
