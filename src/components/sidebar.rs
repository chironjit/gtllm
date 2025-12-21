use crate::utils::{ChatHistory, ChatSession, Theme};
use dioxus::prelude::*;

#[component]
pub fn Sidebar(
    theme: Signal<Theme>,
    sessions: Signal<Vec<ChatSession>>,
    current_session: Signal<Option<String>>,
    collapsed: Signal<bool>,
    on_new_chat: EventHandler<()>,
    on_select_session: EventHandler<String>,
) -> Element {
    let _ = theme.read();
    let is_collapsed = *collapsed.read();

    let toggle_collapse = move |_| {
        let current = *collapsed.read();
        collapsed.set(!current);
    };

    rsx! {
        div {
            class: "shrink-0 bg-[var(--color-base-200)] border-r border-[var(--color-base-300)] flex flex-col h-screen transition-all duration-300",
            class: if is_collapsed { "w-20" } else { "w-64" },

            // Content area with padding
            div {
                class: "flex-1 overflow-y-auto p-4 flex flex-col",

                // New Chat button
                div {
                    class: "mb-4",
                    button {
                        onclick: move |_| on_new_chat.call(()),
                        class: "w-full flex items-center justify-center bg-[var(--color-primary)] hover:bg-[var(--color-primary)]/90 text-[var(--color-primary-content)] rounded-lg font-medium text-sm transition-all duration-200 shadow-sm",
                        class: if is_collapsed { "p-2.5" } else { "gap-1.5 px-3 py-2" },
                        title: if is_collapsed { "New Chat" } else { "" },

                        img {
                            src: asset!("/assets/new_chat.svg"),
                            class: "w-4 h-4",
                            alt: "New Chat"
                        }
                        if !is_collapsed {
                            span { "New Chat" }
                        }
                    }
                }

                // Chat history section
                div {
                    class: "flex-1 overflow-y-auto",

                    if !is_collapsed {
                        h3 {
                            class: "text-xs uppercase text-[var(--color-base-content)]/60 font-semibold px-3 mb-3",
                            "Recent Chats"
                        }
                    }

                    div {
                        class: "space-y-1",

                        if sessions.read().is_empty() {
                            if !is_collapsed {
                                div {
                                    class: "px-3 py-6 text-center flex flex-col items-center",
                                    img {
                                        src: asset!("/assets/no_chats.svg"),
                                        class: "w-12 h-12 mb-2 opacity-50",
                                        alt: "No chats"
                                    }
                                    p {
                                        class: "text-xs text-[var(--color-base-content)]/60",
                                        "No conversations yet"
                                    }
                                }
                            }
                        } else {
                            for session in sessions.read().iter() {
                                {
                                    let is_active = current_session.read().as_ref() == Some(&session.id);
                                    let session_id_for_click = session.id.clone();

                                    rsx! {
                                        button {
                                            key: "{session.id}",
                                            onclick: move |_| {
                                                let sid = session_id_for_click.clone();
                                                on_select_session.call(sid);
                                            },
                                            class: "w-full rounded-lg transition-all duration-200",
                                            class: if is_collapsed {
                                                "p-2 flex items-center justify-center"
                                            } else {
                                                "text-left px-2.5 py-2"
                                            },
                                            class: if is_active {
                                                "bg-[var(--color-primary)]/10 border border-[var(--color-primary)]"
                                            } else {
                                                "hover:bg-[var(--color-base-300)]/50"
                                            },
                                            title: if is_collapsed { "{session.title}" } else { "" },

                                            if is_collapsed {
                                                // Collapsed view: just the icon
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
                                            } else {
                                                // Expanded view: full layout
                                                div {
                                                    class: "flex items-center gap-2.5",

                                                    // Mode icon
                                                    div {
                                                        class: "shrink-0",
                                                        match session.mode.name() {
                                                            "Standard" => rsx! {
                                                                img { src: asset!("/assets/message.svg"), class: "w-3.5 h-3.5", alt: "Standard" }
                                                            },
                                                            "PvP" => rsx! {
                                                                img { src: asset!("/assets/pvp.svg"), class: "w-3.5 h-3.5", alt: "PvP" }
                                                            },
                                                            "Collaborative" => rsx! {
                                                                img { src: asset!("/assets/collaborative.svg"), class: "w-3.5 h-3.5", alt: "Collaborative" }
                                                            },
                                                            "Competitive" => rsx! {
                                                                img { src: asset!("/assets/competitive.svg"), class: "w-3.5 h-3.5", alt: "Competitive" }
                                                            },
                                                            "LLM's Choice" => rsx! {
                                                                img { src: asset!("/assets/choice.svg"), class: "w-3.5 h-3.5", alt: "Choice" }
                                                            },
                                                            _ => rsx! {
                                                                img { src: asset!("/assets/message.svg"), class: "w-3.5 h-3.5", alt: "Chat" }
                                                            },
                                                        }
                                                    }

                                                    // Session info
                                                    div {
                                                        class: "flex-1 min-w-0",
                                                        div {
                                                            class: "text-xs font-medium text-[var(--color-base-content)] truncate",
                                                            "{session.title}"
                                                        }
                                                        div {
                                                            class: "text-[10px] text-[var(--color-base-content)]/50 mt-0.5",
                                                            "{session.mode.name()} • {ChatHistory::format_timestamp_date(&session.timestamp)}"
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

            // Collapse/Expand button at the bottom
            div {
                class: "p-4 border-t border-[var(--color-base-300)]",
                button {
                    onclick: toggle_collapse,
                    class: "w-full flex items-center justify-center bg-[var(--color-base-300)] hover:bg-[var(--color-base-300)]/80 text-[var(--color-base-content)] rounded-lg transition-all duration-200",
                    class: if is_collapsed { "p-2.5" } else { "gap-2 px-3 py-2" },
                    title: if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },

                    if is_collapsed {
                        img {
                            src: asset!("/assets/close_panel.svg"),
                            class: "w-4 h-4",
                            alt: "Expand"
                        }
                    } else {
                        img {
                            src: asset!("/assets/open_panel.svg"),
                            class: "w-4 h-4",
                            alt: "Collapse"
                        }
                        span {
                            class: "text-sm font-medium",
                            "Collapse"
                        }
                    }
                }
            }
        }
    }
}
