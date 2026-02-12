use crate::utils::{parse_message_content, parse_inline_elements, ContentSegment, InlineSegment, Message, Theme};
use dioxus::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

const PARSED_CACHE_MAX_ENTRIES: usize = 256;

#[derive(Default)]
struct ParsedContentCache {
    order: VecDeque<u64>,
    entries: HashMap<u64, (String, Vec<ContentSegment>)>,
}

impl ParsedContentCache {
    fn get(&self, key: u64, content: &str) -> Option<Vec<ContentSegment>> {
        self.entries
            .get(&key)
            .and_then(|(cached_content, segments)| {
                if cached_content == content {
                    Some(segments.clone())
                } else {
                    None
                }
            })
    }

    fn insert(&mut self, key: u64, content: String, segments: Vec<ContentSegment>) {
        if !self.entries.contains_key(&key) {
            self.order.push_back(key);
        }
        self.entries.insert(key, (content, segments));

        while self.order.len() > PARSED_CACHE_MAX_ENTRIES {
            if let Some(evicted_key) = self.order.pop_front() {
                self.entries.remove(&evicted_key);
            }
        }
    }
}

fn parsed_cache() -> &'static Mutex<ParsedContentCache> {
    static CACHE: OnceLock<Mutex<ParsedContentCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(ParsedContentCache::default()))
}

fn content_hash(content: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

fn group_segments(segments: Vec<ContentSegment>) -> Vec<ContentSegment> {
    let mut grouped_segments: Vec<ContentSegment> = Vec::new();
    let mut current_list: Vec<String> = Vec::new();

    for segment in segments {
        match segment {
            ContentSegment::ListItem(text) => {
                current_list.push(text);
            }
            other => {
                if !current_list.is_empty() {
                    let list_items = current_list.drain(..).collect::<Vec<_>>();
                    grouped_segments.push(ContentSegment::ListItem(format!(
                        "\u{0001}LIST_GROUP\u{0001}{}",
                        list_items.join("\u{0001}ITEM\u{0001}")
                    )));
                }
                grouped_segments.push(other);
            }
        }
    }

    if !current_list.is_empty() {
        let list_items = current_list.drain(..).collect::<Vec<_>>();
        grouped_segments.push(ContentSegment::ListItem(format!(
            "\u{0001}LIST_GROUP\u{0001}{}",
            list_items.join("\u{0001}ITEM\u{0001}")
        )));
    }

    grouped_segments
}

async fn parse_content_cached(content: String) -> Vec<ContentSegment> {
    let key = content_hash(&content);
    if let Ok(cache) = parsed_cache().lock() {
        if let Some(cached) = cache.get(key, &content) {
            return cached;
        }
    }

    let content_for_parse = content.clone();
    let parsed = tokio::task::spawn_blocking(move || {
        let segments = parse_message_content(&content_for_parse);
        group_segments(segments)
    })
    .await
    .unwrap_or_else(|_| {
        vec![ContentSegment::Text(content.clone())]
    });

    if let Ok(mut cache) = parsed_cache().lock() {
        cache.insert(key, content, parsed.clone());
    }

    parsed
}

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
                                                class: "px-2 py-1 rounded bg-[var(--color-base-300)] text-[var(--color-base-content)] font-mono text-xs sm:text-sm",
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
    let mut parsed_content = use_signal(String::new);
    let mut grouped_segments = use_signal(|| None::<Vec<ContentSegment>>);

    if *parsed_content.read() != content {
        let request_content = content.clone();
        parsed_content.set(request_content.clone());
        grouped_segments.set(None);

        let mut grouped_segments_clone = grouped_segments.clone();
        let parsed_content_clone = parsed_content.clone();
        spawn(async move {
            let parsed = parse_content_cached(request_content.clone()).await;
            if *parsed_content_clone.read() == request_content {
                grouped_segments_clone.set(Some(parsed));
            }
        });
    }

    rsx! {
        div {
            class: "space-y-2 prose prose-sm sm:prose-base md:prose-lg max-w-none",
            if let Some(parsed) = grouped_segments.read().as_ref() {
                for segment in parsed.iter() {
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
                            rsx! {
                                div {
                                    class: "rounded-lg overflow-hidden my-4",
                                    if !language.is_empty() {
                                        div {
                                            class: "bg-[var(--color-base-300)] px-4 py-2 text-xs sm:text-sm font-semibold text-[var(--color-base-content)]/70 flex items-center justify-between",
                                            span {
                                                "{language}"
                                            }
                                        }
                                    }
                                    pre {
                                        class: "bg-[var(--color-base-300)] text-[var(--color-base-content)] p-4 overflow-x-auto m-0",
                                        code {
                                            class: "font-mono text-xs sm:text-sm md:text-base leading-relaxed",
                                            "{code}"
                                        }
                                    }
                                }
                            }
                        },
                        ContentSegment::Header { level, text } => {
                            let header_class = match level {
                                1 => "text-xl sm:text-2xl md:text-3xl font-bold mt-6 mb-4",
                                2 => "text-lg sm:text-xl md:text-2xl font-bold mt-5 mb-3",
                                3 => "text-base sm:text-lg md:text-xl font-bold mt-4 mb-2",
                                4 => "text-sm sm:text-base md:text-lg font-bold mt-3 mb-2",
                                5 => "text-xs sm:text-sm md:text-base font-bold mt-2 mb-1",
                                _ => "text-xs sm:text-sm md:text-base font-bold mt-2 mb-1",
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
            } else {
                div {
                    class: "whitespace-pre-wrap break-words text-[var(--color-base-content)]/90",
                    "{content}"
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
            class: "flex-1 min-h-0 overflow-y-auto px-3 sm:px-4 md:px-6 py-4 sm:py-5 md:py-6",

            if messages.read().is_empty() {
                div {
                    class: "flex flex-col items-center justify-center h-full max-w-2xl mx-auto text-center",

                    div {
                        class: "text-6xl mb-6",
                        "🎮"
                    }

                    h2 {
                        class: "text-xl sm:text-2xl md:text-3xl font-bold text-[var(--color-base-content)] mb-3",
                        "{empty_title}"
                    }

                    p {
                        class: "text-[var(--color-base-content)]/80 mb-6 text-base sm:text-lg md:text-xl",
                        "{empty_description}"
                    }

                    div {
                        class: "inline-flex items-center px-4 py-2 bg-[var(--color-base-200)] rounded-lg shadow-sm border border-[var(--color-base-300)]",
                        div {
                            class: "text-sm sm:text-base font-medium text-[var(--color-base-content)]",
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
                                                        class: "text-sm sm:text-base bg-[var(--color-primary)] text-[var(--color-primary-content)] p-3 sm:p-4 rounded-lg rounded-tr-none border border-transparent mb-1",
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
                                                    class: "text-sm sm:text-base bg-[var(--color-base-300)] text-[var(--color-base-content)] p-3 sm:p-4 rounded-lg rounded-tl-none mb-1",
                                                    div {
                                                        class: "font-semibold text-[var(--color-base-content)] mb-1 text-sm sm:text-base",
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
                                                        class: "text-xs sm:text-sm text-[var(--color-base-content)]/70 font-medium",
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
