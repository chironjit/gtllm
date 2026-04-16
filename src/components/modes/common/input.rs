use crate::utils::{InputSettings, Theme};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

const CHAT_INPUT_MIN_HEIGHT_PX: u32 = 56;
const CHAT_INPUT_MAX_HEIGHT_PX: u32 = 208;
static NEXT_CHAT_INPUT_ID: AtomicUsize = AtomicUsize::new(0);

fn resize_chat_input(textarea_id: &str) {
    let _ = document::eval(&format!(
        r#"
            const textarea = document.getElementById({textarea_id:?});
            if (!textarea) {{
                return;
            }}

            const minHeight = {CHAT_INPUT_MIN_HEIGHT_PX};
            const maxHeight = {CHAT_INPUT_MAX_HEIGHT_PX};
            textarea.style.height = "auto";
            const nextHeight = Math.min(Math.max(textarea.scrollHeight, minHeight), maxHeight);
            textarea.style.height = `${{nextHeight}}px`;
            textarea.style.overflowY = textarea.scrollHeight > maxHeight ? "auto" : "hidden";
        "#
    ));
}

#[component]
pub fn ChatInput(
    theme: Signal<Theme>,
    input_settings: Signal<InputSettings>,
    is_streaming: bool,
    on_send: EventHandler<String>,
) -> Element {
    let mut input_text = use_signal(|| String::new());
    let _ = theme.read();
    let textarea_id =
        use_hook(|| format!("chat-input-{}", NEXT_CHAT_INPUT_ID.fetch_add(1, Ordering::Relaxed)));

    {
        let textarea_id = textarea_id.clone();
        use_effect(move || {
            let _ = input_text();
            resize_chat_input(&textarea_id);
        });
    }

    let handle_keydown = move |evt: KeyboardEvent| {
        if is_streaming {
            return;
        }

        let settings = input_settings.read();
        let text = input_text.read().clone();

        if settings.ctrl_enter_submit {
            if evt.key() == Key::Enter
                && evt.modifiers().contains(Modifiers::CONTROL)
                && !text.trim().is_empty()
            {
                evt.prevent_default();
                on_send.call(text);
                input_text.set(String::new());
            }
        } else {
            if evt.key() == Key::Enter
                && !evt.modifiers().contains(Modifiers::CONTROL)
                && !text.trim().is_empty()
            {
                evt.prevent_default();
                on_send.call(text);
                input_text.set(String::new());
            }
        }
    };

    let submit_message = move |_: MouseEvent| {
        if is_streaming {
            return;
        }
        let text = input_text.read().clone();
        if !text.trim().is_empty() {
            on_send.call(text);
            input_text.set(String::new());
        }
    };

    let has_content = !input_text.read().trim().is_empty();
    let can_send = has_content && !is_streaming;

    rsx! {
        div {
            class: "relative w-full bg-[var(--color-base-200)] border-[var(--color-base-300)] border-t shadow-lg",

            div {
                class: "max-w-6xl mx-auto px-3 sm:px-4 md:px-6 py-3",

                div {
                    class: "relative",

                    textarea {
                        id: "{textarea_id}",
                        value: "{input_text}",
                        oninput: move |evt| input_text.set(evt.value().clone()),
                        onkeydown: handle_keydown,
                        disabled: is_streaming,
                        rows: "2",
                        placeholder: if is_streaming {
                            "Waiting for response..."
                        } else if input_settings.read().ctrl_enter_submit {
                            "Type your message... (Ctrl+Enter to send)"
                        } else {
                            "Type your message... (Enter to send)"
                        },
                        class: if is_streaming {
                            "w-full px-4 pr-16 py-3 rounded-xl bg-[var(--color-base-100)]/50 text-[var(--color-base-content)]/50 border-[var(--color-base-300)] border-2 resize-none min-h-[3.5rem] max-h-[13rem] overflow-y-hidden font-family-sans opacity-60 cursor-not-allowed"
                        } else {
                            "w-full px-4 pr-16 py-3 rounded-xl bg-[var(--color-base-100)] text-[var(--color-base-content)] border-[var(--color-base-300)] border-2 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent transition-all duration-200 text-sm sm:text-base shadow-sm resize-none min-h-[3.5rem] max-h-[13rem] overflow-y-hidden font-family-sans"
                        },
                        autofocus: true,
                    }

                    button {
                        onclick: submit_message,
                        disabled: !can_send,
                        class: "absolute right-3 bottom-3 p-2 rounded-lg transition-all duration-200 text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-transparent",
                        class: if can_send {
                            "bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 shadow-sm hover:shadow-md"
                        } else {
                            "bg-[var(--color-base-300)] text-[var(--color-base-content)]/50 cursor-not-allowed"
                        },
                        span { "➤" }
                    }
                }

                div {
                    class: "text-xs text-[var(--color-base-content)]/70 mt-1 px-1 text-center",
                    if is_streaming {
                        "Waiting for response..."
                    } else if input_settings.read().ctrl_enter_submit {
                        "Use Ctrl+Enter to send, Enter for new line"
                    } else {
                        "Use Enter to send, Ctrl+Enter for new line"
                    }
                }
            }
        }
    }
}
