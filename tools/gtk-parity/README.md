# Local GTK comparison

These fixtures run the crate beside actual GTK widgets styled by libadwaita. They do not compare two egui renderings. The measured reference is GTK 4.14.5 with the Ubuntu libadwaita 1.5.0 library, X11 at 96 DPI, DejaVu Sans 11 pt, and accent `#3584e4`. The egui fixture uses the same font files at 14.67 logical pixels and the software OpenGL renderer.

The runner starts a private Xvfb server using `-displayfd`. It never connects to the desktop's display. The fixtures use private configuration and cache directories, no session bus, the simple input method, and no accessibility service. They do not change desktop settings or touch the desktop clipboard. This comparison consequently does not test a real IME, screen reader, Wayland compositor, or physical touchpad.

## Run it

The tested host is Ubuntu 24.04 compatible, x86_64 Linux, with Python 3.12, PyGObject, Pillow with X11 capture support, GTK 4, Mesa software OpenGL, and DejaVu fonts. Runtime binaries can be extracted without installing system packages:

```sh
mkdir -p /tmp/gtk-parity-runtime/packages /tmp/gtk-parity-runtime/runtime
(
    cd /tmp/gtk-parity-runtime/packages
    /usr/bin/apt-get download xvfb xdotool libxdo3 libxfont2 gir1.2-adw-1=1.5.0-1ubuntu2 libadwaita-1-0=1.5.0-1ubuntu2
    for package in *.deb; do
        dpkg-deb -x "$package" ../runtime
    done
)
export CARGO_TARGET_DIR=/tmp/egui-local-build
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2
cargo build --locked --manifest-path tools/gtk-parity/Cargo.toml
python3 tools/gtk-parity/run.py \
    --runtime /tmp/gtk-parity-runtime/runtime \
    --binary "$CARGO_TARGET_DIR/debug/gtk-parity-compare" \
    --out /tmp/gtk-parity-results \
    --visuals
```

The pinned libadwaita packages intentionally avoid distribution-specific theme modifications. Another GTK or libadwaita version is a different reference. `metadata.json` records the versions actually loaded. The script expects the extracted Ubuntu directory layout; it is not a cross-platform dependency installer.

Omit `--visuals` to run only interactions. The runner exports raw states after every observation and checks the complete paired trace at the end. A command sequence acknowledgement followed by two more fixture frames fences asynchronous input and rendering. Hover and held-press screenshots also wait for this fence before allowing the animation to settle. Keyboard-focus captures send Tab before focusing the target, so GTK itself enables keyboard focus visibility. Pointer-focus captures retain the modality after releasing the mouse. Native GTK keeps keyboard focus styling for about three seconds after navigation, including across an intervening click, and then hides it while idle. The project tracks the same interval per viewport.

## What is compared

The interaction matrix covers plain and modified button, checkbox and switch activation; closed and open dropdown navigation; both ends and both axes of a horizontal radio group; numeric steps, caret shortcuts, invalid drafts and commit; slider shortcuts; Czech text editing and selection; list navigation with and without a disabled row; nested menu activation; dialog confirmation and cancellation; a nested scroll wheel step; and focus transfer on pointer press/release, including preservation of editor focus when a switch is clicked.

XTest wheel button press and release each become a wheel event in winit 0.30.13's X11 path, whereas GTK acts on the press. The wheel measurement therefore records one button press, then releases it after taking the observation. `wheel-events.log` retains the delivered event. No deduplication is applied to library input, which could discard legitimate device events.

The visual matrix captures light and dark schemes, enabled and disabled controls, plus hover, held press, pointer focus and keyboard focus for nine controls. State review sheets place GTK above egui in each cell. Full-window comparisons place GTK on the left and egui on the right. `visual-metrics.json` records the measured sizes and dominant solid interior colors; it can also be regenerated with `python3 tools/gtk-parity/metrics.py /tmp/gtk-parity-results`. The native window has a small client-side frame; geometry comes from each widget's measured bounds, not absolute screenshot coordinates. Natural text widths can differ because Pango and egui shape and rasterize text differently.

`compare.py` checks values, popup state, text, selection extents, and the caret when the selection is collapsed. Numeric comparison allows an absolute difference of 0.0001. It retains raw focus observations but does not equate GTK descendant focus with egui popup-layer focus, or compare which end of a nonempty selection each API calls its position. The pointer-transfer scenarios also compare editor focus and target focus, treating an open dropdown popup as part of its control. The Rust integration tests separately check other focus transfers and accessibility actions.

