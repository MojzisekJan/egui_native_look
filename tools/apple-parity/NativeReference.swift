// Independently rendered SwiftUI reference. No egui measurements or colours are used.
import SwiftUI
#if os(macOS)
import AppKit
#else
import UIKit
#endif

#if RUST_PROBE
@_silgen_name("apple_parity_probe")
func rustProbe(_ path: UnsafePointer<CChar>) -> Bool
#endif

func option(_ name: String, _ fallback: String) -> String {
    guard let i = CommandLine.arguments.firstIndex(of: name), i + 1 < CommandLine.arguments.count else { return fallback }
    return CommandLine.arguments[i + 1]
}

@MainActor final class Model: ObservableObject {
    @Published var checked = true
    @Published var switched = true
    @Published var segment = 1
    @Published var choice = 1
    @Published var slider = 0.4
    @Published var text = "Hello"
    @Published var password = "secret"
    @Published var number = 3
    @Published var count = 0
    @Published var showAlert = false
    var geometry: [String: [Double]] = [:]
    var trace: [[String: Any]] = []
    var snapshot: [String: Any] {
        ["checked": checked, "switched": switched, "segment": segment, "choice": choice,
         "slider": slider, "text": text, "number": number, "count": count]
    }
}

struct Bounds: PreferenceKey {
    static let defaultValue: [String: CGRect] = [:]
    static func reduce(value: inout [String: CGRect], nextValue: () -> [String: CGRect]) {
        value.merge(nextValue(), uniquingKeysWith: { _, new in new })
    }
}
extension View {
    func measured(_ name: String) -> some View {
        accessibilityIdentifier(name).background(GeometryReader { geometry in
            Color.clear.preference(key: Bounds.self, value: [name: geometry.frame(in: .named("fixture"))])
        })
    }
}

struct Reference: View {
    @ObservedObject var model: Model
    let disabled = CommandLine.arguments.contains("--disabled")
    let dark = option("--scheme", "light") == "dark"
    var size: ControlSize {
        switch option("--size", "regular") {
        case "mini": return .mini
        case "small": return .small
        case "large": return .large
        default: return .regular
        }
    }
    func row<V: View>(_ label: String, @ViewBuilder content: () -> V) -> some View {
        HStack(spacing: 0) {
            Text(label).font(.system(size: 11)).foregroundStyle(.secondary).frame(width: 104, alignment: .leading)
            HStack(spacing: 12) { content() }.frame(maxWidth: .infinity, alignment: .leading)
        }.frame(height: 42)
    }
    var body: some View {
        VStack(spacing: 0) {
            row("Button") { Button { model.count += 1 } label: { Text("Save").measured("button-label") }.buttonStyle(.bordered).measured("button") }
            row("Default") { Button { model.count += 1 } label: { Text("Continue").measured("default-label") }.buttonStyle(.borderedProminent).keyboardShortcut(.defaultAction).measured("default") }
            row("Destructive") { Button("Delete", role: .destructive) { model.count += 1 }.buttonStyle(.bordered).measured("destructive") }
            #if os(macOS)
            row("Checkbox") { Toggle("Enabled", isOn: $model.checked).toggleStyle(.checkbox).measured("checkbox") }
            #else
            row("Toggle") { Toggle("Enabled", isOn: $model.checked).measured("checkbox") }
            #endif
            row("Switch") { Toggle("Switch", isOn: $model.switched).labelsHidden().toggleStyle(.switch).measured("switch") }
            row("Segmented") {
                Picker("Mode", selection: $model.segment) {
                    Text("One").tag(0); Text("Two").tag(1); Text("Three").tag(2)
                }.labelsHidden().pickerStyle(.segmented).frame(width: 216).measured("segmented")
            }
            row("Picker") {
                Picker("Choice", selection: $model.choice) {
                    Text("One").tag(0); Text("Two").tag(1); Text("Three").tag(2)
                }.labelsHidden().frame(width: 160).measured("picker")
            }
            row("Slider") { Slider(value: $model.slider, in: 0...1).frame(width: 216).measured("slider") }
            row("Text field") { TextField("Name", text: $model.text).textFieldStyle(.roundedBorder).frame(width: 216).measured("text") }
            row("Password") { SecureField("Password", text: $model.password).textFieldStyle(.roundedBorder).frame(width: 216).measured("password") }
            row("Stepper") { Stepper("\(model.number)", value: $model.number, in: 0...10).frame(width: 160).measured("stepper") }
            row("Progress") { ProgressView(value: 0.4).frame(width: 216).measured("progress") }
            row("Alert") { Button("Show alert") { model.showAlert = true }.buttonStyle(.bordered).measured("alert") }
            row("State") { Text("clicks=\(model.count) check=\(model.checked ? 1 : 0) value=\(model.slider, specifier: "%.2f")").font(.system(size: 11)) }
        }
        .controlSize(size).disabled(disabled).padding(16)
        .frame(width: 360, height: 620, alignment: .topLeading)
        #if os(macOS)
        .background(Color(nsColor: .windowBackgroundColor))
        #else
        .background(Color(uiColor: .systemBackground))
        #endif
        .coordinateSpace(name: "fixture")
        .background(GeometryReader { geometry in
            Color.clear.preference(key: Bounds.self, value: ["root": geometry.frame(in: .global)])
        })
        .onPreferenceChange(Bounds.self) { bounds in
            model.geometry = bounds.mapValues { [Double($0.minX), Double($0.minY), Double($0.width), Double($0.height)] }
        }
        .preferredColorScheme(dark ? .dark : .light)
        .alert("Save changes?", isPresented: $model.showAlert) {
            Button("Cancel", role: .cancel) {}
            Button { model.count += 1 } label: { Text("Save").measured("button-label") }
            Button("Delete", role: .destructive) { model.count += 10 }
        } message: { Text("Your changes will be saved.") }
    }
}

