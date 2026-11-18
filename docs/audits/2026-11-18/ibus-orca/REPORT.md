# Local IBus and Orca audit, 18 November 2026

The live audit found an unresolved X11 IME delivery failure when a dialog is opened repeatedly. This run does **not** clear the project for release. The Rust tests and ordinary GTK comparison pass, but a single passing live run hid a failure that the repeated-dialog regression now reproduces.

## Corrections made

- The real gallery now publishes its native window title on the AccessKit root every pass. Previously Orca could announce an unnamed frame. The regression checks the initial title, a renamed Czech title and the fallback title. The live gallery tree exposes the named frame, Delay spin button, Volume slider and Search items field.
- The comparison fixture now stores the main entry, invalid entry, dialog entry and form number separately. Their previous shared storage could echo password test text into a different plain field. This was a fixture aliasing defect, not password disclosure by the library's password widget.
- The fixture names its controls for AT-SPI, records raw native IME events and exports IME output and native window focus. Its dialog uses the library's automatic initial focus. A native GTK entry explicitly activates its configured default response.
- `live.py` runs real IBus m17n engines, Orca and AT-SPI on private buses and a private Xvfb display. It includes a repeated-dialog regression, records failures as failures, exits nonzero, and does not overwrite a previous run when the output directory is nonempty.

## What worked

An exploratory complete run recorded 54 observations and 201 generated Orca announcements. Its raw evidence is in [baseline](baseline/verification.json). That result is evidence for those observations only, not a claim of repeatability. It predates the final fixture's removal of redundant manual dialog focus.

- XTest sends ASCII keys through IBus, not synthetic egui IME events. The Latin post-processing engine composes `c` followed by `~` into `č` once. `a` followed by an apostrophe produces `á`. The engine commits its preedit on Escape.
- The Unicode engine composes digit 7. The numeric model stays at 4 through preedit and IME commit, then becomes 7 on the next Enter in both GTK and egui.
- Composition Enter does not submit a dialog in the working first-open sequence. The following Enter submits. Focus transfer during composition commits the text once. Clicking a switch preserves editor focus.
- AT-SPI value actions set a number to 6 and clamp an out-of-range request to 10. Slider actions set 75. Editable-text actions write `č7`. Nonfinite requests do not leave nonfinite or out-of-range model values. The project ignores value actions on disabled controls.
- The password editor remains present in the accessible tree with masked text, and the sentinel does not appear elsewhere after fixture storage was separated. The project exposes the password-text role.
- Orca generates control names, roles, changed numeric values and dialog announcements. The actual gallery has an accessible window name. Generated speech was inspected in the debug log; audio playback and braille output were disabled.

Measured policy differences remain explicit. GTK's first Escape can commit m17n preedit and close the dialog in the same key event; the project's composition guard requires another Escape. GTK allows setting this disabled numeric control through AT-SPI, while the project preserves its value. GTK exposes no checkbox action in this reference; the project's click action works.

## Failing repeated-dialog regression

The final regression selects the IBus engine once and reuses one application. Each cycle opens the dialog, verifies automatic text-field focus and empty text, types `c~`, uses Enter to commit composition and a second Enter to submit.

[The first recorded final run](regression/run.log) fails in cycle 1, the second opening. The first cycle commits `č` and submits once. On the second opening, the editor receives literal `c~`; no matching native preedit/commit arrives. The first Enter therefore submits, increasing the save count to 2. The pre-Enter state still reports native window focus and an enabled IME output. A separate [repeat](regression/repeat/run.log) fails on cycle 3, the fourth opening, with the same literal `c~` and unintended first-Enter submission. This confirms that the failure is intermittent rather than limited to one fixed cycle. The [raw states](regression/live-results.json), [native IME events](regression/ime-events.jsonl) and [failed verification](regression/verification.json) preserve this distinction.

This failure is in the input delivery path through eframe 0.37.0, winit 0.30.13, XIM and IBus 1.5.29. The exact responsible component is not isolated. Direct text-field focus and the library's automatic dialog focus both reproduce it. Re-enabling an engine between cycles would hide it, so the regression deliberately does not do that. A private winit experiment restoring XIC focus, including an Xlib flush, did not fix the regression. No unverified runtime patch or registry edit remains in the project.

The reproduction is available through `live.py --dialog-cycles-only --dialog-cycles 20`. Full setup and commands are in [the fixture README](../../../../tools/gtk-parity/README.md). Resolving this failing regression remains required before claiming reliable repeated-dialog IME behavior on this X11 stack.

For protocol investigation, the versioned [IBus implementation](https://github.com/ibus/ibus/blob/1.5.29/bus/ibusimpl.c) and the cached winit sources were consulted. The diagnosis above is based on local traces, not an assertion that an upstream issue has already been confirmed.

## Other verification

[results.json](results.json) records the combined status. The main all-features suite passed 349 tests including documentation tests, with four normally ignored image tests. All nine gallery tests passed, including the accessible-title regression. Main all-target/all-feature Clippy and final fixture Clippy passed with warnings denied. Rust formatting and Python syntax checks passed.

The ordinary native GTK interaction matrix was rerun after separating fixture storage. It still reports 142 scenarios, 131 matches, eleven documented policy differences and zero unexpected differences. [The recheck](parity-recheck/comparison.json) retains its complete trace. This matrix uses the simple input method and is separate from the failing live IBus regression.

Test logs are in [checks](checks/tests-all.log). [runtime-manifest.txt](runtime-manifest.txt) records native versions. [source-manifest.txt](source-manifest.txt) records source hashes. No CI configuration, desktop input settings or desktop accessibility settings were changed. Foreign-platform execution remains deferred.

The live coverage is X11, two m17n engines, software rendering, AT-SPI actions and generated Orca speech. It does not cover Wayland IME, Japanese/Chinese candidate windows, spoken audio quality, physical braille hardware or all GTK versions. Temporary compiler output is removed after preserving the evidence; [disk-cleanup.json](disk-cleanup.json) records the reclaimed space.
