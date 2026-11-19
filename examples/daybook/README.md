# Daybook: egui_native_look and SwiftUI

Two working task, notes and focus-timer applications with the same initial data and responsive three-column design. Features include task creation with project/priority, completion, confirmed removal, project/status filters, search, persistent notes, a pausable focus timer and light/dark appearance. The egui app also runs on iOS simulators through eframe/WGPU, using the same application source.

## Run

From the repository root:

```sh
cargo run --locked --example daybook
python3 examples/daybook/build.py --out examples/daybook/build
open examples/daybook/build/Daybook-egui.app
open examples/daybook/build/Daybook-SwiftUI.app
```

The packaging script requires macOS 26+, full Xcode and Rust. Use `--target-dir /path/to/build-cache` to share or isolate Cargo and Swift caches. It builds locally and uses an ad-hoc signature, not a distribution certificate. No Apple fonts are bundled. The root workspace's epaint patch applies to the egui build; see `vendor/README.md`.

Build both apps for an existing iPhone or iPad simulator (full Xcode and the matching Rust iOS simulator target required):

```sh
python3 examples/daybook/build.py --platform ios-simulator --out examples/daybook/build
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcrun simctl install DEVICE_ID examples/daybook/build/Daybook-egui-iOS.app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcrun simctl launch DEVICE_ID dev.egui.daybook.egui.ios --demo
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcrun simctl install DEVICE_ID examples/daybook/build/Daybook-SwiftUI-iOS.app
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcrun simctl launch DEVICE_ID dev.egui.daybook.swiftui.ios --demo --capture-ios
```

`--capture-ios` writes `Documents/daybook.png` and `daybook.json` in the simulator app container after layout settles. The PNG is at one pixel per point, excludes window safe areas, and the JSON records its dimensions. Use `simctl get_app_container DEVICE_ID dev.egui.daybook.swiftui.ios data` to find them. Render the egui comparison with `--demo --ios --size WIDTHxHEIGHT --render path.png`. That `--render` command is a macOS rendering; use `simctl io DEVICE_ID screenshot path.png` to capture the actual running iOS egui app, including system bars. Do not confuse these two evidence types.

Validated with iOS 26.5 on iPhone 17 and iPad Pro 13-inch (M5). Both implementations run in the simulators. The egui iPhone flow opened New task at its visible position, typed and added a task, then found it through Search. The SwiftUI iPhone create-task flow was also exercised. iPad egui portrait/landscape rendering and SwiftUI sheet presentation were inspected.

The iOS workspace uses WGPU/Metal instead of the desktop Glow backend. It requests no more inter-stage shader variables than the adapter supports (the simulator exposes 15, below WGPU's default 16). It opts into a local eframe surface-size correction, documented in `vendor/README.md`; without this, the framebuffer stretches vertically and touch targets do not match the image. The application respects `Context::content_rect()` for safe-area layout. Backend diagnostics are saved in the app container's `Documents/backend.log`.

Known gaps: no iOS AccessKit accessibility tree in this configuration; Command-A did not select all during the simulator check; complete software-keyboard, selection/IME and iPad multitasking parity remains unverified. Native touch targets also expose layout differences hidden by mouse-only desktop previews. This is a working simulator host, not certification of SwiftUI parity, a Bevy adapter, or physical-device validation.

## Local data

The applications save separately so comparing them cannot overwrite the other implementation's state:

- macOS egui: `~/Library/Application Support/Daybook/egui.json`
- SwiftUI: the platform Application Support directory, `Daybook/swiftui.json`
- Linux egui: `$XDG_DATA_HOME/daybook/egui.json`, falling back to `~/.local/share/daybook/egui.json`

`DAYBOOK_DATA` overrides the egui file path. Both accept `--demo` for a fresh, non-persistent preview, and `--dark` for dark appearance. Corrupt saved data is not overwritten; a visible error disables persistence until the file is repaired. Timer running state is session-only. The application is a single-window demo; do not edit the same implementation's persistent file from simultaneous processes.

## Reproduce and verify

```sh
cargo test --locked --example daybook
cargo clippy --locked --example daybook -- -D warnings
examples/daybook/build/Daybook-SwiftUI.app/Contents/MacOS/Daybook --self-test
cargo run --locked --example daybook -- --demo --render /tmp/daybook-egui.png
cargo run --locked --example daybook -- --demo --dark --render /tmp/daybook-egui-dark.png
cargo run --locked --example daybook -- --demo --compact --render /tmp/daybook-egui-compact.png
DAYBOOK_SHOT=/tmp/daybook-swiftui.png examples/daybook/build/Daybook-SwiftUI.app/Contents/MacOS/Daybook --demo
```

The Rust tests cover data edits/round-tripping and the complete create-task/dialog/search flow with accessibility checks. SwiftUI self-tests cover model edits, JSON round-tripping and timer transitions. `--render` uses the actual eframe UI at 1180×800 (390×800 with `--compact`); SwiftUI capture sets the same desktop content size and excludes the title bar. Avoid taking a focus-sensitive capture while changing the active application.

## What comparison revealed

The first demo version had several application-level mismatches, since corrected:

- `RichText::strong()` changes text colour, not font weight: explicitly use the installed semibold family and a system-font bold heading family.
- Extra layout spacing and a control-height horizontal metadata row made egui task cards taller. Rich metadata text now takes only its natural height.
- A search-styled egui field was compared with a plain SwiftUI TextField; both now use plain fields.
- A NumberField was compared with a Stepper alone; both now combine an editable value and stepper.
- An explicit SwiftUI tint coloured ordinary buttons; the native reference now follows the normal system tint policy.

The follow-up also centers checkbox/Remove against the measured height of the complete wrapped text column, aligns 32-point navigation rows, centers the natural macOS segment width within its outer frame, and moves padding inside the scroll container. AppKit determines non-overlay scrollbar policy and gutter width on the main thread. Editor insets and persistent macOS editor scrolling now match the reference structure.

Regression coverage includes short/wrapped titles at narrow/wide widths for both Apple looks, center alignment, right-edge containment and checkbox activation. A library regression checks that scrollbar styling cannot recolour content or sibling widgets.

Remaining differences include sidebar selection styling, native editing/selection and application menus, detailed scroll physics, minor font rasterization and window/dialog behaviour. The egui sidebar uses the library ListBox; the SwiftUI sidebar uses application navigation buttons. The multiline notes editor is egui TextEdit versus SwiftUI TextEditor. This is a useful real-app comparison, not a claim of full SwiftUI equivalence.


The simulator comparison additionally fixes compact project navigation in egui, filter/search spacing, secondary heading colours, SwiftUI metric sizing based on actual column width, task text flex/wrapping, and the focus minutes layout when the native iOS stepper does not fit beside its label. Remaining visual differences include accent colour, text-editor insets, sidebar row sizing and the precise allocation of narrow task rows.


The iOS touch-field follow-up keeps the visible TextField bezel at 34 points inside a 48-point interaction slot. Padding taps focus the editor and preserve the public clicked response; search-field clear buttons retain the full-height target. A regression checks geometry, padding activation and typing for both plain and search fields. Full library tests: 364 passed (4 GPU snapshot tests remain opt-in). The iPhone simulator screenshot confirms the smaller bezel; the attempted CUA coordinate tap failed at the automation layer, so that particular padding interaction is backed by the regression test rather than claimed as a manual simulator verification.
