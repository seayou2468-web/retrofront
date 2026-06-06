// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "RetroFront",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "RetroFrontKit", targets: ["RetroFrontKit"]),
        .executable(name: "retrofront-swift", targets: ["RetroFrontApp"]),
    ],
    targets: [
        .target(name: "RetroFrontKit"),
        .executableTarget(name: "RetroFrontApp", dependencies: ["RetroFrontKit"]),
        .testTarget(name: "RetroFrontKitTests", dependencies: ["RetroFrontKit"]),
    ]
)
