use crate::utils::{ChatSession, Theme};
use dioxus::prelude::*;

#[component]
pub fn Sidebar(
    theme: Signal<Theme>,
    sessions: Signal<Vec<ChatSession>>,
    current_session: Signal<Option<usize>>,
    on_new_chat: EventHandler<()>,
    on_select_session: EventHandler<usize>,
) -> Element {
    let _ = theme.read();

    rsx! {
        div {
            class: "w-64 shrink-0 bg-[var(--color-base-200)] border-r border-[var(--color-base-300)] p-4 flex flex-col h-screen overflow-y-auto",

            // New Chat button
            div {
                class: "mb-4",
                button {
                    onclick: move |_| on_new_chat.call(()),
                    class: "w-full flex items-center justify-center gap-1.5 px-3 py-2 bg-[var(--color-primary)] hover:bg-[var(--color-primary)]/90 text-[var(--color-primary-content)] rounded-lg font-medium text-sm transition-all duration-200 shadow-sm",
                    img {
                        src: asset!("/assets/new_chat.svg"),
                        class: "w-4 h-4",
                        alt: "New Chat"
                    }
                    span { "New Chat" }
                }
            }

            // Chat history section
            div {
                class: "flex-1 overflow-y-auto",

                h3 {
                    class: "text-xs uppercase text-[var(--color-base-content)]/60 font-semibold px-3 mb-3",
                    "Recent Chats"
                }

                div {
                    class: "space-y-1",

                    if sessions.read().is_empty() {
                        div {
                            class: "px-3 py-8 text-center",
                            div {
                                class: "text-4xl mb-2 opacity-50",
                                "💭"
                            }
                            p {
                                class: "text-xs text-[var(--color-base-content)]/60",
                                "No conversations yet"
                            }
                        }
                    } else {
                        for session in sessions.read().iter() {
                            {
                                let is_active = current_session.read().as_ref() == Some(&session.id);
                                let session_id = session.id;

                                rsx! {
                                    button {
                                        key: "{session.id}",
                                        onclick: move |_| on_select_session.call(session_id),
                                        class: "w-full text-left px-3 py-2.5 rounded-lg transition-all duration-200",
                                        class: if is_active {
                                            "bg-[var(--color-primary)]/10 border border-[var(--color-primary)]"
                                        } else {
                                            "hover:bg-[var(--color-base-300)]/50"
                                        },

                                        div {
                                            class: "flex items-center gap-3",

                                            // Mode icon
                                            div {
                                                class: "shrink-0",
                                                match session.mode.name() {
                                                    "Standard" => rsx! {
                                                        img { src: asset!("/assets/message.svg"), class: "w-4 h-4", alt: "Standard" }
                                                    },
                                                    "PvP" => rsx! {
                                                        img { src: asset!("/assets/pvp.svg"), class: "w-4 h-4", alt: "PvP" }
                                                    },
                                                    "Collaborative" => rsx! {
                                                        img { src: asset!("/assets/collaborative.svg"), class: "w-4 h-4", alt: "Collaborative" }
                                                    },
                                                    "Competitive" => rsx! {
                                                        img { src: asset!("/assets/competitive.svg"), class: "w-4 h-4", alt: "Competitive" }
                                                    },
                                                    "LLM's Choice" => rsx! {
                                                        img { src: asset!("/assets/choice.svg"), class: "w-4 h-4", alt: "Choice" }
                                                    },
                                                    _ => rsx! {
                                                        img { src: asset!("/assets/message.svg"), class: "w-4 h-4", alt: "Chat" }
                                                    },
                                                }
                                            }

                                            // Session info
                                            div {
                                                class: "flex-1 min-w-0",
                                                div {
                                                    class: "text-sm font-medium text-[var(--color-base-content)] truncate",
                                                    "{session.title}"
                                                }
                                                div {
                                                    class: "text-xs text-[var(--color-base-content)]/60 mt-0.5",
                                                    "{session.timestamp}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
