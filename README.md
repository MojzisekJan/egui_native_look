# egui_native_look

[![crates.io](https://img.shields.io/crates/v/egui_native_look.svg)](https://crates.io/crates/egui_native_look)
[![docs.rs](https://docs.rs/egui_native_look/badge.svg)](https://docs.rs/egui_native_look)

egui controls that combine system styling with interaction behavior, with looks for
macOS, Windows, GNOME, iOS and Android.

Keyboard navigation, focus, pointer and touch gestures, and accessibility are part
of the controls themselves. Platform-specific behavior includes GTK slider
shortcuts and switch dragging. Everything is rendered by egui, so the library runs
wherever egui runs, and your app keeps its own palette and layout.

**[Try the gallery in your browser.](https://mojzisekjan.github.io/egui_native_look/)**
Add `?mac`, `?gtk&light` and so on to the address to open it in a given look.

| | Dark | Light |
| --- | --- | --- |
| macOS | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/mac-dark.png" width="320"> | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/mac-light.png" width="320"> |
| Windows | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/windows-dark.png" width="320"> | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/windows-light.png" width="320"> |
| GTK / GNOME | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/gtk-dark.png" width="320"> | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/gtk-light.png" width="320"> |
| iOS | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/ios-dark.png" width="320"> | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/ios-light.png" width="320"> |
| Material | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/material-dark.png" width="320"> | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/material-light.png" width="320"> |
| Plain | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/plain-dark.png" width="320"> | <img src="https://raw.githubusercontent.com/MojzisekJan/egui_native_look/main/docs/looks/plain-light.png" width="320"> |

## Use

```toml
[dependencies]
egui_native_look = "0.1"
```

```rust
use egui_native_look as look;

// Once.
let mut fonts = egui::FontDefinitions::default();
look::install_fonts(&mut fonts, look::Look::system());
ctx.set_fonts(fonts);
look::install(&ctx, look::Theme::system());

// Every frame.
look::follow_system(&ctx);
ui.add(look::Button::new("Save").accent());
ui.add(look::Switch::new(&mut snap));
look::Form::new("fillet").show(ui, |form| {
    form.row("Radius", |ui| ui.add(look::NumberField::new(&mut radius).range(0.0..=1024.0)));
});
```

`Look::system()` picks the look by target, and `Look::touch()` gives finger-sized
controls on touch screens. The default `system` feature reads the system's accent,
scheme and fonts; turn it off to get the controls alone. See the
[documentation](https://docs.rs/egui_native_look) for every control, and run
`cargo run --example gallery` to try them. `web/build.sh` builds the same gallery for
the browser.

Requires egui 0.37.

## Input contracts

Empty dropdowns, segmented choices and tab lists are disabled and preserve the selected index. An out-of-range index shows no selection until the user chooses a valid item.

Width builders restore automatic sizing for zero, negative or non-finite inputs. Positive dimensions use at least one point and at most 1,000,000 points. Spinner and list row sizes fall back to their defaults for invalid inputs. Window size builders preserve the previous value of each invalid axis. Theme application normalizes invalid control heights and font sizes to the look's defaults and caps font sizes at 256 points.

Sliders disable invalid or constant ranges. Non-finite endpoints preserve the model. Constant ranges normalize the model only with `SliderClamping::Always`. A NaN model draws at the start of a finite range and stays unchanged until editing or explicit normalization. Number fields ignore ranges with non-finite endpoints and preserve NaN when `clamp_existing_to_range(false)` is set. `response.changed()` reports model changes, including requested normalization.

## Dropdown input

Typing in an open dropdown focuses the next item matching the prefix, ignoring case.
Repeated single letters cycle through matching items. The prefix resets after one
second, keyboard navigation, or closing the list. Enter commits the focused item;
Escape and outside clicks preserve the value. Clicking another input keeps focus
on that input. Long lists scroll the focused item into view.

## Touch and assistive technology

On detected touch devices, controls reserve separate targets of at least 48 × 48
points. Icon glyphs, checkbox marks and switch pills retain their visual sizes.
Targets have their own layout space instead of overlapping neighbors. When a
platform first reports touch through an input event, larger targets take effect
between gestures. Caller-owned `TitleBar` rectangles need at least 48 points of
height to contain their targets.

Leaving a pressed control, scrolling, cancelling touch, hiding the control or
losing window focus cancels the tap until release. Keyboard and accessibility
activation remain available afterward. Sliders and the GTK switch retain their
horizontal drag behavior while vertical swipes can scroll their parent.

`accessible_label()` names icon actions, list rows, choices, tabs and progress
independently of visible values or tooltips. Name `ListRow` explicitly when its text
is painted separately. Menus expose menu roles and checked states, tabs expose
selection, sections expose expansion, colour wells expose RGBA data, and progress
indicators expose numeric bounds or an indeterminate busy state. Disabled controls
omit actions they cannot apply. Keyboard focus follows layout order and skips
disabled controls.

Tests inspect AccessKit output and simulate input in all six looks. Physical
TalkBack and VoiceOver runs remain separate device checks.

## Text input

Text fields keep Unicode selection in character units and retain typed text when focus changes. `select_all_on_focus(true)` selects the existing text before the first typing or IME event, including programmatic focus requests. Stable `id_salt` values keep focus, selection and composition attached to fields when rows reorder.

Enter used to confirm an IME candidate keeps the field focused; a later Enter ends editing. During composition, Escape requests cancellation from the input method and keeps focus. Normal Escape behavior resumes once the backend reports that composition has ended. Password fields retain password IME purpose and masked accessibility output. Disabled text fields ignore text edits from accessibility actions.

Focused text fields request scrolling when focus or viewport size changes, so a
scrolling container can reveal them above a software keyboard without another
keystroke. Manual scrolling remains available while the viewport stays unchanged.
The web gallery sizes its canvas to the browser's visual viewport and follows its
resize and pan events. Pinch zoom remains controlled by the browser. Mobile
keyboard behavior still requires testing on physical Android and iOS devices.

## Dialog input

Dialogs keep keyboard navigation on their modal layer. Tab and Shift+Tab reach
both body controls and actions. Focused action buttons scroll into view in short
viewports. Closing restores focus to the enabled opener, including nested dialogs.
Keep calling `show` when the open flag becomes false so application-driven closes
can restore focus too.

Mark one action with `Button::default_action()` to activate it with Enter when a
`TextField` or no control has focus. `accent()` only changes appearance. A focused
button, including an ordinary egui button, keeps its own Enter action. Opening a
dialog with Enter does not also confirm it. Popup selection and IME confirmation
never trigger the default action.

An unhandled Escape closes the top dialog. While editing text, the first Escape
leaves the field and the next closes the dialog. During composition, Escape belongs
to the input method. `close_on_escape(false)` disables Escape dismissal. Call
`ui.close()` from a body or action to close the dialog through its open flag.

## Choice and menu keyboard navigation

Windows segmented choices accept all four arrow keys. Selection follows focus without wrapping; Ctrl+arrow moves focus without changing selection, and Space selects the focused option. Tab re-entry restores focus to the selected option. Other modified shortcuts remain available to the application. Assistive technology receives the focused option and its position within the group.

Popup buttons focus the first enabled menu item when opened from the keyboard. Home/End move to the first/last enabled item, and Enter activates a leaf item and closes the menu. Disabled items are skipped. Nested menus use the same Home/End navigation within their own rows. Right opens a submenu, Left returns to its parent item, and activating a leaf closes the menu chain and restores focus to the launcher. Escape dismisses the chain without activating an item. Menus and submenus limit their height to the viewport and scroll long lists by touch, wheel or keyboard focus. Home/End reveal the destination row. Long menu labels use an ellipsis while retaining their full accessible names; touch scrolling cancels row activation.

## GTK interactions

In the GTK look, slider Left/Right keys follow the rail, while Up/Down and +/− increase or decrease the numeric value. Ctrl changes ten steps at once. Page Up/Down increase or decrease ten steps; Ctrl+Page Up/Down move toward the left/right endpoints. Home/End select the numeric minimum/maximum. A focused slider consumes these shortcuts and keeps focus; unrelated modifiers remain available to the application. Floating sliders without an explicit step use 1% of the rail for one step, including logarithmic ranges.

A GTK switch lets the user drag its thumb and commits the nearest state on release. Vertical gestures remain available to a scrolling container. Disabling the control, cancelling touch or losing window focus cancels the pending change. Switches expose the switch role to assistive technology in every look; indeterminate checkboxes expose a mixed state.

Slider drags preserve the grab offset on GTK and iOS thumbs. Touch interactions wait for a tap release or horizontal drag before changing the value, leaving vertical gestures to a scrolling container. Cancelling touch, disabling the control or losing window focus stops further pointer updates. In the GTK look, Escape stops an active drag at its current value; Shift reduces sensitivity without moving the thumb when the modifier changes.

GTK number fields use their configured step for Up/Down and accessibility increment/decrement, independently of drag speed. Page Up/Down use ten steps; Ctrl+Home/End and Ctrl+Page Down/Up select the numeric minimum/maximum. Accessibility value edits stay in range even when existing out-of-range model values are allowed. Disabled fields ignore accessibility edits.

Slider accessibility exposes finite values, sorted numeric bounds and the configured step. Increase/decrease actions follow numeric direction even on reversed rails and start from the visible knob position when an unclamped model is out of range. Direct value edits are clamped to the range; NaN requests are ignored. Boundary and disabled actions are not advertised, and disabled numeric editors ignore queued edits. Logarithmic sliders expose their bounds and actions without claiming a constant numeric step.

Buttons can pair a decorative glyph with their text using `Button::new("Add item").icon("+", egui::FontId::proportional(16.0))`. The icon and text share one focus stop and touch target. The text supplies the accessible name; accent and disabled colours apply to both. Use an icon font through the supplied `FontId` when needed.

## Development

Use Rust 1.95 or newer. `Cargo.lock` fixes the dependency versions used by local builds and the web gallery.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo clippy --locked --all-targets --no-default-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --no-default-features
cargo test --locked --example gallery
```

`cargo test` includes unit tests, control interaction tests, documentation examples and compile-fail tests that check discarded builders are rejected. The gallery tests exercise narrow viewport layouts. Image comparisons require a GPU or software adapter:

```sh
cargo test --locked --test snapshots -- --include-ignored
```

To generate a local coverage report with `cargo-llvm-cov` installed:

```sh
cargo llvm-cov --locked --lib --test controls --test touch --test navigation --html --output-dir target/coverage
```

Coverage reports describe the code compiled for the tested platform; they do not demonstrate behavior on other operating systems or real touch devices.

## License

[MIT-0](LICENSE): use it anywhere, no credit needed.
