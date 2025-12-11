use dioxus::prelude::*;

#[component]
pub fn StreamingIndicator(is_streaming: bool) -> Element {
    if !is_streaming {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            class: "flex items-center gap-1",
            div {
                class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
            }
            div {
                class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
            }
            div {
                class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
            }
        }
    }
}
