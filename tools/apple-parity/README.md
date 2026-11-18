# SwiftUI reference comparison

The native reference is authored directly in SwiftUI and does not import any egui
measurements, colours or screenshots. It covers bordered/default/destructive buttons,
toggles, switches, segmented/menu pickers, sliders, text/secure fields, steppers and
progress. Captures include light/dark, disabled, small and large control sizes.

Requirements: an Apple silicon Mac running macOS 26 or newer, full Xcode with
Apple platform SDKs 26 or newer, Rust, and Python 3. Pillow is needed
only for the comparison images. For iOS use an **existing booted simulator** with
iOS 26 or newer; the runner does not create devices or download runtimes. The
`aarch64-apple-ios-sim` Rust target must already be installed. macOS-only runs do
not need that target or a simulator.

```sh
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
xcrun simctl list devices available
# Boot a selected existing device if needed:
xcrun simctl boot DEVICE_UDID
python3 tools/apple-parity/run.py --out /tmp/apple-parity --device DEVICE_UDID
python3 tools/apple-parity/compare.py --input /tmp/apple-parity --out /tmp/apple-parity/comparisons
```

Omit `--device` for macOS only. `--target-dir` can reuse an existing Cargo build
directory. Outputs are never written into the source tree. The runner installs
and launches only `dev.egui.apple-parity-reference` in the selected simulator.
It leaves the simulator and test app available for manual inspection.

`run.py` compiles both fixtures, saves native geometry and screenshots, renders
the egui fixture using Metal, and records native macOS mouse/key event traces.
Eight assertions cover Return/default action, modifier handling, ordinary clicks,
pointer excursions and Return from an editor. The trace's slider-arrow cases have
no native slider focus under the captured keyboard-access policy and are **not**
assertions about slider keyboard semantics.

The iOS reference links `ios-probe`, which invokes the real Rust library in the
simulator process. It verifies that CoreText finds the system UI font, egui can
lay out text using it, and controls allocate touch targets. This is not a test of
UIKit delivering touch or software-keyboard events to an egui application. The
egui comparison PNGs are rendered on macOS; this distinction is retained in
`verification.json`.

The three-column images optionally accept `--baseline PATH`, where PATH contains
old `mac-light/egui.png` and `ios-light/egui.png` captures. `compare.py` converts
embedded ICC profiles to sRGB, crops the iOS fixture using its measured root
geometry, and compares logical point dimensions.

The macOS reference is launched as an app bundle through Launch Services; each
capture must report both an active app and a key window. A command-line Cocoa
process can silently produce inactive AppKit controls even when some SwiftUI
controls appear active. Such captures are rejected.

It does not turn a whole-image pixel error into a misleading overall “parity percentage”. Native and egui text
rasterizers and capture scales differ.

Interpret the mapping carefully:

- SwiftUI `.frame(width:)` may center an intrinsic control; egui `.width()` sets
  the control's own width. Picker/segment width differences can therefore come
  from the layout API, not the primitive's drawing.
- SwiftUI's automatic iOS `Toggle` is a switch. The egui `Checkbox` row is a
  deliberately different control and is not an equivalent native reference.
- `NumberField` includes an editable field; SwiftUI `Stepper` contains a label
  and step buttons. Their middle portions should not be compared as equivalents.
- The captured UIKit secure field omits its password glyphs. Do not use that row
  to assess visual masking or infer the application's text value.

The report in `docs/audits/2026-11-18/apple-parity/REPORT.md` distinguishes fixes,
confirmed remaining differences and device checks that have not been performed.
Do not redistribute Apple's system font files; the fixtures read them locally.
