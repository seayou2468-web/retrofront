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
    private let menuView = RetrofrontMenuSurfaceView()

    override func loadView() {
        view = menuView
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        _ = retrofront_renderer_resize(UInt32(view.bounds.width), UInt32(view.bounds.height))
        refreshMenuSurface()
    }

    func bootRetrofrontMenuRuntime() {
        let fm = FileManager.default
        guard let documents = fm.urls(for: .documentDirectory, in: .userDomainMask).first else {
            return
        }
        let dataRoot = documents.appendingPathComponent("RetroArch", isDirectory: true)
        try? fm.createDirectory(at: dataRoot, withIntermediateDirectories: true)

        guard dataRoot.path.withCString({ retrofront_runtime_init($0) }) else {
            menuView.showBootError("Rust runtime init failed")
            return
        }

        if let zip = Bundle.main.url(forResource: "assets", withExtension: "zip") {
            _ = zip.path.withCString { retrofront_resources_unpack($0) }
        }
        _ = retrofront_assets_load_defaults()
        _ = retrofront_menu_bootstrap()
        _ = retrofront_menu_contract_complete()
        refreshMenuSurface()
    }

    fileprivate func performMenuAction(_ action: UInt32) {
        _ = retrofront_menu_action(action)
        _ = retrofront_core_launch_pending()
        refreshMenuSurface()
    }

    fileprivate func refreshMenuSurface() {
        _ = retrofront_menu_draw()
        menuView.reloadFromRuntime()
    }
}

private struct SurfaceEntry {
    let index: Int
    let label: String
    let sublabel: String
    let value: String
    let checked: Bool
}

private struct SurfaceAsset {
    let kind: UInt32
    let path: String
    let driver: String
}

final class RetrofrontMenuSurfaceView: UIView {
    private var title = "Retrofront"
    private var driver = "materialui"
    private var selectedIndex = 0
    private var entries: [SurfaceEntry] = []
    private var assets: [SurfaceAsset] = []
    private var menuFont: UIFont?
    private var menuIcon: UIImage?

