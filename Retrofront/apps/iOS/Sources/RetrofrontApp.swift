import UIKit

@_silgen_name("retrofront_slint_ios_main")
func retrofront_slint_ios_main() -> Int32

final class RetrofrontAppDelegate: UIResponder, UIApplicationDelegate {
  var window: UIWindow?

  func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    DispatchQueue.main.async {
      let exitCode = retrofront_slint_ios_main()
      if exitCode != 0 {
        NSLog("Retrofront Slint runtime exited with code %d", exitCode)
      }
    }
    return true
  }
}

@main
struct RetrofrontApp {
  static func main() {
    UIApplicationMain(
      CommandLine.argc,
      CommandLine.unsafeArgv,
      nil,
      NSStringFromClass(RetrofrontAppDelegate.self)
    )
  }
}