Two existing API policies deliberately differ from a default `GtkSpinButton` and `GtkListBox`:

- Number fields commit only fully parsed, finite values. Invalid drafts preserve the previous model. Default GTK update policy may turn invalid text into zero, accept a numeric prefix, or clamp infinity. Locale-specific parsing remains an explicit custom parser.
- `ListBox::item_enabled(false, ...)` skips the row during navigation and activation. GTK's list can select an insensitive row through keyboard navigation. Subsequent page positions differ until the selections meet again.

The comparator lists the exact native/project values for these eleven observations. Other fields in the same observation still have to match. Any additional difference makes the command fail. These exceptions are reported as policy differences, not counted as matching GTK behavior.

The fixture is a separate, unpublished Cargo package and is excluded from the library's published package. Regression tests for measured defects live in the main `tests/` suite; no CI configuration is added.


## Live IBus and Orca

The repeated-dialog X11 test currently exposes an unresolved IME delivery failure with stock winit 0.30.13 and IBus 1.5.29. A single complete run can pass, so it must not be treated as release clearance. See the live audit's failing repeated-dialog trace. The exact cause within eframe/winit/XIM/IBus is not isolated. A proposed XIC refocus/flush change was tested in a private copy and did not fix the regression; no speculative dependency patch is applied.

`live.py` adds actual IBus engines, AT-SPI actions and Orca to the native comparison. It requires the system's `ibus-daemon`, Orca, pyatspi, PyGObject IBus bindings and AT-SPI bus launcher. The additional engine runtime can be extracted without changing installed packages:

```sh
mkdir -p /tmp/ime-runtime/packages /tmp/ime-runtime/runtime
(
    cd /tmp/ime-runtime/packages
    /usr/bin/apt-get download ibus-m17n libm17n-0 m17n-db libotf1
    for package in *.deb; do
        dpkg-deb -x "$package" ../runtime
    done
)
glib-compile-schemas /tmp/ime-runtime/runtime/usr/share/glib-2.0/schemas
cargo build --locked --all-features --example gallery
/usr/bin/python3 tools/gtk-parity/live.py \
    --runtime /tmp/gtk-parity-runtime/runtime \
    --ime-runtime /tmp/ime-runtime/runtime \
    --binary "$CARGO_TARGET_DIR/debug/gtk-parity-compare" \
    --gallery-binary "$CARGO_TARGET_DIR/debug/examples/gallery" \
    --dialog-cycles 20 \
    --out /tmp/ibus-orca-results
```

Use the build environment above for both Cargo commands. The output directory must be empty. Each run creates its own Xvfb display, session bus, AT-SPI bus, IBus daemon and Orca preferences. The runner terminates its own processes when it exits. It does not change the desktop's input method, accessibility settings, clipboard or audio. Orca speech is collected from its generated-speech debug output, with speech playback and braille disabled.

The tests send physical ASCII key sequences through XTest. IBus m17n `t:latn-post` generates Czech preedit and commits; `t:unicode` produces a composed numeric digit. No synthetic egui IME events are injected by this runner. It checks numeric draft/commit boundaries, dialog confirmation and cancellation, repeated dialog focus, focus transfer during composition, AT-SPI editing and value actions, disabled controls, nonfinite values and password masking. GTK and egui expose preedit differently, so the comparison uses final committed text and model values. Orca output and complete accessible trees are retained separately. `verification.json` records success only after every assertion passes.

The runner stops when it observes less than 6 GiB free. It builds nothing itself. After preserving results, remove only the temporary Cargo output:

```sh
cargo clean --manifest-path tools/gtk-parity/Cargo.toml --target-dir "$CARGO_TARGET_DIR"
```

This tests X11 input and the two named engines. It does not verify Wayland, Japanese/Chinese candidate windows, spoken audio quality or a physical braille display. See the live audit report for measured native differences and any remaining limitations.

For the focused live regression without the full comparison:

```sh
/usr/bin/python3 tools/gtk-parity/live.py \
    --runtime /tmp/gtk-parity-runtime/runtime \
    --ime-runtime /tmp/ime-runtime/runtime \
    --binary "$CARGO_TARGET_DIR/debug/gtk-parity-compare" \
    --dialog-cycles-only --dialog-cycles 20 \
    --out /tmp/ibus-dialog-regression
```

Both successful and failed runs write `verification.json`; failures retain the last observed case and exit nonzero. Each cycle must compose `č` through IBus, consume the first Enter as an IME commit, and submit only on the second Enter. The engine is selected once before the loop, so the runner cannot hide a lost input context by re-enabling it between dialogs.
