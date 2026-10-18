# egui_native_look

![The controls in each look, dark and light](docs/looks.png)

egui controls drawn the way macOS, Windows, GNOME, iOS and Android draw theirs.
Everything stays egui, so it runs wherever egui runs; the app keeps its own palette and
layout.

| Look | Style | Controls | Text | Accent |
| --- | --- | --- | --- | --- |
| `Mac` | SwiftUI / AppKit | 22 pt | San Francisco | System Settings |
| `Windows` | WinUI 3 / Fluent 2 | 32 px | Segoe UI Variable | Personalization |
| `Gtk` | GTK 4 / libadwaita | 34 px | Adwaita Sans, Cantarell | GNOME accent |
| `Ios` | SwiftUI / UIKit | 44 pt | the app's | systemBlue |
| `Material` | Material 3 | 40 dp | Roboto if present | baseline primary |
| `Plain` | the app's own | 28 | the app's | the app's |

`Look::system()` picks by target; the web gets `Plain`.

## Use

```toml
[dependencies]
egui_native_look = { git = "https://github.com/MojzisekJan/egui_native_look" }
```

```rust
use egui_native_look as look;

// Once.
let mut fonts = egui::FontDefinitions::default();
look::install_fonts(&mut fonts, look::Look::system());
ctx.set_fonts(fonts);
look::install(&ctx, look::Theme::system());

// In the UI. The scheme follows egui's (ctx.set_theme).
ui.add(look::Button::new("Save").accent());
ui.add(look::Switch::new(&mut wireframe));
ui.add(look::Checkbox::new(&mut snap, "Snap to grid"));
look::radio_value(ui, &mut tool, Tool::Move, "Move");
ui.add(look::Segmented::new("axis", &mut axis, &["X", "Y", "Z"]));
ui.add(look::Dropdown::new("shape", &mut shape, &["Cuboid", "Sphere"]));
ui.add(look::NumberField::new(&mut radius, 0.0..=1024.0));
ui.add(look::Slider::new(&mut volume, 0.0..=100.0));
ui.add(look::TextField::new(&mut name).hint("Name"));
look::section(ui, "layers", "Layers", true, |ui| { /* … */ });
look::Dialog::new("export", "Export").show(&ctx, &mut open, |ui| { /* body */ }, |ui| { /* buttons */ });
```

Also: `Tabs`, `ListRow`, `MenuItem`, `ProgressBar`, `Spinner`, `TitleBar` for an
undecorated window, and `shortcut_text` (⇧⌘S or Ctrl+Shift+S). `Theme` takes the app's
palette (`Schemes`) and sizes (`Metrics`); `colors(ctx)` gives the resolved colours for the
app's own drawing; the painting functions (`paint_button`, `focus_ring`, ...) are public.

Text is centred on its capitals rather than on egui's line box, so labels sit in the
middle of their controls at every size (`text`, `galley_top`, `entry_margin`).

## Examples and tests

- `cargo run --example gallery [mac|windows|gtk|ios|material|plain] [light]`
- `cargo test` clicks every control in every look and compares each look, dark and
  light, with the pictures in `tests/snapshots` (`UPDATE_SNAPSHOTS=1` to accept a change).
- `cargo bench` times a frame of 200 rows of controls in each look against egui's own.

Requires egui 0.36.

## Licence

[MIT-0](LICENSE): use, change and ship it anywhere, commercially or not, with no credit
or notice required.
