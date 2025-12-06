use super::Modal;
use crate::utils::Theme;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PromptType {
    Initial,
    Review,
    Consensus,
}

impl PromptType {
    pub fn name(&self) -> &'static str {
        match self {
            PromptType::Initial => "Initial Response",
            PromptType::Review => "Review Feedback",
            PromptType::Consensus => "Consensus Synthesis",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PromptType::Initial => "Prompt sent to each model for their initial response",
            PromptType::Review => "Prompt sent to models to review other responses",
            PromptType::Consensus => "Prompt sent to synthesize final collaborative answer",
        }
    }

    pub fn variables(&self) -> &'static str {
        match self {
            PromptType::Initial => "{user_question}",
            PromptType::Review => "{user_question}, {other_responses}",
            PromptType::Consensus => "{user_question}, {initial_responses}, {reviews}",
        }
    }
}

#[component]
pub fn PromptEditorModal(
    theme: Signal<Theme>,
    open: Signal<bool>,
    prompt_type: PromptType,
    current_prompt: String,
    default_prompt: String,
    on_save: EventHandler<String>,
) -> Element {
    let _ = theme.read();
    let mut edited_prompt = use_signal(|| current_prompt.clone());

    // Clone for closures
    let current_prompt_clone = current_prompt.clone();
    let current_prompt_clone2 = current_prompt.clone();
    let current_prompt_clone3 = current_prompt.clone();
    let default_prompt_clone = default_prompt.clone();

    // Update edited_prompt when current_prompt changes
    use_effect(move || {
        edited_prompt.set(current_prompt_clone.clone());
    });

    rsx! {
        Modal {
            theme,
            open,
            on_close: move |_| {
                edited_prompt.set(current_prompt_clone3.clone());
                open.set(false);
            },

            // Modal content
            div {
                class: "p-6",

                // Header
                div {
                    class: "flex items-start justify-between mb-4",
                    div {
                        h2 {
                            class: "text-xl font-bold text-[var(--color-base-content)]",
                            "Edit {prompt_type.name()} Prompt"
                        }
                        p {
                            class: "text-sm text-[var(--color-base-content)]/70 mt-1",
                            "{prompt_type.description()}"
                        }
                    }
                    button {
                        class: "text-2xl text-[var(--color-base-content)]/70 hover:text-[var(--color-base-content)] transition-colors",
                        onclick: move |_| {
                            edited_prompt.set(current_prompt_clone2.clone());
                            open.set(false);
                        },
                        "×"
                    }
                }

                // Variable hints
                div {
                    class: "mb-4 p-3 bg-[var(--color-base-200)] rounded border border-[var(--color-base-300)]",
                    div {
                        class: "text-xs font-semibold text-[var(--color-base-content)] mb-1",
                        "Available Variables:"
                    }
                    div {
                        class: "text-xs text-[var(--color-base-content)]/70 font-mono",
                        "{prompt_type.variables()}"
                    }
                }

                // Prompt editor textarea
                div {
                    class: "mb-4",
                    textarea {
                        value: "{edited_prompt}",
                        oninput: move |evt| edited_prompt.set(evt.value()),
                        rows: "14",
                        class: "w-full p-3 border-2 rounded-lg font-mono text-sm bg-[var(--color-base-100)] text-[var(--color-base-content)] border-[var(--color-base-300)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent resize-y min-h-[300px]",
                        placeholder: "Enter your prompt template...",
                        autofocus: true,
                    }
                }

                // Character count
                div {
                    class: "text-xs text-[var(--color-base-content)]/50 mb-4 text-right",
                    "{edited_prompt.read().len()} characters"
                }

                // Action buttons
                div {
                    class: "flex justify-between items-center gap-3",
                    button {
                        onclick: move |_| {
                            edited_prompt.set(default_prompt_clone.clone());
                        },
                        class: "px-4 py-2 text-sm rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] text-[var(--color-base-content)] hover:bg-[var(--color-base-300)] transition-colors flex items-center gap-1.5",
                        img {
                            src: asset!("/assets/refresh.svg"),
                            class: "w-4 h-4",
                            alt: "Reset",
                        }
                        "Reset to Default"
                    }
                    div {
                        class: "flex gap-2",
                        button {
                            onclick: move |_| {
                                edited_prompt.set(current_prompt.clone());
                                open.set(false);
                            },
                            class: "px-4 py-2 text-sm rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] text-[var(--color-base-content)] hover:bg-[var(--color-base-300)] transition-colors",
                            "Cancel"
                        }
                        button {
                            onclick: move |_| {
                                on_save.call(edited_prompt());
                                open.set(false);
                            },
                            class: "px-4 py-2 text-sm rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 transition-colors font-medium",
                            "Save Template"
                        }
                    }
                }
            }
        }
    }
}
