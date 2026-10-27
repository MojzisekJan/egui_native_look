# Changelog

## Unreleased

## 0.3.0 - 2026-10-27

- Preserve GTK theme palette accents when no explicit or system accent overrides them.

- Support GTK Insert overwrite mode in text and numeric editors, with a block cursor, Unicode grapheme replacement and IME commit handling. Clipboard paste keeps insertion semantics.
- Add regression coverage for IME surrounding deletion and verify current gallery roles/focus through Orca/AT-SPI and controls through nested Wayland at 100–200% scale.

- Add Page Up/Down and Ctrl paging to `ListBox`, using row positions and viewport height.
- Keep orthogonal wheel events inside a scrollable GTK child instead of scrolling its parent.
- Handle GTK Ctrl+Insert, Shift+Insert and Shift+Delete clipboard shortcuts on Linux, preserving password copy restrictions.

- Add `ScrollArea` with viewport-dependent GTK wheel steps and nested boundary capture, retaining egui behavior in other looks.
- Keep GTK dialog responses outside the message scroll area; allow oversized response lists to scroll separately.
- Add `ListBox` with stable item values, one Tab stop, keyboard navigation, activation, accessibility and disabled-item skipping. Use it in the gallery.
- Preserve replaced selections during GTK Undo and restore caret position during Redo.
- Add explicit `Schemes::system` loading for installed libadwaita 1.5 colour overrides and use it in the native gallery.

- Match GTK neutral colours, disabled opacity, compact switches and checkboxes, icon button padding, progress allocation and overlay scrollbar dimensions to the native reference.
- Animate GTK button fills and switch movement with measured transition durations.
- Add `Button::destructive()` and `TextField::invalid(bool)`, including accessibility validation state.
- Focus GTK form editors when their row label is clicked, without adding a Tab stop; toggle switches through their row labels.
- Read Linux desktop fonts through Fontconfig and add `Metrics::system` for the configured text size. Use these metrics in the native gallery.

- Use text editing in GTK form number rows, skip their step buttons during Tab navigation, and draw focus around the row.
- Enable the native gallery's AccessKit bridge for platform screen readers.

- Commit GTK numeric drafts on Enter or focus transfer, keep editor focus on Enter and pointer steps, and omit step buttons from Tab navigation. Reject invalid drafts without changing the model.
- Wrap long GTK alert headings and give stacked responses the full dialog width; recalculate the layout when labels change.
- Size nested menus to their contents instead of expanding them across the viewport.

- Match measured GTK desktop pointer re-entry, focused wheel input, text selection in number fields, and entry Escape behavior. Preserve touch cancellation and scrolling through unfocused controls.
- Align GTK dropdown labels, arrows and selection marks, control rounding, and connected alert responses with the GTK 4.14 / libadwaita 1.5 reference.
- Preserve system accent tracking when copying an installed theme. Add `AccentSource` and `Theme::accent` for explicit ownership.
- Restore application style values when switching away from look-specific shadows, opacity and scrolling settings; make direct style application independent of the previous look.
- Bound checkerboard drawing and clamp inner geometry for tiny translucent colour wells.

## 0.2.0

