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


The iOS touch-field follow-up keeps the visible TextField bezel at 34 points inside a 44-point interaction slot (48 before the sheet follow-up). Padding taps focus the editor and preserve the public clicked response; search-field clear buttons retain the full-height target. A regression checks geometry, padding activation and typing for both plain and search fields. Full library tests: 364 passed (4 GPU snapshot tests remain opt-in). The iPhone simulator screenshot confirms the smaller bezel; the attempted CUA coordinate tap failed at the automation layer, so that particular padding interaction is backed by the regression test rather than claimed as a manual simulator verification.


## iOS sheet and spacing follow-up

Both apps accept `--demo --show-add-task` for reproducible sheet captures. The egui form now uses a full-width phone sheet and a floating 580×650-point iPad sheet instead of an alert. Cancel/reopen clears the discarded draft; Return submits a valid title; Escape and the backdrop cancel; dragging the header down dismisses. IME confirmation cannot submit. Destructive task removal still uses an alert.

iOS targets reserve 44 points while preserving native visual control sizes. The demo compensates for invisible touch padding around its filter/search controls and reserves the actual checkbox target width when wrapping task text. Focused iOS text fields retain a thin border rather than a macOS focus ring.

The iOS host reads UIKit's keyboardLayoutGuide and repositions the sheet into the available area above a docked keyboard. The local eframe patch normalizes UIKit Return before egui-winit drops it as a control character. A real iPad simulator run typed and submitted “iPad Return test”, verified as the seventh task. Regressions cover phone/tablet cancel, reopening, submission, IME protection, dismissal and action visibility above a keyboard; all five demo tests and 364 library/doc tests pass (four GPU snapshots remain opt-in). Root and iOS clippy pass.

See `comparison/daybook-ios-sheets.html` for final simulator captures and the remaining differences. UIKit's predictive/accessory keyboard row, full native selection, interactive presentation animation, accessibility and multitasking still differ or remain unverified. This sheet implementation belongs to the demo; consuming apps still need their own keyboard geometry integration.


### Compact sheet and safe-area follow-up

Landscape iPhone now uses the full-width sheet, with content inset around the display cutout. The modal backdrop covers the viewport instead of clipping to content safe areas. A downward gesture on blank sheet space tracks the panel; a short gesture restores it and a longer gesture dismisses it without stealing input from child controls.

Both demos keep Cancel/Add task outside the scrollable form body when a compact keyboard reduces the available height. The SwiftUI reference had the same compact-height defect: its fixed VStack put actions under the keyboard. Its iOS implementation now uses a single persistent editor in a bounded ScrollView, preserving focus through keyboard layout changes and keeping native control styling. Desktop SwiftUI retains its original layout.

Seven Rust demo tests and both desktop/iOS clippy pass. Added regressions check the actual backdrop clip rect, action visibility/activation with compact keyboards, landscape width and short/long blank-area drags. SwiftUI compiled for iOS and macOS and passed its model self-test. Real landscape iPhone checks added a task using the egui soft-keyboard Return and the SwiftUI Add task button. CUA coordinate clicking of the egui action still failed at the automation layer; that activation is covered by the harness, not claimed as a manual pass. The updated comparison includes portrait, landscape and keyboard-visible captures for both implementations. Keyboard predictions/accessory rows, colour and full text-editing parity remain open.


### Alert actions, safe areas and UIKit keyboard follow-up

Both apps accept `--demo --show-remove-task` for reproducible alert captures. The demo declares the destructive action first in the Dialog's documented right-to-left action layout, putting Keep task on the left and Remove on the right. All iOS alert actions now use the semibold family; the backdrop covers the entire viewport including status/home safe areas.

The iOS host explicitly applies the app's light/dark appearance to its UIWindow because winit 0.30 ignores set_theme on iOS. A small, non-text UIKit command responder delivers Escape while the removal alert is open. This fixes a real simulator failure that synthetic egui keyboard tests could not detect. On dismissal the host ends inherited editing to prevent UIKit restoring the UIKeyInput parent and opening an unwanted keyboard. This is demo-specific host integration, not a general keyboard adapter for Bevy.

Live iPhone and iPad checks cancelled the alert with Escape while preserving all six tasks. The final iPad check also verifies the keyboard remains hidden. Native SwiftUI ignores unfocused Return and cancels with Escape. Regression tests cover backdrop non-dismissal, action ordering, cancellation, explicit deletion, semibold action labels and safe-area dimming. All 365 library/doc tests and eight demo tests pass; four GPU snapshot groups remain opt-in. Root and iOS clippy pass.

See `comparison/daybook-ios-alerts.html` for light/dark simulator comparisons on both devices and keyboard appearance captures. Native glass/blur, exact description wrapping, predictive/accessory rows, native text selection, general hardware keyboard commands and physical-device coverage remain open.


### Dialog typography and material follow-up

`--demo --show-remove-task --long-alert` opens the same long title/message in both implementations. iOS alert titles now wrap instead of truncating, reserve their measured multi-line height, and keep clear of the message. `Dialog::message(...)` adds an optional plain-text message before the custom body; on iOS it uses 20-point message line advances and a conservative orphan-word push-out approximation. It preserves paragraphs and leaves long unbreakable words to egui. Custom `ui.label` body content retains egui wrapping. The two captured short/long examples now break at the same words as SwiftUI on iPhone and iPad.

The alert has 40-point corners, a two-point material rim (320-point outer width with 316-point content), adjusted vertical insets, and message contrast sampled against the native reference. Dark alerts now use a lighter surface/action fill and 50% backdrop dimming; light alerts keep 20%. These are static material approximations, not actual native backdrop blur or continuous corners.

All 367 library/doc tests and eight demo tests pass; four GPU snapshot groups remain opt-in. New regressions verify full multi-line headings, no overlap with message/actions at 320/402/1032-point viewports, and preservation of paragraphs, Unicode words and unbreakable strings. Desktop/iOS clippy pass. The final geometry changes were followed by another controls/demo run and both clippy checks. See `comparison/daybook-ios-dialog-wrap.html` for all twelve simulator captures, enlarged dialog comparisons and limits. Physical devices, Dynamic Type, broader language coverage, exact button font weights and glass rendering still need work. Apple's [line-break strategy documentation](https://developer.apple.com/documentation/uikit/nsparagraphstyle/linebreakstrategy-swift.struct) describes the native push-out behavior; the implementation here intentionally approximates only the single-word orphan case.
