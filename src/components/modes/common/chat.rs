use crate::utils::{parse_message_content, ContentSegment, Message, Theme};
use dioxus::prelude::*;

fn get_user_avatar() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect fill='%2306b6d4' width='40' height='40' rx='20'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='20'%3E=d%3C/text%3E%3C/svg%3E"
}

fn get_assistant_avatar() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect fill='%236366f1' width='40' height='40' rx='20'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='20'%3E>%3C/text%3E%3C/svg%3E"
}

#[component]
fn FormattedText(theme: Signal<Theme>, content: String) -> Element {
    let _theme_val = theme.read();
    let segments = parse_message_content(&content);

    rsx! {
        div {
            class: "space-y-2",
            for segment in segments {
                match segment {
                    ContentSegment::Text(text) => {
                        rsx! {
                            span {
                                class: "whitespace-pre-wrap",
                                "{text}"
                            }
                        }
                    },
                    ContentSegment::InlineCode(code) => {
                        rsx! {
                            code {
                                class: "px-2 py-1 rounded bg-[var(--color-base-300)] text-[var(--color-base-content)] font-mono text-sm",
                                "{code}"
                            }
                        }
                    },
                    ContentSegment::CodeBlock { language, code } => {
                        let code_clone = code.clone();
                        rsx! {
                            div {
                                class: "rounded-lg overflow-hidden",
                                if !language.is_empty() {
                                    div {
                                        class: "bg-[var(--color-base-300)] px-4 py-2 text-xs font-semibold text-[var(--color-base-content)]/70 flex items-center justify-between",
                                        span {
                                            "{language}"
                                        }
                                        button {
                                            onclick: move |_| {
                                                let _ = code_clone.clone();
                                            },
                                            class: "hover:opacity-70 transition-opacity",
                                            "📋"
                                        }
                                    }
                                }
                                pre {
                                    class: "bg-[var(--color-base-300)] text-[var(--color-base-content)] p-4 overflow-x-auto",
                                    code {
                                        class: "font-mono text-sm leading-relaxed",
                                        "{code}"
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

#[component]
pub fn ChatDisplay(
    theme: Signal<Theme>,
    messages: Signal<Vec<Message>>,
    empty_title: String,
    empty_description: String,
) -> Element {
    let _theme_val = theme.read();

    rsx! {
        div {
            class: "grow min-h-0 overflow-y-auto px-4 sm:px-6 md:px-5 py-6",

            if messages.read().is_empty() {
                div {
                    class: "flex flex-col items-center justify-center h-full max-w-2xl mx-auto text-center",

                    div {
                        class: "text-6xl mb-6",
                        "🎮"
                    }

                    h2 {
                        class: "text-2xl font-bold text-[var(--color-base-content)] mb-3",
                        "{empty_title}"
                    }

                    p {
                        class: "text-[var(--color-base-content)]/80 mb-6 text-lg",
                        "{empty_description}"
                    }

                    div {
                        class: "inline-flex items-center px-4 py-2 bg-[var(--color-base-200)] rounded-lg shadow-sm border border-[var(--color-base-300)]",
                        div {
                            class: "text-sm font-medium text-[var(--color-base-content)]",
                            "Start chatting below"
                        }
                    }
                }
            } else {
                div {
                    class: "space-y-4",

                    for message in messages.read().iter() {
                        {
                            let is_user = message.is_user;

                            rsx! {
                                div {
                                    key: "{message.id}",
                                    class: "flex items-start mb-4",

                                    if !is_user {
                                        img {
                                            class: "rounded-full mr-4",
                                            src: "{get_assistant_avatar()}",
                                            width: "40",
                                            height: "40",
                                            alt: "{message.sender}"
                                        }
                                    }

                                    div {
                                        class: "flex-1",

                                        if is_user {
                                            div {
                                                class: "flex justify-end",
                                                div {
                                                    class: "max-w-2xl",
                                                    div {
                                                        class: "text-sm bg-[var(--color-primary)] text-[var(--color-primary-content)] p-3 rounded-lg rounded-tr-none border border-transparent mb-1",
                                                        FormattedText {
                                                            theme,
                                                            content: message.content.clone(),
                                                        }
                                                    }
                                                    div {
                                                        class: "flex items-center justify-end",
                                                        div {
                                                            class: "text-xs text-[var(--color-base-content)]/70 font-medium",
                                                            "{message.timestamp}"
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            div {
                                                div {
                                                    class: "text-sm bg-[var(--color-base-300)] text-[var(--color-base-content)] p-3 rounded-lg rounded-tl-none mb-1",
                                                    div {
                                                        class: "font-semibold text-[var(--color-base-content)] mb-1",
                                                        "{message.sender}"
                                                    }
                                                    FormattedText {
                                                        theme,
                                                        content: message.content.clone(),
                                                    }
                                                }
                                                div {
                                                    class: "flex items-center justify-between",
                                                    div {
                                                        class: "text-xs text-[var(--color-base-content)]/70 font-medium",
                                                        "{message.timestamp}"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if is_user {
                                        img {
                                            class: "rounded-full ml-4",
                                            src: "{get_user_avatar()}",
                                            width: "40",
                                            height: "40",
                                            alt: "User"
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
