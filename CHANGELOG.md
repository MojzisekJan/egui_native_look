# Changelog

## Unreleased

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
