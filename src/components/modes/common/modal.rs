use crate::utils::Theme;
use dioxus::prelude::*;

#[component]
pub fn Modal(
    theme: Signal<Theme>,
    open: Signal<bool>,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    let _ = theme.read();

    if !open() {
        return rsx! {};
    }

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/50 z-[9998] flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),

            // Modal content wrapper - stop propagation so clicking inside doesn't close
            div {
                class: "bg-[var(--color-base-100)] rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] overflow-auto relative",
                onclick: move |e| e.stop_propagation(),

                {children}
            }
        }
    }
}
