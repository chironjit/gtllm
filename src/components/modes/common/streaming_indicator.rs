use dioxus::prelude::*;

#[component]
pub fn StreamingIndicator(is_streaming: bool) -> Element {
    if !is_streaming {
        return rsx! {};
    }

    rsx! {
        div {
            class: "flex items-center gap-1",
            div {
                class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse",
            }
            div {
                class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse",
            }
            div {
                class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse",
            }
        }
    }
}

/// Shown immediately when streaming starts, before any tokens arrive.
/// Provides visible feedback that the LLM request is in-flight.
#[component]
pub fn ThinkingIndicator() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2 py-2",

            div {
                class: "flex gap-1",
                div {
                    class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse",
                }
                div {
                    class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse",
                    style: "animation-delay: 200ms",
                }
                div {
                    class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse",
                    style: "animation-delay: 400ms",
                }
            }

            span {
                class: "text-sm text-[var(--color-base-content)]/60",
                "Thinking..."
            }
        }
    }
}
