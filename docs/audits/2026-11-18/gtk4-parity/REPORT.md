# GTK 4 parity, 18 November 2026

The crate was compared directly with GTK 4.14.5 and Ubuntu libadwaita 1.5.0. The reference is an independent PyGObject application using native GTK controls. The project application uses the crate through eframe. Both receive XTest input on a private Xvfb display at 96 DPI. No CI configuration was added, and no desktop settings were changed.

## Corrections

1. GTK button-like controls no longer activate on modified Space or Enter. Pointer and accessibility activation remain independent.
2. Ctrl+Home/End in numeric editors move the caret instead of changing the model to a range endpoint. Ctrl+Page Down/Up retain their native endpoint behavior. IME candidate handling remains protected, including the caret shortcuts.
3. GTK dropdown navigation stops at the first and last rows instead of wrapping. Navigation still does not commit until activation.
4. GTK radio navigation follows the layout and leaves the group at its edges. It no longer wraps or changes a horizontal group's choice on vertical navigation. Disabled options remain skipped.
5. A scroll area's wheel hit test now uses its actual viewport. Previously, a sibling area's horizontally extended text clip rectangle could scale another area's input. The new regression failed before the fix with 14.797276 points instead of GTK's 20.082989 points for one step in a 90-point viewport. After the fix, the native measurement is 20.0829885 and egui is 20.0829926, with the parent staying at zero.
6. GTK lists paint the native view background, white in the light scheme and RGB 30 in the dark scheme. Container sensitivity does not fade this background.
7. The linked numeric field's focus outline surrounds the entire control, including both step buttons, rather than just its text editor.
8. Checkbox and radio marks have the measured held-press and selected-hover fills. The unchecked pressed fill is RGB 200 in light mode and RGB 91 in dark mode; the selected pressed accent is RGB 42, 106, 182.
9. Suggested/destructive button and switch keyboard outlines sit outside their bodies with the native gap. Ordinary buttons retain their inset outline, and destructive buttons use the window accent for focus. Checkbox and radio outlines include their labels. Pointer focus alone does not show these keyboard outlines. Navigation enables them for about three seconds, including across an intervening click, before they time out while idle. The timing state is isolated per viewport. Menu row highlighting uses the same visibility mode.

10. Mouse focus transfers on press for buttons, checkboxes and dropdown launchers. Switches retain the prior library editor's focus. Scoped interaction policy preserves the application's egui setting, including when widget evaluation unwinds.
11. Text selection uses the native 30% tint in both schemes and tints selected glyphs. The previous 55% dark-mode selection was visibly too strong.

Twelve new integration-test functions cover these measurements and frame timing. A unit regression also checks restoration of the application focus policy after unwinding. Existing dropdown, radio navigation and numeric shortcut tests were corrected where their old expectations disagreed with native GTK. New tests run in the main Rust suite, including without default features.

## Native results

The paired interaction matrix contains 142 scenarios, or 284 native/project observations. It reports 131 matching scenarios, eleven documented API-policy differences, and no unexpected value, popup, text, selection or scroll differences. See [comparison.json](comparison.json) and the complete [traces.json](traces.json).

The eleven differences follow these two existing API policies:

- Number fields require a complete, finite parse and preserve the previous model for invalid text. Default GtkSpinButton update policy can produce zero, accept a numeric prefix or clamp infinity.
- Disabled project list items are skipped during navigation and activation. GtkListBox can select an insensitive row with the keyboard, which also changes subsequent page positions. The same list with all rows enabled matches throughout the measured sequence.

The comparator checks the exact native and project values for these exceptions. Any additional difference, including another field in an excepted scenario, fails the run. Raw focus observations remain in the trace; framework-specific popup ancestry and the reported end of a nonempty text selection are not equated. Pointer-transfer scenarios additionally check editor and target focus, treating an open popup as part of its dropdown. Rust integration tests check the other focus transfers separately. Global traversal among independently laid-out widgets remains egui's; the paired checker does not require identical GTK targets for every modified arrow key.

The visual matrix includes light/dark and enabled/disabled base views, plus hover, held press, pointer focus and keyboard focus for nine controls. See [states-light.png](states-light.png) and [states-dark.png](states-dark.png), with GTK above egui in each cell. Full-window comparisons put GTK on the left. The native window's small client-side frame explains the global coordinate offset; comparisons use measured widget bounds.

The fixed-width button, entry, number field, slider and dropdown measure 240 × 34 in both applications; the switch is 46 × 26 and the progress bar 240 × 8. Natural text widths differ slightly because Pango and egui use different text layout and rendering. [visual-metrics.json](visual-metrics.json) records dimensions and measured dominant interior colors. These metrics do not compare individual glyph pixels.

## Verification and limits

The existing image references remain unchanged. The local validation results and exact commands are in [checks/results.json](checks/results.json). They include formatting, all targets with all features, documentation tests, no-default-features tests, both Clippy configurations with warnings denied, all image regressions, Rust 1.95 MSRV, five library cross-compilation targets, the wasm gallery, web viewport tests, RustSec checks for both lockfiles, the separate native fixture's Clippy check, and package contents. The five foreign targets were checked by compilation; native execution remains deferred. Both RustSec checks report zero vulnerabilities. Three existing informational unmaintained-package notices remain for transitive zvariant, zvariant_derive and zbus_names.

[source-manifest.txt](source-manifest.txt) identifies the source files and regression references by SHA-256. [metadata.json](metadata.json) records the loaded GTK/libadwaita versions and font. [runtime-manifest.txt](runtime-manifest.txt) records runtime package versions and font hashes. The reproducible setup and measurement boundaries are described in [the fixture README](../../../../tools/gtk-parity/README.md).

This verifies the recorded GTK/libadwaita versions, X11 input, software rendering and the listed scenarios. It does not establish equivalence with every GTK version or stylesheet. A real Wayland compositor, physical touchpad momentum, live input methods and screen-reader speech remain outside this run. Native testing on other operating systems remains deferred as requested. Font rasterization, antialiasing, translucent text color composition and subpixel color rounding can still differ.
