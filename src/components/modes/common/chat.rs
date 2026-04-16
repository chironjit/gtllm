use crate::utils::{
    parse_inline_elements, parse_message_content, ContentSegment, InlineSegment, ListBlock,
    Message, TableAlignment, TableBlock, Theme,
};
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

fn parse_content_cached(content: &str) -> Vec<ContentSegment> {
    let key = content_hash(content);

    if let Ok(cache) = parsed_cache().lock() {
        if let Some(cached) = cache.get(key, content) {
            return cached;
        }
    }

    let parsed = parse_message_content(content);

    if let Ok(mut cache) = parsed_cache().lock() {
        cache.insert(key, content.to_string(), parsed.clone());
    }

    parsed
}

fn get_user_avatar() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect fill='%2306b6d4' width='40' height='40' rx='20'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='20'%3E=d%3C/text%3E%3C/svg%3E"
}

fn get_assistant_avatar() -> &'static str {
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect fill='%236366f1' width='40' height='40' rx='20'/%3E%3Ctext x='50%25' y='50%25' text-anchor='middle' dy='.3em' fill='white' font-size='20'%3E%3E%3C/text%3E%3C/svg%3E"
}

#[component]
fn InlineNodes(theme: Signal<Theme>, segments: Vec<InlineSegment>) -> Element {
    let _theme_value = theme.read();

    rsx! {
        for segment in segments {
            match segment {
                InlineSegment::Text(text) => rsx! {
                    span { "{text}" }
                },
                InlineSegment::Bold(children) => rsx! {
                    strong {
                        class: "font-semibold",
                        InlineNodes { theme, segments: children }
                    }
                },
                InlineSegment::Italic(children) => rsx! {
                    em {
                        class: "italic",
                        InlineNodes { theme, segments: children }
                    }
                },
                InlineSegment::BoldItalic(children) => rsx! {
                    strong {
                        class: "font-semibold italic",
                        InlineNodes { theme, segments: children }
                    }
                },
                InlineSegment::Link { text, url } => rsx! {
                    a {
                        href: "{url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "text-[var(--color-primary)] underline underline-offset-2 break-all",
                        InlineNodes { theme, segments: text }
                    }
                },
                InlineSegment::InlineCode(code) => rsx! {
                    code {
                        class: "rounded bg-[var(--color-base-300)] px-1.5 py-0.5 font-mono text-[0.95em]",
                        "{code}"
                    }
                },
                InlineSegment::Formula(formula) => rsx! {
                    code {
                        class: "rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] px-1.5 py-0.5 font-mono italic text-[0.95em]",
                        "{formula}"
                    }
                },
            }
        }
    }
}

#[component]
fn InlineMarkdown(theme: Signal<Theme>, text: String) -> Element {
    rsx! {
        InlineNodes {
            theme,
            segments: parse_inline_elements(&text),
        }
    }
}

