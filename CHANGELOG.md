# Changelog

## Unreleased

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
