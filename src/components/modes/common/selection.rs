use crate::utils::Theme;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct LLMOption {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[component]
pub fn LLMSelection(
    theme: Signal<Theme>,
    available_llms: Vec<LLMOption>,
    selected_llms: Signal<Vec<String>>,
    max_selection: Option<usize>,
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let _ = theme.read();

    let mut toggle_llm = move |llm_id: String| {
        let mut current = selected_llms.read().clone();
        if let Some(pos) = current.iter().position(|id| id == &llm_id) {
            current.remove(pos);
        } else {
            if let Some(max) = max_selection {
                if current.len() >= max {
                    return;
                }
            }
            current.push(llm_id);
        }
        selected_llms.set(current.clone());
        on_change.call(current);
    };

    rsx! {
        div {
            class: "space-y-3",

            h3 {
                class: "text-sm font-semibold text-[var(--color-base-content)] mb-3",
                "Select LLMs"
                if let Some(max) = max_selection {
                    span {
                        class: "text-[var(--color-base-content)]/70 font-normal ml-2",
                        "({selected_llms.read().len()}/{max})"
                    }
                }
            }

            div {
                class: "grid grid-cols-1 sm:grid-cols-2 gap-3",

                for llm in available_llms.iter() {
                    {
                        let is_selected = selected_llms.read().contains(&llm.id);
                        let llm_id = llm.id.clone();
                        let is_disabled = if let Some(max) = max_selection {
                            !is_selected && selected_llms.read().len() >= max
                        } else {
                            false
                        };

                        rsx! {
                            button {
                                key: "{llm.id}",
                                onclick: move |_| if !is_disabled { toggle_llm(llm_id.clone()) },
                                disabled: is_disabled,
                                class: "p-3 rounded-lg border-2 transition-all duration-200 text-left",
                                class: if is_selected {
                                    "bg-[var(--color-primary)]/10 border-[var(--color-primary)] text-[var(--color-base-content)]"
                                } else if is_disabled {
                                    "bg-[var(--color-base-200)] border-[var(--color-base-300)] text-[var(--color-base-content)]/50 cursor-not-allowed"
                                } else {
                                    "bg-[var(--color-base-200)] border-[var(--color-base-300)] text-[var(--color-base-content)] hover:border-[var(--color-primary)]/50"
                                },

                                div {
                                    class: "flex items-center justify-between mb-1",
                                    span {
                                        class: "font-semibold text-sm",
                                        "{llm.name}"
                                    }
                                    if is_selected {
                                        span {
                                            class: "text-[var(--color-primary)]",
                                            ""
                                        }
                                    }
                                }
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/70",
                                    "{llm.description}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
