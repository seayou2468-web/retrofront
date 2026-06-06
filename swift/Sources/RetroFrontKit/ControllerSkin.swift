import Foundation

public enum RetroJoypadID: UInt32, CaseIterable, Sendable {
    case b = 0, y = 1, select = 2, start = 3, up = 4, down = 5, left = 6, right = 7
    case a = 8, x = 9, l = 10, r = 11, l2 = 12, r2 = 13, l3 = 14, r3 = 15
}

public enum ControllerHitShape: Equatable, Sendable {
    case rect(x: Double, y: Double, width: Double, height: Double)
    case circle(x: Double, y: Double, radius: Double)

    public func contains(x: Double, y: Double) -> Bool {
        switch self {
        case let .rect(bx, by, width, height):
            return x >= bx && x <= bx + width && y >= by && y <= by + height
        case let .circle(bx, by, radius):
            let dx = x - bx
            let dy = y - by
            return dx * dx + dy * dy <= radius * radius
        }
    }
}

public struct ControllerSkinButton: Equatable, Sendable {
    public var name: String
    public var port: UInt32
    public var joypadID: RetroJoypadID
    public var shape: ControllerHitShape

    public init(name: String, port: UInt32 = 0, joypadID: RetroJoypadID, shape: ControllerHitShape) {
        self.name = name
        self.port = port
        self.joypadID = joypadID
        self.shape = shape
    }
}

public struct ControllerSkin: Equatable, Sendable {
    public var name: String
    public var image: String?
    public var buttons: [ControllerSkinButton]

    public init(name: String = "Default Touch Pad", image: String? = nil, buttons: [ControllerSkinButton] = ControllerSkin.standardGamepadButtons) {
        self.name = name
        self.image = image
        self.buttons = buttons
    }

    public func hitTest(x: Double, y: Double) -> [ControllerSkinButton] {
        buttons.filter { $0.shape.contains(x: x, y: y) }
    }

    public static let standardGamepadButtons: [ControllerSkinButton] = [
        .init(name: "up", joypadID: .up, shape: .rect(x: 0.12, y: 0.62, width: 0.10, height: 0.10)),
        .init(name: "down", joypadID: .down, shape: .rect(x: 0.12, y: 0.82, width: 0.10, height: 0.10)),
        .init(name: "left", joypadID: .left, shape: .rect(x: 0.02, y: 0.72, width: 0.10, height: 0.10)),
        .init(name: "right", joypadID: .right, shape: .rect(x: 0.22, y: 0.72, width: 0.10, height: 0.10)),
        .init(name: "a", joypadID: .a, shape: .circle(x: 0.86, y: 0.72, radius: 0.055)),
        .init(name: "b", joypadID: .b, shape: .circle(x: 0.74, y: 0.82, radius: 0.055)),
        .init(name: "start", joypadID: .start, shape: .rect(x: 0.53, y: 0.84, width: 0.10, height: 0.05)),
        .init(name: "select", joypadID: .select, shape: .rect(x: 0.37, y: 0.84, width: 0.10, height: 0.05)),
    ]
}