- Frame tooltips in the macOS look as macOS's tool tip (a light grey box with a hairline edge and a small shadow) while this crate's menus keep macOS's menu, and press macOS's default button and switch darker on the light scheme and lighter on the dark, as measured from AppKit with a real pointer.
- Move to the next menu item starting with what is typed in an open menu, as macOS and GTK menus do.
- Give macOS number fields without steppers a text field's height too.
- Report the macOS switch's flush width from `switch_width`.
- Give the macOS look's scroll areas macOS's overlay scrollers: a six-point rounded handle three points in, shown while the pointer is over the area and widening under it.
- Select a text field's whole text when it gains the focus from the keyboard, as macOS, WinUI and GTK entries do; a click still only places the caret.
- Add `Metrics::small` and `Metrics::large`: SwiftUI's small (20-point, 11-point text) and large (28-point) control sizes in the macOS look, with switches, marks, steppers, knobs, rails and paddings in proportion and text fields sized as SwiftUI sizes them; the other looks scale in proportion.
- Give macOS tool windows a 32-point title bar with a 14-point close button sixteen points from the corner and a faint separator, as macOS 26 draws them.
- Draw macOS icon buttons borderless, as wide as their glyph and lit only in the accent when on, and the macOS colour well as a capsule, as SwiftUI does.
- Mark each step of a stepped macOS slider with a dot under its rail, size macOS segmented choices to their longest label when no width is given, and set macOS marks five points from their labels, as SwiftUI does.
- Grey the macOS look's accent in an inactive window, as macOS does: the default button turns ordinary, checked marks, switches, segments, sliders and progress take greys, and focus rings hide. `Colors::active` tells which.
- Grey a macOS list's selection while none of its rows has the focus or the window is inactive, and give a list row the focus when it is clicked, as macOS lists do.
- Draw macOS sheets without an edge, a shade above the window on the dark scheme, as SwiftUI does.
- Light the menu row the keyboard is on in every look; it showed no focus before.
- Match macOS menus to SwiftUI: 24-point rows that touch and span the menu, a menu 40 points wider than its pop-up button, macOS's menu fill, edge and faint separators, and labels after a 24-point column.
- Draw disabled macOS controls in their own disabled colours without egui's extra fading, and match SwiftUI's disabled switch, check box, segment, slider, stepper and number field, and its keyboard focus ring.
- Match the macOS look to SwiftUI cell by cell in sRGB: San Francisco's text optical size, fills tinted as SwiftUI's, 12-point button padding, 26-point text and number fields that keep their edge, a 20 by 24 stepper, soft knob shadows and a flush switch.
- Move only the focus with the arrows and Home/End in GTK segmented choices and tabs, choosing with Space or Enter as libadwaita does.
- Restyle the macOS look after macOS 26, measured against SwiftUI: 24-point controls with six-point corners; flat grey buttons, tracks and marks tinted from the window background; the accent on chosen segments; a wide switch with a pill knob; a pill slider knob on a six-point rail; pop-up chevrons without the accent square; a capsule search field; a stepper after number fields; an eight-spoke spinner; a plain disclosure header; and sheets without a title bar.
- Keep a toast while the pointer is over it, and announce toasts to screen readers as polite status messages.
- Let the thumb of every system look's switch be dragged, not only GTK's.
- Add a Cut, Copy, Paste and Select All menu to text fields, keeping the selection through a secondary click.
- Focus an opened dialog's first text field, except on touch screens.
- Add `RadioGroup`: one Tab stop on the chosen option, arrows that choose and wrap, disabled options skipped.
- Add a clear button to search fields, Up and Down to sliders in every look, wrapping to dropdown lists and menus, and repeating to held − and + buttons.
- Open a popup button's menu on its ticked item from the keyboard in every look, and over the button in the macOS look.

- Keep dialog actions side by side whenever they fit and stack them only when they do not, instead of stacking them in every dialog narrower than 376 points.
- Let a dialog grow with content added after it opened, up to the viewport, instead of scrolling within its opening height.
- Let the first swipe on a touch screen scroll the gallery; egui enables drag scrolling only after it has seen a touch.
- Name the gallery's Light and Switch switches, delay field and volume slider for screen readers, and check the phone layout's accessible names in tests.
- Update the README look screenshots for the current gallery and use general settings in the usage example.

- Make gallery search filter built-in and added items, preserve selection and row identity, show empty results and return focus after clearing the filter.

- Make gallery buttons and menu actions report their results, add selectable list items with the icon button and distinguish dialog confirmation from cancellation.
- Use gallery icons available in the bundled fonts and cover complete gallery interactions in all six looks.

- Remove the window shadow and hairline outline from Material dialogs so rounded corners meet the modal backdrop cleanly. Add light/dark dialog snapshots.

- Add `Button::icon` for a decorative glyph beside the label, with shared focus, touch handling and accessible name. Show regular, accent and disabled examples in the gallery.

- Use general application examples in the gallery: preferences, documents, languages and notifications.

- Bound popup menus and submenus to the viewport, scroll long lists and reveal Home/End destinations.
- Fit long menu labels and shortcuts within narrow rows while preserving complete accessible names.
- Test phone menu geometry, nested long lists, touch scrolling and outside dismissal across all six looks.

- Apply Home/End navigation inside nested menus and cover submenu entry, parent focus, leaf activation and Escape dismissal in all six looks.

