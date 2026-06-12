// swift-tools-version: 6.1
import PackageDescription
import Foundation

let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path
let rustReleasePath = "\(packageRoot)/target/release"
let includeLinuxGUI = ProcessInfo.processInfo.environment["RETROFRONT_DISABLE_LINUX_GUI"] != "1"

var dependencies: [Package.Dependency] = []
if includeLinuxGUI {
  dependencies.append(.package(url: "https://github.com/makoni/swift-adwaita.git", .upToNextMinor(from: "1.1.0")))
}

var targets: [Target] = [
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
  .testTarget(
    name: "RetrofrontSwiftTests",
    dependencies: ["RetrofrontSwift"],
    path: "frontend/Tests/RetrofrontSwiftTests"
  ),
]

if includeLinuxGUI {
  targets.append(
    .executableTarget(
      name: "RetrofrontLinux",
      dependencies: ["RetrofrontSwift", .product(name: "Adwaita", package: "swift-adwaita")],
      path: "apps/linux/Sources"
    )
  )
}

let products: [Product] = includeLinuxGUI
  ? [
      .library(name: "RetrofrontSwift", targets: ["RetrofrontSwift"]),
      .executable(name: "retrofront-linux", targets: ["RetrofrontLinux"]),
    ]
  : [
      .library(name: "RetrofrontSwift", targets: ["RetrofrontSwift"]),
    ]

let package = Package(
  name: "Retrofront",
  platforms: [
    .iOS(.v18),
    .macOS(.v13),
  ],
  products: products,
  dependencies: dependencies,
  targets: targets
)
