# egui_native_look

[![crates.io](https://img.shields.io/crates/v/egui_native_look.svg)](https://crates.io/crates/egui_native_look)
[![docs.rs](https://docs.rs/egui_native_look/badge.svg)](https://docs.rs/egui_native_look)

egui controls drawn the way macOS, Windows, GNOME, iOS and Android draw theirs. It is
still all egui, so it runs wherever egui runs, and your app keeps its own palette and
layout.

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
`cargo run --example gallery` to try them.

Requires egui 0.37.

## License

[MIT-0](LICENSE): use it anywhere, no credit needed.
