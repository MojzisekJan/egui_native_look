# Current Apple follow-up, 2026-11-18

This evidence supersedes earlier follow-up measurements for the modified controls; it does not certify complete SwiftUI parity.

The current changes fix point-size-dependent San Francisco tracking through the local epaint patch, platform-specific default-label weights, outward pixel rounding, focused macOS switch/segment Return routing, iOS slider shadow, constant iOS menu-picker height and Apple progress layout heights. See vendor/README.md for the required downstream workspace patch.

Native input was repeated with an active key window checked throughout every stage. Earlier traces that only checked activity at final capture were insufficient: an inactive window can ignore Space or fail to open a menu. The corrected traces pass eight normal-input and seven keyboard assertions. Native animation settling is allowed before state inspection.

All 12 existing native visual captures were retained and egui was rerendered using each native display scale. The iOS simulator and its data were not removed or rebuilt in this continuation. Native iOS host event delivery remains unverified. Current Rust checks: 362 tests including doctests, four snapshot groups, 356 tests without system integration, host/iOS clippy, format and whitespace checks all passed.

Regular Save/Continue widths match the references on both platforms. iOS picker layout is 103/3 points at small, regular and large sizes; progress layout is 4 points on iOS and 12/20/20 on macOS. egui's subpixel quantization leaves a 0.021-point picker-height difference. Small iOS Save remains 1/3 point narrower.

Outstanding: UIKit editing semantics and host integration, physical-device and VoiceOver verification, animation/outline/rasterization differences, and intrinsic child sizing within fixed SwiftUI frames. Checkbox versus iOS Toggle and editable NumberField versus Stepper are distinct semantic controls, not evidence of equivalent widgets. SecureField capture suppression must not be mistaken for an empty field value.
