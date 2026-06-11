// swift-tools-version: 5.9
import PackageDescription
import Foundation

let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path
let rustReleasePath = "\(packageRoot)/target/release"

let package = Package(
  name: "Retrofront",
  platforms: [
    .iOS(.v15),
    .macOS(.v13),
  ],
  products: [
    .library(name: "RetrofrontSwift", targets: ["RetrofrontSwift"]),
    .executable(name: "retrofront-linux", targets: ["RetrofrontLinux"]),
  ],
  targets: [
    .target(
      name: "CRetrofrontCore",
      path: "frontend/CRetrofrontCore",
      publicHeadersPath: "."
    ),
    .target(
      name: "RetrofrontSwift",
      dependencies: ["CRetrofrontCore"],
      path: "frontend/Sources/RetrofrontSwift",
      linkerSettings: [
        .unsafeFlags(["-L", rustReleasePath, "-Xlinker", "-rpath", "-Xlinker", rustReleasePath]),
        .linkedLibrary("retrofront_core"),
      ]
    ),
    .systemLibrary(
      name: "CGtk",
      path: "apps/linux/CGtk",
      pkgConfig: "gtk+-3.0",
      providers: [
        .apt(["libgtk-3-dev"]),
        .brew(["gtk+3"]),
      ]
    ),
    .executableTarget(
      name: "RetrofrontLinux",
      dependencies: ["RetrofrontSwift", "CGtk"],
      path: "apps/linux/Sources"
    ),
    .testTarget(
      name: "RetrofrontSwiftTests",
      dependencies: ["RetrofrontSwift"],
      path: "frontend/Tests/RetrofrontSwiftTests"
    ),
  ]
)
