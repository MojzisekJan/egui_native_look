# Apple parity: integration boundary

The library remains host-independent. A Bevy integration is a candidate, not a tested shipping iOS host.

Inspected 2026-11-18: upstream bevy_egui main declares version 0.43.2, Bevy 0.20 and egui 0.36. This checkout requires egui 0.37. These versions cannot share egui Context/Ui values. A matched bevy_egui revision or a tested port is required before an executable integration can be claimed. Adding two egui versions does not solve this.

Sources:
- https://github.com/vladbat00/bevy_egui/blob/main/Cargo.toml
- https://github.com/vladbat00/bevy_egui/blob/main/src/input.rs

Upstream process_ime_system enables IME and updates its rectangle through winit; write_ime_messages_system translates preedit and commit. This establishes an existing transport to reuse. It does not establish native selection handles, magnifier, autofill or UIKit editor semantics.

Integration acceptance requires matching dependency versions, host window/context mapping, physical device keyboard tests, secure input purpose, correct preedit ranges and composition interruption, safe-area/keyboard insets, and VoiceOver checks. Current library tests verify egui output and input contracts; they do not certify this host path.

Further fixes in this follow-up: nested window action scopes propagate consumed Enter and blocked composition state; buttons recheck popup state at activation. Native label measurements establish iOS button horizontal insets of 10/12/20 points (small/regular/large). Small button labels use 15-point type while normal fields stay at 17.