- Support Windows choice focus movement with Ctrl+arrow, Space confirmation and vertical arrow navigation, with accessible group positions.
- Add Home/End navigation to popup menus and close activated leaf items on Enter.
- Cover choice focus, application shortcuts and disabled menu items in a dedicated navigation regression suite.

- Reserve non-overlapping 48-point touch targets while preserving icon and mark dimensions; defer adaptation until between pointer sequences.
- Cancel interrupted taps through late release without blocking subsequent keyboard or accessibility activation.
- Add independent accessible names and expose list selection, tab roles, checked menu items, expanded sections, colour values and progress/busy states.
- Remove unavailable actions from disabled controls, preserve focus order and add a six-look touch/accessibility regression suite to coverage.

- Restore dialog opener focus after Escape, close buttons, `ui.close()` and application state changes.
- Add explicit `Button::default_action()` and `Dialog::close_on_escape`, preserving focused control, popup and IME priority.
- Scroll focused buttons into view and cover dialog keyboard navigation, nested modals and short phone viewports.

- Keep focused text fields visible after viewport resizing without overriding later manual scrolling.
- Fit the web gallery to the visible browser viewport during software keyboard and browser chrome changes; add local viewport event tests.

- Add case-insensitive prefix navigation and repeated-letter cycling to dropdowns.
- Preserve the clicked input's focus when dismissing an open dropdown outside its list.


- Select text before the first input after keyboard or programmatic focus, preserving subsequent typing and composition updates.
- Separate IME confirmation and cancellation from text field submission and focus loss.
- Prevent accessibility edits to disabled text fields and test Unicode selections, reordered fields and password composition.

- Add slider accessibility increment, decrement and value editing, numeric bounds and step metadata, including reversed and logarithmic ranges.
- Keep non-finite model values out of slider accessibility metadata and ignore edits to disabled slider number fields.

- Preserve slider thumb grab offsets on GTK and iOS, accumulate small stepped movements and retain vertical touch scrolling.
- Stop interrupted slider drags and add GTK Escape cancellation and Shift fine adjustment.
- Separate GTK number field keyboard and accessibility steps from drag speed, add page and endpoint shortcuts, and ignore accessibility edits on disabled fields.

- Add complete slider key bindings in the GTK look, with numeric and visual directions, page steps and focus retention.
- Add GTK switch thumb dragging while preserving vertical scrolling and cancelling interrupted gestures.
- Expose switches with their own accessibility role and report mixed checkbox states and click-frame changes correctly.

- Define and test empty selections, invalid dimensions, non-finite numeric inputs and theme metric normalization.
- Disable sliders with invalid or constant bounds, avoid false change reports for unchanged NaN values, and preserve finite interpolation across extreme ranges.

- Keep both endpoints reachable when a slider step does not divide its range evenly.
- Test progress boundaries, spinner animation and title bar actions across looks.
- Allow one differing edge pixel in the macOS light control snapshot without changing other snapshots' tolerances.

- Warn when widget builders or theme configuration values are discarded, with compile-fail tests for each builder.
- Lock dependencies for reproducible test and gallery builds.
- Add unit tests for slider conversions and form help text, and provide a default tab bar.

- Add stable text field IDs and password masking, including copy and accessibility handling.
- Make number field steps work for integers and reversed bounds, and clamp values when a step is applied.
- Add translatable names to number field step buttons.

- Keep segmented choices focused after clicks, support Home/End navigation in choices and tabs, and handle empty or out-of-range selections without overflow.
- Make integer slider keyboard steps work on short ranges, respect reversed ranges, and ignore invalid step sizes.
- Expose individual tabs with the tab accessibility role.
- Make the gallery scroll and stack its controls in narrow viewports, and choose a touch look on touch devices by default.
- Reserve space for form labels before the first measurement and after viewport resizing.
- Add accessible names to text fields, with form row labels as the default.
- Add keyboard navigation, focus restoration, scrolling and accessibility information to dropdown menus.
- Make dialogs modal, limit their size to the viewport and stack actions in narrow dialogs.
- Report web gallery loading failures and close the sample dialog from its action buttons.

## 0.1.0

First release on crates.io, for egui 0.37.
