# Changelog

## Unreleased

- Fixed: pop-up buttons and dropdowns of the same id in the rows of a loop open their own
  list; before, a click opened them all and they closed at once.
- Fixed: the arrow keys move a slider by more than one step; the focus stays on it.
- Breaking: `NumberField` is drawn as an entry of the look (the entry's fill, a thin edge,
  WinUI's and Material's underline, the focus ring), not as a button. Unbounded, it keeps
  to its type's range (0 to 255 for a `u8`); a negative step counts as its size, a zero
  step as 1.
- Breaking: `Segmented` takes the focus as a whole, and Left and Right move the choice;
  `Tabs` follow the arrows too. Segments are radio buttons to a screen reader, and
  Material's segments draw the focus ring.
- `follow_system` no longer asks for a frame every two seconds, so an idle app sleeps. It
  reads the system on the first frame and as the window comes to the front, and on
  Windows and macOS (from the registry and the application's appearance, without a
  process) every two seconds of frames drawn anyway. GNOME's reading still takes a process
  off the UI thread; what it finds is applied on the UI thread, and only to the look it
  was read for.
- `on_accent` follows the accent's lightness: dark text on a yellow or a pale accent.
- `install` no longer sets egui's `override_text_color`; the body and button text styles
  take `Metrics::body`, as documented.
- Without `install`, the controls take this system's look with its own accent, read from
  nothing; before, the first control ran `gsettings` on Linux.
- `cap_middle` measures the context's own fonts; a change of fonts, or another context,
  no longer leaves text off centre.
- `Metrics::scaled` holds the factor between 0.25 and 8, and ignores one that is not a
  number. `ProgressBar::new(NaN)` shows an empty bar. A slider's range may run from high
  to low.
- Escape closes only the dialog on top, and not while a field has the focus or a pop-up
  is open.
- Names for a screen reader: `Switch::accessible_label`, `Slider::accessible_label`, the
  close buttons of `Dialog` and `ToolWindow`, the buttons of `TitleBar`. The names a
  control writes itself can be translated: `Dialog::close_label`,
  `ToolWindow::close_label`, `TitleBar::button_labels`, `ColorWell::label`.
- Long text wraps or is cut short: check box and radio labels wrap with the `Ui`, toasts
  wrap, form labels and window titles are cut short with the whole text under the
  pointer.
- Toasts show in the viewport they were asked for in.
- The `system` feature, on by default, holds everything that reads the system.
- `rust-version` is 1.95; the snapshot pictures are left out of the package, and the
  snapshot tests out of a plain `cargo test`.

- Breaking: `NumberField::new` takes the value alone and is unbounded; `.range()` bounds
  it, with any of egui's numbers. `Palette` has `surface`, the fill of floating windows,
  dialogs, menus and pop-ups.
- `NumberField::steppers(false)`: the field without − and +.
- `Slider`'s number takes `prefix`, `suffix`, `custom_formatter`, `custom_parser` and
  decimals, as `NumberField` does.
- `MenuItem::submenu`: an item with an icon that opens a submenu.
- `Palette::of(dark)`, to change a few colours of the default palette.
- `install` keeps what the app set in egui's style and the crate does not draw with:
  shadows, spacing, the heading.
- `follow_system`: the accent and the scheme follow the system's while the app runs,
  read again as the window comes to the front; `system_scheme`.
- `PopupButton` with a list of the app's own, `menu_value`, `submenu`, and `combo_icon`
  for egui's `ComboBox`.
- `Slider` takes any of egui's numbers, a label, a logarithmic scale, steps, clamping and
  the number beside it.
- `NumberField` takes `speed`, `prefix`, `suffix`, `custom_formatter`, `custom_parser`,
  decimals and `clamp_existing_to_range`; typing replaces the number.
- `TextField::select_all_on_focus`.
- `Tab` and `TabBar`, one tab at a time.
- `Section`: the header's response, opened or closed from code, an id of its own.
- `ToolWindow` and `window_frame` for floating windows.
- `MenuItem` is as wide as its content rather than all the room there is.
- `Look::touch`, `touch_screen`, `Metrics::scaled`.
- `Form`: label and control rows, aligned the look's way, a boxed list in libadwaita and
  on iOS.
- `IconButton`, `ColorWell`, and toasts (`toast`, `show_toasts`).
- `NumberField::width`.
- Every public item documented.

## 0.1.0

- Six looks: macOS, Windows, GTK, iOS, Material and plain, each dark and light.
- The theme in the context; the system's accent and fonts.
- `Button`, `Switch`, `Checkbox`, `Radio`, `Segmented`, `Dropdown`, `Tabs`, `Slider`,
  `TextField`, `NumberField`, `section`, `ListRow`, `MenuItem`, `ProgressBar`,
  `Spinner`, `Dialog`, `TitleBar`, `shortcut_text`.