    override init(frame: CGRect) {
        super.init(frame: frame)
        isMultipleTouchEnabled = true
        backgroundColor = .black
        addGestureRecognizer(UISwipeGestureRecognizer(target: self, action: #selector(swipeUp(_:))).configured(.up))
        addGestureRecognizer(UISwipeGestureRecognizer(target: self, action: #selector(swipeDown(_:))).configured(.down))
        addGestureRecognizer(UISwipeGestureRecognizer(target: self, action: #selector(swipeLeft(_:))).configured(.left))
        addGestureRecognizer(UISwipeGestureRecognizer(target: self, action: #selector(swipeRight(_:))).configured(.right))
        addGestureRecognizer(UITapGestureRecognizer(target: self, action: #selector(tap(_:))))
        let longPress = UILongPressGestureRecognizer(target: self, action: #selector(cancel(_:)))
        longPress.minimumPressDuration = 0.45
        addGestureRecognizer(longPress)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) is not supported")
    }

    func reloadFromRuntime() {
        title = readString { retrofront_menu_title($0, $1) }
        driver = readString { retrofront_menu_driver($0, $1) }
        selectedIndex = Int(retrofront_menu_selected_index())
        let count = Int(retrofront_menu_entry_count())
        assets = loadSurfaceAssets(for: driver)
        menuFont = assets.lazy.filter { $0.kind == 1 }.compactMap { UIFont(contentsOfFile: $0.path) }.first
        menuIcon = assets.lazy.filter { $0.kind == 2 }.compactMap { UIImage(contentsOfFile: $0.path) }.first
        entries = (0..<count).map { index in
            var raw = retrofront_menu_entry_t()
            _ = retrofront_menu_entry(index, &raw)
            return SurfaceEntry(
                index: index,
                label: withUnsafePointer(to: raw.label) { pointer in
                    pointer.withMemoryRebound(to: CChar.self, capacity: 1024) { String(cString: $0) }
                },
                sublabel: withUnsafePointer(to: raw.sublabel) { pointer in
                    pointer.withMemoryRebound(to: CChar.self, capacity: 1024) { String(cString: $0) }
                },
                value: withUnsafePointer(to: raw.value) { pointer in
                    pointer.withMemoryRebound(to: CChar.self, capacity: 1024) { String(cString: $0) }
                },
                checked: raw.checked
            )
        }
        setNeedsDisplay()
    }

    override func draw(_ rect: CGRect) {
        guard let context = UIGraphicsGetCurrentContext() else { return }
        let palette = MenuPalette(driver: driver)
        context.setFillColor(palette.background.cgColor)
        context.fill(rect)

        if let menuIcon {
            menuIcon.draw(in: CGRect(x: 24, y: safeAreaInsets.top + 20, width: 36, height: 36))
        }
        let titleX: CGFloat = menuIcon == nil ? 24 : 72
        drawText(title, in: CGRect(x: titleX, y: safeAreaInsets.top + 20, width: rect.width - titleX - 24, height: 42), font: menuFont ?? .boldSystemFont(ofSize: 32), color: palette.title)
        drawText("\(driver.uppercased()) · assets: \(assets.count)", in: CGRect(x: 24, y: safeAreaInsets.top + 64, width: rect.width - 48, height: 24), font: .systemFont(ofSize: 13), color: palette.subtext)

        let startY = safeAreaInsets.top + 104
        let rowHeight = CGFloat(max(retrofront_menu_driver_row_height(), UInt32(driver == "rgui" ? 34 : 58)))
        for (visibleIndex, entry) in entries.enumerated() {
            let y = startY + CGFloat(visibleIndex) * rowHeight
            guard y < rect.maxY - safeAreaInsets.bottom - 20 else { break }
            let sidebar = CGFloat(retrofront_menu_driver_sidebar_width())
            let rowX = driver == "ozone" && sidebar > 0 ? min(sidebar * 0.25, 96) : 16
            let row = CGRect(x: rowX, y: y, width: rect.width - rowX - 16, height: rowHeight - 6)
            if entry.index == selectedIndex {
                context.setFillColor(palette.selection.cgColor)
                context.fill(row)
            }
            let prefix = entry.checked ? "✓ " : ""
            let iconSize = CGFloat(min(max(retrofront_menu_driver_icon_size(), UInt32(8)), UInt32(48)))
            if let menuIcon, driver != "rgui" {
                menuIcon.draw(in: CGRect(x: row.minX + 8, y: row.midY - iconSize / 2, width: iconSize, height: iconSize))
            }
            let textInset = driver == "rgui" || menuIcon == nil ? 16 : iconSize + 20
            drawText(prefix + entry.label, in: row.insetBy(dx: textInset, dy: 6), font: menuFont ?? .systemFont(ofSize: driver == "rgui" ? 17 : 20, weight: entry.index == selectedIndex ? .bold : .regular), color: palette.text)
            if !entry.sublabel.isEmpty {
                drawText(entry.sublabel, in: CGRect(x: row.minX + CGFloat(textInset), y: row.minY + 30, width: row.width - CGFloat(textInset) - 16, height: 20), font: .systemFont(ofSize: 13), color: palette.subtext)
            }
            if !entry.value.isEmpty {
                drawText(entry.value, in: CGRect(x: row.maxX - 150, y: row.minY + 9, width: 132, height: 22), font: .systemFont(ofSize: 13), color: palette.subtext, alignment: .right)
            }
        }
        drawControls(in: rect, palette: palette)
    }


    private func loadSurfaceAssets(for driver: String) -> [SurfaceAsset] {
        let count = Int(retrofront_menu_asset_count())
        return (0..<count).compactMap { index in
            let path = readString { retrofront_menu_asset_path(index, $0, $1) }
            guard !path.isEmpty else { return nil }
            let assetDriver = readString { retrofront_menu_asset_driver(index, $0, $1) }
            guard assetDriver.isEmpty || assetDriver == driver else { return nil }
            return SurfaceAsset(kind: retrofront_menu_asset_kind(index), path: path, driver: assetDriver)
        }
    }

    func showBootError(_ message: String) {
        title = message
        entries = []
        setNeedsDisplay()
    }

    private func drawControls(in rect: CGRect, palette: MenuPalette) {
        let labels = [("Back", UInt32(5)), ("Up", UInt32(0)), ("Down", UInt32(1)), ("OK", UInt32(4))]
        let bottom = rect.maxY - safeAreaInsets.bottom - 58
        let width = (rect.width - 40) / CGFloat(labels.count)
        for (offset, item) in labels.enumerated() {
            let button = CGRect(x: 8 + CGFloat(offset) * width, y: bottom, width: width - 8, height: 44)
            UIGraphicsGetCurrentContext()?.setFillColor(palette.selection.withAlphaComponent(0.72).cgColor)
            UIGraphicsGetCurrentContext()?.fill(button)
            drawText(item.0, in: button.insetBy(dx: 8, dy: 11), font: .boldSystemFont(ofSize: 15), color: palette.text, alignment: .center)
        }
    }

    @objc private func swipeUp(_ recognizer: UISwipeGestureRecognizer) { host?.performMenuAction(1) }
    @objc private func swipeDown(_ recognizer: UISwipeGestureRecognizer) { host?.performMenuAction(0) }
    @objc private func swipeLeft(_ recognizer: UISwipeGestureRecognizer) { host?.performMenuAction(5) }
    @objc private func swipeRight(_ recognizer: UISwipeGestureRecognizer) { host?.performMenuAction(4) }
    @objc private func cancel(_ recognizer: UILongPressGestureRecognizer) {
        guard recognizer.state == .began else { return }
        host?.performMenuAction(5)
    }
    @objc private func tap(_ recognizer: UITapGestureRecognizer) {
        let location = recognizer.location(in: self)
        if handleControlTap(location) { return }
        let rowHeight = CGFloat(max(retrofront_menu_driver_row_height(), UInt32(driver == "rgui" ? 34 : 58)))
        let row = Int((location.y - (safeAreaInsets.top + 104)) / rowHeight)
        if row >= 0 && row < entries.count {
            _ = retrofront_menu_set_selected_index(entries[row].index)
        }
        host?.performMenuAction(4)
    }

    private func handleControlTap(_ location: CGPoint) -> Bool {
        let labels: [UInt32] = [5, 0, 1, 4]
        let bottom = bounds.maxY - safeAreaInsets.bottom - 58
        guard location.y >= bottom && location.y <= bottom + 44 else { return false }
        let width = (bounds.width - 40) / CGFloat(labels.count)
        let index = Int((location.x - 8) / width)
        guard index >= 0 && index < labels.count else { return false }
        host?.performMenuAction(labels[index])
        return true
    }

    private var host: RetrofrontHostViewController? {
        sequence(first: next, next: { $0?.next }).first { $0 is RetrofrontHostViewController } as? RetrofrontHostViewController
    }

    private func drawText(_ text: String, in rect: CGRect, font: UIFont, color: UIColor, alignment: NSTextAlignment = .left) {
        let paragraph = NSMutableParagraphStyle()
        paragraph.alignment = alignment
        (text as NSString).draw(in: rect, withAttributes: [
            .font: font,
            .foregroundColor: color,
            .paragraphStyle: paragraph,
        ])
    }

    private func readString(_ fill: (UnsafeMutablePointer<CChar>?, Int) -> Bool) -> String {
        var buffer = [CChar](repeating: 0, count: 1024)
        let capacity = buffer.count
        _ = buffer.withUnsafeMutableBufferPointer { fill($0.baseAddress, capacity) }
        return String(cString: buffer)
    }
}

private struct MenuPalette {
    let background: UIColor
    let title: UIColor
    let text: UIColor
    let subtext: UIColor
    let selection: UIColor

    init(driver: String) {
        switch driver {
        case "rgui":
            background = UIColor(red: 0.02, green: 0.08, blue: 0.10, alpha: 1)
            title = .white
            text = .white
            subtext = UIColor(white: 0.78, alpha: 1)
            selection = UIColor(red: 0.0, green: 0.45, blue: 0.50, alpha: 1)
        case "xmb":
            background = UIColor(red: 0.03, green: 0.06, blue: 0.16, alpha: 1)
            title = .white
            text = .white
            subtext = UIColor(red: 0.70, green: 0.80, blue: 1.0, alpha: 1)
            selection = UIColor(red: 0.15, green: 0.34, blue: 0.85, alpha: 0.85)
        case "materialui":
            background = UIColor(red: 0.06, green: 0.08, blue: 0.10, alpha: 1)
            title = .white
            text = .white
            subtext = UIColor(white: 0.72, alpha: 1)
            selection = UIColor(red: 0.10, green: 0.55, blue: 0.36, alpha: 0.9)
        default:
            background = UIColor(red: 0.09, green: 0.09, blue: 0.10, alpha: 1)
            title = .white
            text = .white
            subtext = UIColor(white: 0.74, alpha: 1)
            selection = UIColor(red: 0.86, green: 0.42, blue: 0.10, alpha: 0.9)
        }
    }
}

private extension UISwipeGestureRecognizer {
    func configured(_ direction: UISwipeGestureRecognizer.Direction) -> Self {
        self.direction = direction
        return self
    }
}
