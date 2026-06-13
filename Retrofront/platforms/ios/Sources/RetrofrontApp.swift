import UIKit

@main
final class RetrofrontAppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        let controller = RetrofrontHostViewController()
        let window = UIWindow(frame: UIScreen.main.bounds)
        window.rootViewController = controller
        window.makeKeyAndVisible()
        self.window = window
        controller.bootRetrofrontMenuRuntime()
        return true
    }

    func applicationWillTerminate(_ application: UIApplication) {
        retrofront_runtime_shutdown()
    }
}

final class RetrofrontHostViewController: UIViewController {
    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black
    }

    func bootRetrofrontMenuRuntime() {
        let fm = FileManager.default
        guard let documents = fm.urls(for: .documentDirectory, in: .userDomainMask).first else {
            return
        }
        let dataRoot = documents.appendingPathComponent("RetroArch", isDirectory: true)
        try? fm.createDirectory(at: dataRoot, withIntermediateDirectories: true)

        guard dataRoot.path.withCString({ retrofront_runtime_init($0) }) else {
            return
        }

        if let zip = Bundle.main.url(forResource: "assets", withExtension: "zip") {
            _ = zip.path.withCString { retrofront_resources_unpack($0) }
        }
        _ = retrofront_assets_load_defaults()
        _ = retrofront_menu_bootstrap()
        _ = retrofront_menu_contract_complete()
        _ = retrofront_menu_draw()
    }
}
