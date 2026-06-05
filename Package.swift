// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "RetroFront",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "RetroFrontCore", targets: ["RetroFrontCore"]),
        .library(name: "RetroFrontiOS", targets: ["RetroFrontiOS"]),
        .library(name: "CLibretroHost", targets: ["CLibretroHost"])
    ],
    targets: [
        .target(
            name: "CLibretroHost",
            publicHeadersPath: "include",
            cSettings: [
                .headerSearchPath("../../Externals/libretro-common/include")
            ],
            linkerSettings: [
                .linkedLibrary("dl", .when(platforms: [.linux]))
            ]
        ),
        .target(
            name: "RetroFrontCore",
            dependencies: ["CLibretroHost"]
        ),
        .target(
            name: "RetroFrontiOS",
            dependencies: ["RetroFrontCore"]
        ),
        .testTarget(
            name: "RetroFrontCoreTests",
            dependencies: ["RetroFrontCore"]
        )
    ]
)
