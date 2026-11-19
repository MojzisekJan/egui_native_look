# epaint point-size correction

Source: crates.io epaint 0.37.0 (MIT OR Apache-2.0); original licences are in epaint/.

The isolated source patch passes the logical font size to harfrust ShapeOptions. Without it AAT trak uses its 12-point fallback at every text size. SF therefore has incorrect spacing despite loading the right system font. This checkout and both independent fixture workspaces use the patch. No Apple fonts are included.

Cargo patches do not propagate from a library dependency to a consuming workspace. Applications consuming this checkout must add the following to their own workspace root until an upstream release includes the fix:

```toml
[patch.crates-io]
epaint = { path = "path/to/egui_native_look/vendor/epaint" }
```

The source change is also preserved as epaint-point-size.patch for upstream review. This is not a published upstream fix. The normal-size SF tracking regression verifies size dependence and invariance under display scale using the locally installed system font.


# eframe iOS Metal surface correction

Source: crates.io eframe 0.37.0 (MIT OR Apache-2.0); licences retained in `eframe/`.
Only the independent `examples/daybook/ios` workspace opts into this patch.

Winit 0.30 reports a full-view `Resized` event on iOS, but `Window::inner_size()` excludes safe areas. Eframe's stale-resize correction replaced the former with the latter, configuring a shorter Metal surface while input still used the full window. On iPhone 17 this stretched a 778-point surface over 874 points and displaced taps from their visible controls. Preserve the iOS event size; desktop stale-event handling remains unchanged. `eframe-ios-surface.patch` contains the isolated source change. This is a local workaround, not a published upstream fix.

Verified by opening the visible New task button, typing a title, submitting it, and filtering the new task on the iPhone simulator; iPad portrait/landscape rendering was also inspected. Keep the patch until an upstream renderer/winit integration removes the mismatch.

The same iOS-only patch normalizes winit 0.30 UIKit `Key::Character("\n"/"\r")` to Enter. egui-winit otherwise filters these control characters without emitting an egui key event. Both press and release are preserved; other platforms and ordinary input are unchanged. Verified by submitting the iPad Daybook sheet using Return on the simulator.
