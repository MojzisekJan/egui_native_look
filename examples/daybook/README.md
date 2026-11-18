# Daybook: egui_native_look and SwiftUI

Two working task, notes and focus-timer applications with the same initial data and responsive three-column design. Features include task creation with project/priority, completion, confirmed removal, project/status filters, search, persistent notes, a pausable focus timer and light/dark appearance. The egui app also previews the iOS look.

## Run

From the repository root:

```sh
cargo run --locked --example daybook
python3 examples/daybook/build.py --out examples/daybook/build
open examples/daybook/build/Daybook-egui.app
open examples/daybook/build/Daybook-SwiftUI.app
```

The packaging script requires macOS 26+, full Xcode and Rust. Use `--target-dir /path/to/build-cache` to share or isolate Cargo and Swift caches. It builds locally and uses an ad-hoc signature, not a distribution certificate. No Apple fonts are bundled. The root workspace's epaint patch applies to the egui build; see `vendor/README.md`.

`Daybook.swift` also type-checks for the iOS 26 simulator. For an iOS app target, include `Daybook.swift` and `seed.json` as a bundle resource. The supplied packaging script builds macOS apps only; this demo does not establish an egui iOS host integration or physical-device validation.

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

Remaining differences include sidebar row height/selection styling, native intrinsic segment widths inside fixed SwiftUI frames, scrollbar layout, text editor insets/line breaks, minor font rasterization and window/dialog behaviour. The egui sidebar uses the library ListBox; the SwiftUI sidebar uses application navigation buttons. The multiline notes editor is egui TextEdit versus SwiftUI TextEditor. This is a useful real-app comparison, not a claim of full SwiftUI equivalence.
