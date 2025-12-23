use crate::utils::{parse_message_content, parse_inline_elements, ContentSegment, InlineSegment, Message, Theme};
use dioxus::prelude::*;

fn get_user_avatar() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect fill='%2306b6d4' width='40' height='40' rx='20'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='20'%3E=d%3C/text%3E%3C/svg%3E"
}

fn get_assistant_avatar() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect fill='%236366f1' width='40' height='40' rx='20'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='20'%3E>%3C/text%3E%3C/svg%3E"
}

#[component]
fn InlineMarkdown(theme: Signal<Theme>, text: String) -> Element {
    let _theme_val = theme.read();
    let inline_segments = parse_inline_elements(&text);

    rsx! {
        for segment in inline_segments {
            match segment {
                InlineSegment::Text(t) => {
                    rsx! {
                        span { "{t}" }
                    }
                },
                InlineSegment::Bold(t) => {
                    let bold_segments = parse_inline_elements(&t);
                    rsx! {
                        strong {
                            class: "font-bold",
                            // Parse inline markdown inside bold text (e.g., links)
                            for segment in bold_segments {
                                match segment {
                                    InlineSegment::Text(text) => {
                                        rsx! { span { "{text}" } }
                                    },
                                    InlineSegment::Link { text: link_text, url } => {
                                        rsx! {
                                            a {
                                                href: "{url}",
                                                target: "_blank",
                                                rel: "noopener noreferrer",
                                                class: "text-[var(--color-primary)] hover:underline",
                                                "{link_text}"
                                            }
                                        }
                                    },
                                    InlineSegment::InlineCode(code) => {
                                        rsx! {
                                            code {
                                                class: "px-2 py-1 rounded bg-[var(--color-base-300)] text-[var(--color-base-content)] font-mono text-sm",
                                                "{code}"
                                            }
                                        }
                                    },
                                    // Don't nest bold/italic inside bold - just render as text
                                    InlineSegment::Bold(_) | InlineSegment::Italic(_) => {
                                        rsx! { span { "{t}" } }
                                    },
                                }
                            }
                        }
                    }
                },
                InlineSegment::Italic(t) => {
                    rsx! {
                        em {
                            class: "italic",
                            "{t}"
                        }
                    }
                },
                InlineSegment::Link { text: t, url: u } => {
                    rsx! {
                        a {
                            href: "{u}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "text-[var(--color-primary)] hover:underline",
                            "{t}"
                        }
                    }
                },
                InlineSegment::InlineCode(c) => {
                    rsx! {
                        code {
                            class: "px-2 py-1 rounded bg-[var(--color-base-300)] text-[var(--color-base-content)] font-mono text-sm",
                            "{c}"
                        }
                    }
                },
            }
        }
    }
}

#[component]
pub fn FormattedText(theme: Signal<Theme>, content: String) -> Element {
    let _theme_val = theme.read();
    let segments = parse_message_content(&content);
    
    // Group consecutive list items together
    let mut grouped_segments: Vec<ContentSegment> = Vec::new();
    let mut current_list: Vec<String> = Vec::new();
    
    for segment in segments {
        match segment {
            ContentSegment::ListItem(text) => {
                current_list.push(text);
            },
            other => {
                // If we have accumulated list items, add them as a group
                if !current_list.is_empty() {
                    // Use a special marker to indicate this is a grouped list
                    // We'll use a character that won't appear in normal text
                    let list_items = current_list.drain(..).collect::<Vec<_>>();
                    grouped_segments.push(ContentSegment::ListItem(format!("\u{0001}LIST_GROUP\u{0001}{}", list_items.join("\u{0001}ITEM\u{0001}"))));
                }
                grouped_segments.push(other);
            }
        }
    }
    // Handle list items at the end
    if !current_list.is_empty() {
        let list_items = current_list.drain(..).collect::<Vec<_>>();
        grouped_segments.push(ContentSegment::ListItem(format!("\u{0001}LIST_GROUP\u{0001}{}", list_items.join("\u{0001}ITEM\u{0001}"))));
    }

    rsx! {
        div {
            class: "space-y-2 prose prose-sm max-w-none",
            for segment in grouped_segments {
                match segment {
                    ContentSegment::Text(text) => {
                        rsx! {
                            p {
                                class: "whitespace-pre-wrap",
                                InlineMarkdown {
                                    theme,
                                    text: text.clone(),
                                }
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
                                class: "rounded-lg overflow-hidden my-4",
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
                                    class: "bg-[var(--color-base-300)] text-[var(--color-base-content)] p-4 overflow-x-auto m-0",
                                    code {
                                        class: "font-mono text-sm leading-relaxed",
                                        "{code}"
                                    }
                                }
                            }
                        }
                    },
                    ContentSegment::Header { level, text } => {
                        let header_class = match level {
                            1 => "text-2xl font-bold mt-6 mb-4",
                            2 => "text-xl font-bold mt-5 mb-3",
                            3 => "text-lg font-bold mt-4 mb-2",
                            4 => "text-base font-bold mt-3 mb-2",
                            5 => "text-sm font-bold mt-2 mb-1",
                            _ => "text-sm font-bold mt-2 mb-1",
                        };
                        rsx! {
                            match level {
                                1 => rsx! { h1 { class: "{header_class} text-[var(--color-base-content)]", InlineMarkdown { theme, text: text.clone() } } },
                                2 => rsx! { h2 { class: "{header_class} text-[var(--color-base-content)]", InlineMarkdown { theme, text: text.clone() } } },
                                3 => rsx! { h3 { class: "{header_class} text-[var(--color-base-content)]", InlineMarkdown { theme, text: text.clone() } } },
                                4 => rsx! { h4 { class: "{header_class} text-[var(--color-base-content)]", InlineMarkdown { theme, text: text.clone() } } },
                                5 => rsx! { h5 { class: "{header_class} text-[var(--color-base-content)]", InlineMarkdown { theme, text: text.clone() } } },
                                _ => rsx! { h6 { class: "{header_class} text-[var(--color-base-content)]", InlineMarkdown { theme, text: text.clone() } } },
                            }
                        }
                    },
                    ContentSegment::ListItem(text) => {
                        // Check if this is a grouped list (contains our special marker)
                        if text.starts_with("\u{0001}LIST_GROUP\u{0001}") {
                            let items_str = text.strip_prefix("\u{0001}LIST_GROUP\u{0001}").unwrap_or("");
                            let items: Vec<&str> = items_str.split("\u{0001}ITEM\u{0001}").collect();
                            rsx! {
                                ul {
                                    class: "list-disc ml-6 my-2 space-y-1",
                                    for item in items {
                                        if !item.is_empty() {
                                            li {
                                                InlineMarkdown {
                                                    theme,
                                                    text: item.to_string(),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Single list item - wrap in ul for proper rendering
                            rsx! {
                                ul {
                                    class: "list-disc ml-6 my-2",
                                    li {
                                        InlineMarkdown {
                                            theme,
                                            text: text.clone(),
                                        }
                                    }
                                }
                            }
                        }
                    },
                    ContentSegment::Blockquote(text) => {
                        rsx! {
                            blockquote {
                                class: "border-l-4 border-[var(--color-base-300)] pl-4 py-2 my-2 italic text-[var(--color-base-content)]/80",
                                InlineMarkdown {
                                    theme,
                                    text: text.clone(),
                                }
                            }
                        }
                    },
                    ContentSegment::HorizontalRule => {
                        rsx! {
                            hr {
                                class: "border-t border-[var(--color-base-300)] my-4",
                            }
                        }
                    },
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
