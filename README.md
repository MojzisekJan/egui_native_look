# egui_native_look

[![crates.io](https://img.shields.io/crates/v/egui_native_look.svg)](https://crates.io/crates/egui_native_look)
[![docs.rs](https://docs.rs/egui_native_look/badge.svg)](https://docs.rs/egui_native_look)

egui controls that combine system styling with interaction behavior, with looks for
macOS, Windows, GNOME, iOS and Android.

Keyboard navigation, focus, pointer and touch gestures, and accessibility are part
of the controls themselves. Platform-specific behavior includes GTK slider
shortcuts and switch dragging. Everything is rendered by egui, so the library runs
wherever egui runs. Each look starts from its system's window and text colours
(`Schemes::for_look`); an app can set its own palette and keeps its layout.

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
egui_native_look = "0.3"
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
look::Form::new("settings").show(ui, |form| {
    form.row("Notifications", |ui| ui.add(look::Switch::new(&mut notifications)));
    form.row("Retries", |ui| ui.add(look::NumberField::new(&mut retries).range(1.0..=64.0)));
});
```

`Look::system()` picks the look by target, and `Look::touch()` gives finger-sized
controls on touch screens. `Metrics::small` and `Metrics::large` give SwiftUI's small and large control sizes in
the macOS look, for dense or roomy interfaces. The default `system` feature reads the system's accent,
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
second, keyboard navigation, or closing the list. Up and Down wrap round at the list's ends. Enter commits the focused item;
Escape and outside clicks preserve the value. Clicking another input keeps focus
on that input. Long lists scroll the focused item into view.

## Touch and assistive technology

On detected touch devices, controls reserve separate targets of at least 48 × 48
points. Icon glyphs, checkbox marks and switch pills retain their visual sizes.
Targets have their own layout space instead of overlapping neighbors. When a
platform first reports touch through an input event, larger targets take effect
between gestures. Caller-owned `TitleBar` rectangles need at least 48 points of
height to contain their targets.

Leaving a pressed touch control, scrolling, cancelling touch, hiding the control or
losing window focus cancels the tap until release. Keyboard and accessibility
activation remain available afterward. A desktop GTK button can activate after the
pointer leaves and returns before release. Sliders and the GTK switch retain their
horizontal drag behavior while vertical swipes can scroll their parent.

`accessible_label()` names icon actions, list rows, choices, tabs and progress
independently of visible values or tooltips. Name `ListRow` explicitly when its text
is painted separately. Menus expose menu roles and checked states, tabs expose
selection, sections expose expansion, colour wells expose RGBA data, and progress
indicators expose numeric bounds or an indeterminate busy state. Disabled controls
omit actions they cannot apply. Keyboard focus follows layout order and skips
disabled controls.

The native gallery enables eframe's `accesskit` feature so its accessibility tree reaches platform screen readers. Applications that disable eframe's default features must enable `accesskit` explicitly.

Tests inspect AccessKit output and simulate input in all six looks. Physical
TalkBack and VoiceOver runs remain separate device checks.

## Text input

Text fields keep Unicode selection in character units and retain typed text when focus changes. `select_all_on_focus(true)` selects the existing text before the first typing or IME event, including programmatic focus requests. Stable `id_salt` values keep focus, selection and composition attached to fields when rows reorder.

Enter used to confirm an IME candidate keeps the field focused; a later Enter ends editing. During composition, Escape requests cancellation from the input method and keeps focus. Normal Escape behavior resumes once the backend reports that composition has ended. GTK entries retain focus on Escape outside a dialog; inside a dialog, an unhandled Escape closes it. Password fields retain password IME purpose and masked accessibility output. Disabled text fields ignore text edits from accessibility actions.

Focused text fields request scrolling when focus or viewport size changes, so a
scrolling container can reveal them above a software keyboard without another
keystroke. Manual scrolling remains available while the viewport stays unchanged.
The web gallery sizes its canvas to the browser's visual viewport and follows its
resize and pan events. Pinch zoom remains controlled by the browser. Mobile
keyboard behavior still requires testing on physical Android and iOS devices.

## Dialog input

Dialogs keep keyboard navigation on their modal layer. An opened dialog focuses its
first enabled text field, except on a touch screen, where that would raise the
software keyboard. Tab and Shift+Tab reach
both body controls and actions. Focused action buttons scroll into view in short
viewports. Closing restores focus to the enabled opener, including nested dialogs.
Keep calling `show` when the open flag becomes false so application-driven closes
can restore focus too.

Mark one action with `Button::default_action()` to activate it with Enter when a
`TextField` or no control has focus. `accent()` only changes appearance. A focused
button, including an ordinary egui button, keeps its own Enter action. Opening a
dialog with Enter does not also confirm it. Popup selection and IME confirmation
never trigger the default action.

An unhandled Escape closes the top dialog. GTK closes it directly when a text field is focused. In other looks, the first Escape
leaves the field and the next closes the dialog. During composition, Escape belongs
to the input method. `close_on_escape(false)` disables Escape dismissal. Call
`ui.close()` from a body or action to close the dialog through its open flag.

## Choice and menu keyboard navigation

Windows segmented choices accept all four arrow keys. Selection follows focus without wrapping; Ctrl+arrow moves focus without changing selection, and Space selects the focused option. Tab re-entry restores focus to the selected option. Other modified shortcuts remain available to the application. Assistive technology receives the focused option and its position within the group.

Popup buttons focus the ticked menu item, or else the first enabled one, when opened from the keyboard. In the macOS look a popup button's menu opens over the button with the ticked item on the button's text. Home/End move to the first/last enabled item, and Enter activates a leaf item and closes the menu. Disabled items are skipped, and Up and Down wrap round at a menu's ends. Typing moves to the next item starting with what was typed, as in a dropdown's list. Nested menus use the same Home/End navigation within their own rows. Right opens a submenu, Left returns to its parent item, and activating a leaf closes the menu chain and restores focus to the launcher. Escape dismisses the chain without activating an item. Menus and submenus limit their height to the viewport and scroll long lists by touch, wheel or keyboard focus. Home/End reveal the destination row. Long menu labels use an ellipsis while retaining their full accessible names; touch scrolling cancels row activation.

## Tooltips and egui's pop-ups

In the macOS look egui frames tooltips as macOS's tool tip: a light grey box with a hairline edge. This crate's `PopupButton`, `Dropdown`, `submenu` and text field menus keep macOS's menu over it; egui's own `ComboBox` and `menu_button` pop-ups take the tool tip's frame, so prefer this crate's menus in that look.

## More contrast

`Theme::high_contrast(true)` follows a system asking for more contrast: controls take an edge, troughs and separators darken and secondary text comes closer to the text colour, after libadwaita's high-contrast values. egui does not report the setting; the app passes it on. In the gallery, add the word `contrast`.

## Inactive windows

In the macOS look an inactive window (egui's `focused` input is false) draws its accent in grey, as macOS does: the default button looks ordinary, checked marks, switched-on switches, chosen segments, slider fills and progress turn grey, and focus rings hide. `Colors::active` reports it for an app's own drawing.

## Lists

A click on a `ListRow` gives it the keyboard focus. In the macOS look a picked row shows the accent while a row of its list (the same `Ui`) has the focus and the window is active, and grey otherwise; `picked_text` gives the matching text colour.

## Toasts

A toast stays while the pointer is over it and runs out after the pointer leaves. Screen readers announce each toast as a polite status message.

## Radio groups

`RadioGroup` makes radio buttons one control for the keyboard: Tab stops only on the chosen option (the first enabled one while none is chosen), the arrows choose and focus the previous or next enabled option, wrapping round except in the Windows look, and Space chooses the focused option. Assistive technology receives a radio group with the options inside it. `radio_value` remains for single radio buttons laid out independently.

## Sliders, steppers and search fields

Sliders take all four arrows in every look: Up and Right raise the value, Down and Left lower it. Holding a number field's − or + (or a half of macOS's stepper) repeats the step after half a second, ten times a second, until the pointer is released or moves away. A search field shows a clear button while it holds text; the button empties the field, reports a change and keeps the focus in the field. A text field that gains the focus from the keyboard selects its whole text; a click places the caret. A secondary click or a long press on a text field opens Cut, Copy, Paste and Select All; Cut and Copy need a selection and stay off for passwords, and the field keeps its selection and focus for the command. `TextField::context_menu(false)` leaves the menu to the app.

## GTK interactions

In the GTK look, segmented choices and tabs move only the focus with the arrows and Home/End, as libadwaita's toggle groups do; Space or Enter chooses the focused option.

In the GTK look, slider Left/Right keys follow the rail, while Up/Down and +/− increase or decrease the numeric value. Ctrl changes ten steps at once. Page Up/Down increase or decrease ten steps; Ctrl+Page Up/Down move toward the left/right endpoints. Home/End select the numeric minimum/maximum. A focused slider consumes these shortcuts and keeps focus; unrelated modifiers remain available to the application. Floating sliders without an explicit step use 1% of the rail for one step, including logarithmic ranges.

A switch in any system look lets the user drag its thumb and commits the nearest state on release; the plain look's switch only toggles. Vertical gestures remain available to a scrolling container. Disabling the control, cancelling touch or losing window focus cancels the pending change. Switches expose the switch role to assistive technology in every look; indeterminate checkboxes expose a mixed state.

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

## GTK reference behavior

GTK controls are checked against GTK 4.14 and libadwaita 1.5. GTK number fields
edit text directly: dragging selects text instead of changing the value. Enter
or leaving the field commits valid text; Enter keeps focus. Typing does not
change the bound value until commit. Invalid or incomplete text preserves the
previous value, while valid numbers outside the range are clamped. Stepping
first commits the draft. The field has one Tab stop; its pointer steppers keep
focus in the editor. Prefix, suffix, formatting and parsing remain available.

The default parser uses a decimal point. Applications choose locale-specific
formatting explicitly, for example:

```rust
# fn field(ui: &mut egui::Ui, value: &mut f64) {
ui.add(egui_native_look::NumberField::new(value)
    .fixed_decimals(2)
    .custom_formatter(|value, _| format!("{value:.2}").replace('.', ","))
    .custom_parser(|text| text.replace(',', ".").parse().ok()));
# }
```

Long alert headings wrap, and responses stack at full width when their labels
do not fit side by side. Submenus size to their contents within the viewport. Focused
number fields and sliders consume wheel input; unfocused controls let their
scrolling parent handle it. Ctrl+Up and Ctrl+Down navigate away from a number field.
Alerts use connected bottom responses, and dropdowns use a bold label, a filled
arrow and a check beside the selected item's text. System accent colours can differ
between desktop distributions; the configured palette remains overridable.

Copying `theme(ctx)` preserves system accent tracking through `AccentSource`.
Use `theme.accent(Some([dark_rgb, light_rgb]))` to pin an application's accent,
or `.accent(None)` to use its palette. A fresh theme's `Automatic` policy still
infers a fixed accent from a populated `accent` field. A resolved theme records
`System` or `Fixed` explicitly.

GTK labels and glyphs use the configured desktop font when `install_fonts` can
resolve it through Fontconfig. Use `Metrics::system(Look::Gtk)` during setup to
also read its size; `Metrics::of` keeps fixed, reproducible metrics. The native
gallery uses system metrics. Font lookup is optional and falls back to installed
Adwaita or Cantarell fonts.

`Button::destructive()` marks a destructive action. `TextField::invalid(true)`
shows an error state and exposes it to accessibility tools; the application still
validates the value. Clicking a GTK form editor's label transfers focus to that
editor without adding another Tab stop. Clicking a GTK switch row label toggles
the switch. The gallery includes destructive and validation-state examples.

Use this crate's `ScrollArea` to get GTK wheel distances and nested boundary
handling. A wheel line moves by the visible extent raised to the power of 2/3;
point-based input keeps its distance. A scrollable child retains the whole wheel
event at its ends and for input in another axis. Other looks keep egui's scrolling behavior. Using `egui::ScrollArea`
directly still uses egui's policies.

GTK dialogs keep actions outside the message scroll area. If the actions alone
exceed the available height, their footer scrolls separately. `ListBox` provides
one Tab stop, arrow, Home/End and Page Up/Down navigation, Ctrl navigation without changing
selection, Enter/Space activation, accessible items and disabled-item skipping.
Item values must be unique and stable when filtering.

```rust
# fn list(ui: &mut egui::Ui, selected: &mut usize) {
use egui_native_look::{ListBox, ScrollArea};
ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
    let result = ListBox::new("documents", selected).show(ui, |list| {
        list.item(10, "Documents");
        list.item_enabled(false, 20, "Unavailable");
        list.item(30, "Pictures");
    });
    if let Some(item) = result.activated {
        // Open the item identified by its stable value.
        let _ = item;
    }
});
# }
```

GTK text fields preserve the selection that an edit replaced when undoing it.
Redo restores the resulting caret. Ctrl+Insert, Shift+Insert and Shift+Delete
also support copying, pasting and cutting on Linux. Text shaping and rasterization still use
egui and can differ from Pango.

For distributions with libadwaita 1.5 colour overrides, use
`Theme::new(Look::Gtk).schemes(Schemes::system(Look::Gtk))` during setup. This
reads supported named colour declarations from the selected installed theme;
it does not interpret arbitrary widget CSS. Fixed `Schemes::for_look` palettes
remain reproducible. The native gallery uses system schemes and metrics.

### Remaining GTK differences

Matching the tested controls does not imply complete GTK compatibility. Touchpad
unit conversion, momentum, overshoot and scrollbar fading still differ. Pixel
scroll events keep egui's distances; they are not equivalent to GDK surface units.
GTK text fields and numeric editors support Insert overwrite mode with a block
cursor. Typed text and IME commits replace the next grapheme when no text is
selected; clipboard pastes insert normally. Text shaping remains egui's.

IME surrounding deletion is handled by the editor, but the bundled winit
Wayland backend does not forward `delete_surrounding_text`. Full surrounding-text
exchange therefore still requires backend support.

Fractional-scale layout tests cover 125%, 150% and 200%. GTK and egui also pass
keyboard, pointer, menu and dialog checks through a nested Wayland compositor at
100%, 125%, 150% and 200%. This does not establish identical text rasterization
or physical-monitor behavior. An isolated Orca/AT-SPI check verifies gallery
names, roles and keyboard focus with speech disabled; spoken interaction still
needs a listening test.
List paging uses row positions and 90% of the visible height; scroll animation
and alignment can still differ from GTK. Multi-selection is not implemented.

## License

[MIT-0](LICENSE): use it anywhere, no credit needed.