#[component]
fn ListView(theme: Signal<Theme>, list: ListBlock) -> Element {
    let list_class = if list.ordered {
        "list-decimal ml-6 my-3 space-y-2"
    } else {
        "list-disc ml-6 my-3 space-y-2"
    };

    rsx! {
        if list.ordered {
            ol {
                class: "{list_class}",
                for (index, item) in list.items.into_iter().enumerate() {
                    li {
                        key: "{index}",
                        class: "pl-1",
                        if !item.text.is_empty() {
                            div {
                                class: "whitespace-pre-wrap break-words",
                                InlineMarkdown {
                                    theme,
                                    text: item.text,
                                }
                            }
                        }
                        for (child_index, child_list) in item.children.into_iter().enumerate() {
                            div {
                                key: "{child_index}",
                                class: "mt-2",
                                ListView {
                                    theme,
                                    list: child_list,
                                }
                            }
                        }
                    }
                }
            }
        } else {
            ul {
                class: "{list_class}",
                for (index, item) in list.items.into_iter().enumerate() {
                    li {
                        key: "{index}",
                        class: "pl-1",
                        if !item.text.is_empty() {
                            div {
                                class: "whitespace-pre-wrap break-words",
                                InlineMarkdown {
                                    theme,
                                    text: item.text,
                                }
                            }
                        }
                        for (child_index, child_list) in item.children.into_iter().enumerate() {
                            div {
                                key: "{child_index}",
                                class: "mt-2",
                                ListView {
                                    theme,
                                    list: child_list,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn table_cell_alignment_class(alignment: &TableAlignment) -> &'static str {
    match alignment {
        TableAlignment::Left => "text-left",
        TableAlignment::Center => "text-center",
        TableAlignment::Right => "text-right",
        TableAlignment::None => "text-left",
    }
}

#[component]
fn TableView(theme: Signal<Theme>, table: TableBlock) -> Element {
    rsx! {
        div {
            class: "my-4 overflow-x-auto rounded-xl border border-[var(--color-base-300)]",
            table {
                class: "min-w-full border-collapse text-sm sm:text-base",
                thead {
                    class: "bg-[var(--color-base-200)]",
                    tr {
                        for (index, header) in table.headers.iter().enumerate() {
                            th {
                                key: "{index}",
                                class: "whitespace-pre-wrap break-words border-b border-[var(--color-base-300)] px-4 py-3 font-semibold text-[var(--color-base-content)] {table_cell_alignment_class(table.alignments.get(index).unwrap_or(&TableAlignment::None))}",
                                InlineMarkdown {
                                    theme,
                                    text: header.clone(),
                                }
                            }
                        }
                    }
                }
                tbody {
                    for (row_index, row) in table.rows.iter().enumerate() {
                        tr {
                            key: "{row_index}",
                            class: if row_index % 2 == 0 {
                                "bg-[var(--color-base-100)]"
                            } else {
                                "bg-[var(--color-base-200)]/40"
                            },
                            for (cell_index, cell) in row.iter().enumerate() {
                                td {
                                    key: "{cell_index}",
                                    class: "whitespace-pre-wrap break-words border-t border-[var(--color-base-300)] px-4 py-3 align-top text-[var(--color-base-content)] {table_cell_alignment_class(table.alignments.get(cell_index).unwrap_or(&TableAlignment::None))}",
                                    InlineMarkdown {
                                        theme,
                                        text: cell.clone(),
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
fn BlockNodes(theme: Signal<Theme>, segments: Vec<ContentSegment>) -> Element {
    rsx! {
        for (index, segment) in segments.into_iter().enumerate() {
            match segment {
                ContentSegment::Paragraph(text) => rsx! {
                    p {
                        key: "{index}",
                        class: "whitespace-pre-wrap break-words leading-7",
                        InlineMarkdown { theme, text }
                    }
                },
                ContentSegment::Header { level, text } => {
                    let header_class = match level {
                        1 => "text-2xl sm:text-3xl font-bold leading-tight mt-2",
                        2 => "text-xl sm:text-2xl font-bold leading-tight mt-2",
                        3 => "text-lg sm:text-xl font-semibold leading-tight mt-1",
                        4 => "text-base sm:text-lg font-semibold leading-tight mt-1",
                        5 => "text-sm sm:text-base font-semibold uppercase tracking-wide mt-1",
                        _ => "text-sm font-semibold uppercase tracking-wide mt-1",
                    };

                    rsx! {
                        match level {
                            1 => rsx! { h1 { key: "{index}", class: "{header_class}", InlineMarkdown { theme, text } } },
                            2 => rsx! { h2 { key: "{index}", class: "{header_class}", InlineMarkdown { theme, text } } },
                            3 => rsx! { h3 { key: "{index}", class: "{header_class}", InlineMarkdown { theme, text } } },
                            4 => rsx! { h4 { key: "{index}", class: "{header_class}", InlineMarkdown { theme, text } } },
                            5 => rsx! { h5 { key: "{index}", class: "{header_class}", InlineMarkdown { theme, text } } },
                            _ => rsx! { h6 { key: "{index}", class: "{header_class}", InlineMarkdown { theme, text } } },
                        }
                    }
                }
                ContentSegment::CodeBlock { language, code } => rsx! {
                    div {
                        key: "{index}",
                        class: "overflow-hidden rounded-xl border border-[var(--color-base-300)] bg-[var(--color-base-200)]",
                        if !language.is_empty() {
                            div {
                                class: "border-b border-[var(--color-base-300)] px-4 py-2 text-xs font-semibold uppercase tracking-wide text-[var(--color-base-content)]/70",
                                "{language}"
                            }
                        }
                        pre {
                            class: "overflow-x-auto p-4",
                            code {
                                class: "font-mono text-xs sm:text-sm leading-6 text-[var(--color-base-content)]",
                                "{code}"
                            }
                        }
                    }
                },
                ContentSegment::FormulaBlock(formula) => rsx! {
                    div {
                        key: "{index}",
                        class: "overflow-x-auto rounded-xl border border-[var(--color-base-300)] bg-[var(--color-base-200)] px-4 py-3 font-mono italic text-sm sm:text-base leading-7",
                        "{formula}"
                    }
                },
                ContentSegment::Table(table) => rsx! {
                    div {
                        key: "{index}",
                        TableView { theme, table }
                    }
                },
                ContentSegment::List(list) => rsx! {
                    div {
                        key: "{index}",
                        ListView { theme, list }
                    }
                },
                ContentSegment::Blockquote(children) => rsx! {
                    blockquote {
                        key: "{index}",
                        class: "border-l-4 border-[var(--color-primary)]/50 bg-[var(--color-base-200)]/60 pl-4 py-2 italic",
                        BlockNodes { theme, segments: children }
                    }
                },
                ContentSegment::HorizontalRule => rsx! {
                    hr {
                        key: "{index}",
                        class: "border-0 border-t border-[var(--color-base-300)] my-2",
                    }
                },
            }
        }
    }
}

#[component]
pub fn FormattedText(theme: Signal<Theme>, content: String) -> Element {
    let _theme_value = theme.read();
    let segments = parse_content_cached(&content);

    rsx! {
        div {
            class: "space-y-3 text-sm sm:text-base text-[var(--color-base-content)]",
            if segments.is_empty() {
                div {
                    class: "whitespace-pre-wrap break-words",
                    "{content}"
                }
            } else {
                BlockNodes {
                    theme,
                    segments,
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
    let _theme_value = theme.read();

    rsx! {
        div {
            class: "flex-1 min-h-0 overflow-y-auto px-3 sm:px-4 md:px-6 py-4 sm:py-5 md:py-6",

            if messages.read().is_empty() {
                div {
                    class: "flex h-full max-w-2xl mx-auto flex-col items-center justify-center text-center",

                    div {
                        class: "mb-6 text-6xl",
                        "🎮"
                    }

                    h2 {
                        class: "mb-3 text-xl sm:text-2xl md:text-3xl font-bold text-[var(--color-base-content)]",
                        "{empty_title}"
                    }

                    p {
                        class: "mb-6 text-base sm:text-lg md:text-xl text-[var(--color-base-content)]/80",
                        "{empty_description}"
                    }

                    div {
                        class: "inline-flex items-center rounded-lg border border-[var(--color-base-300)] bg-[var(--color-base-200)] px-4 py-2 shadow-sm",
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
                                    class: "mb-4 flex items-start",

                                    if !is_user {
                                        img {
                                            class: "mr-4 rounded-full",
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
                                                        class: "mb-1 rounded-lg rounded-tr-none border border-transparent bg-[var(--color-primary)] p-3 sm:p-4 text-[var(--color-primary-content)]",
                                                        FormattedText {
                                                            theme,
                                                            content: message.content.clone(),
                                                        }
                                                    }
                                                    div {
                                                        class: "flex items-center justify-end",
                                                        div {
                                                            class: "text-xs font-medium text-[var(--color-base-content)]/70",
                                                            "{message.timestamp}"
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            div {
                                                div {
                                                    class: "mb-1 rounded-lg rounded-tl-none bg-[var(--color-base-300)] p-3 sm:p-4 text-[var(--color-base-content)]",
                                                    div {
                                                        class: "mb-1 text-sm sm:text-base font-semibold text-[var(--color-base-content)]",
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
                                                        class: "text-xs sm:text-sm font-medium text-[var(--color-base-content)]/70",
                                                        "{message.timestamp}"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if is_user {
                                        img {
                                            class: "ml-4 rounded-full",
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
