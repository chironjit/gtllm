use crate::utils::{ChatMode, Settings, Theme};
use dioxus::prelude::*;

#[component]
pub fn NewChat(
    theme: Signal<Theme>,
    app_settings: Signal<Settings>,
    on_mode_select: EventHandler<ChatMode>,
    on_open_settings: EventHandler<()>,
) -> Element {
    let _ = theme.read();
    let has_api_key = app_settings.read().has_api_key();

    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full px-4 py-6",

            div {
                class: "max-w-3xl w-full",

                // Header
                div {
                    class: "text-center mb-6",
                    h1 {
                        class: "text-2xl font-bold text-[var(--color-base-content)] mb-2",
                        "Welcome to GTLLM"
                    }
                    p {
                        class: "text-sm text-[var(--color-base-content)]/80",
                        "Choose your chat mode"
                    }
                }

                // API Key Warning Banner
                if !has_api_key {
                    div {
                        class: "mb-4 p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/50",
                        div {
                            class: "flex items-center gap-2",
                            div {
                                class: "flex-shrink-0 text-lg",
                                "⚠️"
                            }
                            div {
                                class: "flex-1 min-w-0",
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/80",
                                    span {
                                        class: "font-semibold",
                                        "API Key Required: "
                                    }
                                    "Configure your OpenRouter API key in settings to use the app."
                                }
                            }
                            button {
                                onclick: move |_| on_open_settings.call(()),
                                class: "flex-shrink-0 px-3 py-1.5 rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 text-xs font-medium transition-all duration-200",
                                "Settings"
                            }
                        }
                    }
                }

                // Mode selection grid
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-3",

                    // Standard mode
                    button {
                        onclick: move |_| {
                            if has_api_key {
                                on_mode_select.call(ChatMode::Standard);
                            }
                        },
                        disabled: !has_api_key,
                        class: if has_api_key {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group"
                        } else {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] transition-all duration-200 text-left opacity-50 cursor-not-allowed"
                        },

                        div {
                            class: "flex items-start gap-3",
                            img {
                                src: asset!("/assets/message.svg"),
                                class: "w-8 h-8 group-hover:scale-110 transition-transform",
                                alt: "Standard"
                            }
                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-base font-bold text-[var(--color-base-content)] mb-1",
                                    "Standard"
                                }
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/70",
                                    "Traditional conversation with one or more LLMs"
                                }
                            }
                        }
                    }

                    // PvP mode
                    button {
                        onclick: move |_| {
                            if has_api_key {
                                on_mode_select.call(ChatMode::PvP);
                            }
                        },
                        disabled: !has_api_key,
                        class: if has_api_key {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group"
                        } else {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] transition-all duration-200 text-left opacity-50 cursor-not-allowed"
                        },

                        div {
                            class: "flex items-start gap-3",
                            img {
                                src: asset!("/assets/pvp.svg"),
                                class: "w-8 h-8 group-hover:scale-110 transition-transform",
                                alt: "PvP"
                            }
                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-base font-bold text-[var(--color-base-content)] mb-1",
                                    "PvP"
                                }
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/70",
                                    "2 LLMs compete, 1 LLM moderates"
                                }
                            }
                        }
                    }

                    // Collaborative mode
                    button {
                        onclick: move |_| {
                            if has_api_key {
                                on_mode_select.call(ChatMode::Collaborative);
                            }
                        },
                        disabled: !has_api_key,
                        class: if has_api_key {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group"
                        } else {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] transition-all duration-200 text-left opacity-50 cursor-not-allowed"
                        },

                        div {
                            class: "flex items-start gap-3",
                            img {
                                src: asset!("/assets/collaborative.svg"),
                                class: "w-8 h-8 group-hover:scale-110 transition-transform",
                                alt: "Collaborative"
                            }
                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-base font-bold text-[var(--color-base-content)] mb-1",
                                    "Collaborative"
                                }
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/70",
                                    "Multiple LLMs jointly agree on best solution"
                                }
                            }
                        }
                    }

                    // Competitive mode
                    button {
                        onclick: move |_| {
                            if has_api_key {
                                on_mode_select.call(ChatMode::Competitive);
                            }
                        },
                        disabled: !has_api_key,
                        class: if has_api_key {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group"
                        } else {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] transition-all duration-200 text-left opacity-50 cursor-not-allowed"
                        },

                        div {
                            class: "flex items-start gap-3",
                            img {
                                src: asset!("/assets/competitive.svg"),
                                class: "w-8 h-8 group-hover:scale-110 transition-transform",
                                alt: "Competitive"
                            }
                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-base font-bold text-[var(--color-base-content)] mb-1",
                                    "Competitive"
                                }
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/70",
                                    "All LLMs vote for the best answer"
                                }
                            }
                        }
                    }

                    // Choice mode
                    button {
                        onclick: move |_| {
                            if has_api_key {
                                on_mode_select.call(ChatMode::LLMChoice);
                            }
                        },
                        disabled: !has_api_key,
                        class: if has_api_key {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group md:col-span-2"
                        } else {
                            "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] transition-all duration-200 text-left opacity-50 cursor-not-allowed md:col-span-2"
                        },

                        div {
                            class: "flex items-start gap-3",
                            img {
                                src: asset!("/assets/choice.svg"),
                                class: "w-8 h-8 group-hover:scale-110 transition-transform",
                                alt: "LLM's Choice"
                            }
                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-base font-bold text-[var(--color-base-content)] mb-1",
                                    "LLM's Choice"
                                }
                                p {
                                    class: "text-xs text-[var(--color-base-content)]/70",
                                    "LLMs decide to collaborate or compete - let the LLMs choose its own strategy"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
