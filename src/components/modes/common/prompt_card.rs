use crate::utils::Theme;
use dioxus::prelude::*;

#[component]
pub fn PromptCard(
    theme: Signal<Theme>,
    title: String,
    phase_number: u8,
    prompt: String,
    on_edit: EventHandler<()>,
) -> Element {
    let _ = theme.read();

    rsx! {
        div {
            class: "border border-[var(--color-base-300)] rounded-lg p-3 bg-[var(--color-base-200)] hover:border-[var(--color-primary)]/50 transition-all cursor-pointer",
            onclick: move |_| on_edit.call(()),

            div {
                class: "flex items-center justify-between mb-2",
                div {
                    class: "flex items-center gap-2",
                    span {
                        class: "inline-flex items-center justify-center w-6 h-6 rounded-full bg-[var(--color-primary)] text-[var(--color-primary-content)] text-xs font-bold",
                        "{phase_number}"
                    }
                    span {
                        class: "font-semibold text-sm text-[var(--color-base-content)]",
                        "{title}"
                    }
                }
                button {
                    class: "text-[var(--color-primary)] hover:opacity-70 text-sm transition-opacity flex items-center gap-1",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_edit.call(());
                    },
                    img {
                        src: asset!("/assets/edit.svg"),
                        class: "w-3.5 h-3.5",
                        alt: "Edit",
                    }
                    "Edit"
                }
            }

            // Prompt preview (first 100 chars)
            div {
                class: "text-xs text-[var(--color-base-content)]/70 line-clamp-2 font-mono break-words",
                "{prompt.chars().take(100).collect::<String>()}..."
            }
        }
    }
}
