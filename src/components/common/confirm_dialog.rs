use crate::components::modes::Modal;
use crate::utils::Theme;
use dioxus::prelude::*;

#[component]
pub fn ConfirmDialog(
    theme: Signal<Theme>,
    open: Signal<bool>,
    title: String,
    message: String,
    confirm_label: String,
    confirm_danger: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut open_signal = open;
    let confirm_button_class = if confirm_danger {
        "px-4 py-2 rounded-lg text-sm font-medium bg-red-600 text-white hover:bg-red-700 transition-colors"
    } else {
        "px-4 py-2 rounded-lg text-sm font-medium bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 transition-colors"
    };

    rsx! {
        Modal {
            theme,
            open,
            on_close: move |_| on_cancel.call(()),

            div {
                class: "p-6",

                h3 {
                    class: "text-lg font-semibold text-[var(--color-base-content)] mb-2",
                    "{title}"
                }

                p {
                    class: "text-sm text-[var(--color-base-content)]/70 mb-6",
                    "{message}"
                }

                div {
                    class: "flex justify-end gap-3",

                    button {
                        class: "px-4 py-2 rounded-lg text-sm font-medium bg-[var(--color-base-300)] text-[var(--color-base-content)] hover:bg-[var(--color-base-300)]/80 transition-colors",
                        onclick: move |_| {
                            open_signal.set(false);
                            on_cancel.call(());
                        },
                        "Cancel"
                    }

                    button {
                        class: confirm_button_class,
                        onclick: move |_| {
                            open_signal.set(false);
                            on_confirm.call(());
                        },
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}
