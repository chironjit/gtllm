use crate::utils::Theme;
use dioxus::prelude::*;

#[component]
pub fn ModelResponseCard(
    theme: Signal<Theme>,
    model_id: String,
    content: String,
    error_message: Option<String>,
    is_streaming: bool,
) -> Element {
    let border_color = if error_message.is_some() {
        "border-red-500"
    } else {
        "border-[var(--color-base-300)]"
    };

    rsx! {
        div {
            class: "p-4 rounded-lg border-2 {border_color} bg-[var(--color-base-100)]",

            // Model ID header
            div {
                class: "flex items-center justify-between mb-2",
                div {
                    class: "font-mono text-sm font-semibold text-[var(--color-primary)]",
                    "{model_id}"
                }
                if is_streaming {
                    div {
                        class: "flex gap-1",
                        div { class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse" }
                        div { class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse animation-delay-200" }
                        div { class: "w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse animation-delay-400" }
                    }
                }
            }

            // Content or error
            if let Some(error) = error_message {
                div {
                    class: "text-sm text-red-600 font-medium",
                    "Error: {error}"
                }
            } else {
                div {
                    class: "text-sm text-[var(--color-base-content)] break-words",
                    super::FormattedText {
                        theme,
                        content: content.clone(),
                    }
                }
            }
        }
    }
}