@MainActor func export(_ model: Model, image: Data?, root: String, metadata: [String: Any]) {
    try! FileManager.default.createDirectory(atPath: root, withIntermediateDirectories: true)
    let result: [String: Any] = ["metadata": metadata, "geometry": model.geometry, "state": model.snapshot, "trace": model.trace]
    try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys]).write(to: URL(fileURLWithPath: root + "/reference.json"))
    if let image { try! image.write(to: URL(fileURLWithPath: root + "/reference.png")) }
    print("REFERENCE_OUTPUT \(root)")
}

#if os(macOS)
// Override only this fixture's keyboard-navigation policy, without writing user defaults.
@MainActor final class KeyboardReferenceApplication: NSApplication {
    override var isFullKeyboardAccessEnabled: Bool { true }
}
@MainActor final class Delegate: NSObject, NSApplicationDelegate {
    let model = Model()
    var window: NSWindow!
    var captureAttempts = 0
    var probeActivationAttempts = 0
    var confirmedFocus = ""
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        window = NSWindow(contentRect: NSRect(x: 100, y: 100, width: 360, height: 620), styleMask: [.titled, .closable], backing: .buffered, defer: false)
        window.title = "SwiftUI parity reference"
        window.contentView = NSHostingView(rootView: Reference(model: model))
        window.appearance = NSAppearance(named: option("--scheme", "light") == "dark" ? .darkAqua : .aqua)
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            if CommandLine.arguments.contains("--probe") { self.probe(0) }
            else { self.capture() }
        }
    }
    func controls(_ view: NSView) -> [NSView] {
        [view] + view.subviews.flatMap { controls($0) }
    }
    func focusControl(_ name: String) {
        guard let r = model.geometry[name] else { return }
        let point = NSPoint(x: r[0] + r[2] / 2, y: r[1] + r[3] / 2)
        let candidates = controls(window.contentView!).filter { view in
            view is NSControl && view.acceptsFirstResponder && view.convert(view.bounds, to: window.contentView).contains(point)
        }
        confirmedFocus = ""
        if let target = candidates.last, window.makeFirstResponder(target), window.firstResponder === target {
            confirmedFocus = name
        }
    }

    func key(_ characters: String, code: UInt16, flags: NSEvent.ModifierFlags = []) {
        var flags = flags
        if [123, 124, 125, 126, 115, 119].contains(code) { flags.formUnion([.function, .numericPad]) }
        for type in [NSEvent.EventType.keyDown, .keyUp] {
            let event = NSEvent.keyEvent(with: type, location: .zero, modifierFlags: flags, timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber, context: nil, characters: characters, charactersIgnoringModifiers: characters, isARepeat: false, keyCode: code)!
            NSApp.postEvent(event, atStart: false)
        }
    }
    func pointer(_ name: String, excursion: Bool = false, returnInside: Bool = true) {
        guard let bounds = model.geometry[name] else { fatalError("Missing control \(name)") }
        let inside = NSPoint(x: bounds[0] + bounds[2] / 2, y: 620 - bounds[1] - bounds[3] / 2)
        let outside = NSPoint(x: 350, y: inside.y)
        var events: [(NSEvent.EventType, NSPoint)] = [(.mouseMoved, inside), (.leftMouseDown, inside)]
        if excursion { events.append((.leftMouseDragged, outside)) }
        if excursion && returnInside { events.append((.leftMouseDragged, inside)) }
        events.append((.leftMouseUp, returnInside ? inside : outside))
        for (type, point) in events {
            NSApp.postEvent(NSEvent.mouseEvent(with: type, location: point, modifierFlags: [], timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber, context: nil, eventNumber: 1, clickCount: 1, pressure: type == .leftMouseUp ? 0 : 1)!, atStart: false)
        }
    }
    func probe(_ index: Int) {
        // Launch Services activation can complete after applicationDidFinishLaunching.
        // A successful activation at capture time cannot validate earlier input.
        guard NSApp.isActive && window.isKeyWindow else {
            guard probeActivationAttempts < 20 else { fatalError("Native input probe requires an active key window") }
            probeActivationAttempts += 1
            NSApp.activate(ignoringOtherApps: true)
            window.makeKeyAndOrderFront(nil)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { self.probe(index) }
            return
        }
        probeActivationAttempts = 0
        let stages: [(String, () -> Void)] = [
            ("return_default", { self.key("\r", code: 36) }),
            ("command_return", { self.key("\r", code: 36, flags: .command) }),
            ("shift_return", { self.key("\r", code: 36, flags: .shift) }),
            ("button_click", { self.pointer("button") }),
            ("button_leave_return_release", { self.pointer("button", excursion: true) }),
            ("button_leave_release", { self.pointer("button", excursion: true, returnInside: false) }),
            ("checkbox_click", { self.pointer("checkbox") }),
            ("checkbox_return", { self.key("\r", code: 36) }),
            ("checkbox_space", { self.key(" ", code: 49) }),
            ("checkbox_command_space", { self.key(" ", code: 49, flags: .command) }),
            ("slider_click", { self.pointer("slider") }),
            ("slider_right", { self.key("\u{f703}", code: 124) }),
            ("slider_shift_right", { self.key("\u{f703}", code: 124, flags: .shift) }),
            ("slider_option_right", { self.key("\u{f703}", code: 124, flags: .option) }),
            ("slider_command_right", { self.key("\u{f703}", code: 124, flags: .command) }),
            ("text_click", { self.pointer("text") }),
            ("text_return", { self.key("\r", code: 36) }),
            ("focus_switch", { self.focusControl("switch") }),
            ("focused_switch_space", { self.key(" ", code: 49) }),
            ("focused_switch_return", { self.key("\r", code: 36) }),
            ("focus_segments", { self.focusControl("segmented") }),
            ("focused_segments_return", { self.key("\r", code: 36) }),
            ("picker_open", { self.pointer("picker") }),
            ("picker_down", { self.key("\u{f701}", code: 125) }),
            ("picker_return", { self.key("\r", code: 36) }),
            ("picker_reopen", { self.pointer("picker") }),
            ("picker_escape_down", { self.key("\u{f701}", code: 125) }),
            ("picker_escape", { self.key("\u{1b}", code: 53) }),
        ]
        guard index < stages.count else { capture(); return }
        let before = model.snapshot
        stages[index].1()
        // Allow native control/menu animations and deferred SwiftUI bindings to settle.
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.6) {
            self.model.trace.append(["case": stages[index].0, "before": before, "after": self.model.snapshot, "app_active": NSApp.isActive, "key_window": self.window.isKeyWindow, "confirmed_control": self.confirmedFocus, "firstResponder": String(describing: type(of: self.window.firstResponder!))])
            self.probe(index + 1)
        }
    }
    func capture() {
        if (!NSApp.isActive || !window.isKeyWindow) && captureAttempts < 8 {
            captureAttempts += 1
            NSApp.activate(ignoringOtherApps: true)
            window.makeKeyAndOrderFront(nil)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { self.capture() }
            return
        }
        let view = window.contentView!
        view.layoutSubtreeIfNeeded()
        let rep = view.bitmapImageRepForCachingDisplay(in: view.bounds)!
        view.cacheDisplay(in: view.bounds, to: rep)
        export(model, image: rep.representation(using: .png, properties: [:]), root: option("--out", "/tmp/swiftui-reference"), metadata: ["platform": "macOS", "version": ProcessInfo.processInfo.operatingSystemVersionString, "scale": window.backingScaleFactor, "scheme": option("--scheme", "light"), "size": option("--size", "regular"), "disabled": CommandLine.arguments.contains("--disabled"), "native_controls": controls(view).filter { $0 is NSControl }.map { ["type": String(describing: type(of: $0)), "acceptsFocus": $0.acceptsFirstResponder, "frame": NSStringFromRect($0.convert($0.bounds, to: view))] }, "full_keyboard_access": NSApp.isFullKeyboardAccessEnabled, "actual_app_active": NSApp.isActive, "actual_key_window": window.isKeyWindow])
        if CommandLine.arguments.contains("--quit") { NSApp.terminate(nil) }
    }
}
@main struct Main {
    @MainActor static func main() {
        let app = CommandLine.arguments.contains("--full-keyboard-probe") ? KeyboardReferenceApplication.shared : NSApplication.shared
        let delegate = Delegate()
        app.delegate = delegate
        app.run()
    }
}
#else
@MainActor final class Delegate: NSObject, UIApplicationDelegate {
    func application(_ application: UIApplication, configurationForConnecting session: UISceneSession, options: UIScene.ConnectionOptions) -> UISceneConfiguration {
        let config = UISceneConfiguration(name: "Reference", sessionRole: session.role)
        config.delegateClass = SceneDelegate.self
        return config
    }
}
@MainActor final class SceneDelegate: NSObject, UIWindowSceneDelegate {
    let model = Model()
    var window: UIWindow?
    func scene(_ scene: UIScene, willConnectTo session: UISceneSession, options: UIScene.ConnectionOptions) {
        guard let scene = scene as? UIWindowScene else { return }
        let window = UIWindow(windowScene: scene)
        window.rootViewController = UIHostingController(rootView: Reference(model: model))
        window.overrideUserInterfaceStyle = option("--scheme", "light") == "dark" ? .dark : .light
        window.makeKeyAndVisible()
        self.window = window
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            let data = UIGraphicsImageRenderer(bounds: window.bounds).pngData { _ in window.drawHierarchy(in: window.bounds, afterScreenUpdates: true) }
            let root = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0].path
            export(self.model, image: data, root: root, metadata: ["platform": "iOS", "version": UIDevice.current.systemVersion, "scale": window.screen.scale, "scheme": option("--scheme", "light"), "size": option("--size", "regular"), "disabled": CommandLine.arguments.contains("--disabled")])
            #if RUST_PROBE
            let success = (root + "/rust-probe.json").withCString { rustProbe($0) }
            print("RUST_PROBE \(success)")
            #endif
        }
    }
}
@main struct Main {
    static func main() { UIApplicationMain(CommandLine.argc, CommandLine.unsafeArgv, nil, NSStringFromClass(Delegate.self)) }
}
#endif
