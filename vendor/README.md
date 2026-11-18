# epaint point-size correction

Source: crates.io epaint 0.37.0 (MIT OR Apache-2.0); original licences are in epaint/.

The isolated source patch passes the logical font size to harfrust ShapeOptions. Without it AAT trak uses its 12-point fallback at every text size. SF therefore has incorrect spacing despite loading the right system font. This checkout and both independent fixture workspaces use the patch. No Apple fonts are included.

Cargo patches do not propagate from a library dependency to a consuming workspace. Applications consuming this checkout must add the following to their own workspace root until an upstream release includes the fix:

```toml
[patch.crates-io]
epaint = { path = "path/to/egui_native_look/vendor/epaint" }
```

The source change is also preserved as epaint-point-size.patch for upstream review. This is not a published upstream fix. The normal-size SF tracking regression verifies size dependence and invariance under display scale using the locally installed system font.
