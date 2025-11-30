use crate::utils::{ChatMode, Theme};
use dioxus::prelude::*;

#[component]
pub fn NewChat(theme: Signal<Theme>, on_mode_select: EventHandler<ChatMode>) -> Element {
    let _ = theme.read();

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
                        "Game Theory meets Large Language Models - Choose your chat mode"
                    }
                }

                // Mode selection grid
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-3",

                    // Standard mode
                    button {
                        onclick: move |_| on_mode_select.call(ChatMode::Standard),
                        class: "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group",

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
                                    "Single LLM chat - Traditional one-on-one conversation with an AI assistant"
                                }
                            }
                        }
                    }

                    // PvP mode
                    button {
                        onclick: move |_| on_mode_select.call(ChatMode::PvP),
                        class: "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group",

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
                                    "2 bots compete, 1 moderator judges - Watch AI models battle it out"
                                }
                            }
                        }
                    }

                    // Collaborative mode
                    button {
                        onclick: move |_| on_mode_select.call(ChatMode::Collaborative),
                        class: "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group",

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
                                    "Multiple bots jointly agree on best solution - Teamwork makes the dream work"
                                }
                            }
                        }
                    }

                    // Competitive mode
                    button {
                        onclick: move |_| on_mode_select.call(ChatMode::Competitive),
                        class: "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group",

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
                                    "All bots vote for the best (can't vote for their own) - Democracy in action"
                                }
                            }
                        }
                    }

                    // Choice mode
                    button {
                        onclick: move |_| on_mode_select.call(ChatMode::LLMChoice),
                        class: "p-4 rounded-lg bg-[var(--color-base-200)] border-2 border-[var(--color-base-300)] hover:border-[var(--color-primary)] transition-all duration-200 text-left group md:col-span-2",

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
                                    "LLMs decide to collaborate or compete - Let the AI choose its own strategy"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
