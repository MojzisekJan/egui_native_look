//! UIKit command responder for alerts without a text editor.
//!
//! winit's UIKeyInput view relinquishes first responder when IME is disabled.
//! A zero-sized non-text responder receives Escape only while our alert is open.
use std::cell::Cell;

use eframe::egui;
use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly};
use objc2_foundation::{MainThreadMarker, NSArray, NSObjectProtocol};
use objc2_ui_kit::{UIKeyCommand, UIKeyInputEscape, UIKeyModifierFlags, UIView};

struct KeyState {
    pending: Cell<bool>,
    ctx: egui::Context,
    commands: Retained<NSArray<UIKeyCommand>>,
}

define_class!(
    // SAFETY: UIView has no extra subclassing requirements; this class has no Drop.
    #[unsafe(super = UIView)]
    #[thread_kind = MainThreadOnly]
    #[ivars = KeyState]
    struct DaybookAlertKeyView;

    // SAFETY: NSObjectProtocol has no additional requirements.
    unsafe impl NSObjectProtocol for DaybookAlertKeyView {}

    impl DaybookAlertKeyView {
        // SAFETY: These signatures match UIResponder's command/responder methods.
        #[unsafe(method(canBecomeFirstResponder))]
        fn can_become_first_responder(&self) -> bool { true }

        #[unsafe(method_id(keyCommands))]
        fn key_commands(&self) -> Option<Retained<NSArray<UIKeyCommand>>> {
            Some(self.ivars().commands.clone())
        }

        // SAFETY: UIKit sends this registered selector with its UIKeyCommand sender.
        #[unsafe(method(daybookCancelAlert:))]
        fn cancel_alert(&self, _sender: &UIKeyCommand) {
            self.ivars().pending.set(true);
            self.ivars().ctx.request_repaint();
        }
    }
);

pub(super) struct AlertKeys {
    view: Retained<DaybookAlertKeyView>,
}

impl AlertKeys {
    pub(super) fn attach(parent: &UIView, ctx: egui::Context) -> Self {
        let mtm = MainThreadMarker::new().expect("UIKit host runs on main thread");
        // SAFETY: UIKit exports this immutable key constant; the selector is implemented above.
        let command = unsafe {
            UIKeyCommand::keyCommandWithInput_modifierFlags_action(
                UIKeyInputEscape,
                UIKeyModifierFlags::empty(),
                sel!(daybookCancelAlert:),
                mtm,
            )
        };
        command.setWantsPriorityOverSystemBehavior(true);
        let allocated = DaybookAlertKeyView::alloc(mtm).set_ivars(KeyState {
            pending: Cell::new(false),
            ctx,
            commands: NSArray::from_retained_slice(&[command]),
        });
        // SAFETY: UIView's init has this signature and initializes a zero-sized view.
        let view: Retained<DaybookAlertKeyView> = unsafe { msg_send![super(allocated), init] };
        parent.addSubview(&view);
        Self { view }
    }

    pub(super) fn set_active(&self, active: bool) {
        if active {
            if !self.view.isFirstResponder() {
                self.view.becomeFirstResponder();
            }
        } else {
            self.view.ivars().pending.set(false);
            if self.view.isFirstResponder() {
                self.view.resignFirstResponder();
                // UIKit can restore the UIKeyInput parent as responder. No
                // editor belongs to this alert, so dismiss that inherited IME.
                if let Some(window) = self.view.window() {
                    window.endEditing(true);
                }
            }
        }
    }

    pub(super) fn take_escape(&self) -> bool {
        self.view.ivars().pending.replace(false)
    }
}
