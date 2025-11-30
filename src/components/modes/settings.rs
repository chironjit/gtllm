use crate::utils::{InputSettings, Settings as AppSettings, Theme};
use dioxus::prelude::*;

#[component]
pub fn Settings(
    theme: Signal<Theme>,
    app_settings: Signal<AppSettings>,
    input_settings: Signal<InputSettings>,
    on_settings_change: EventHandler<InputSettings>,
    on_api_key_save: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let _ = theme.read();
    let current_settings = *input_settings.read();
    let mut api_key_input = use_signal(|| String::new());
    let mut show_api_key = use_signal(|| false);

    rsx! {
        div {
            class: "flex flex-col h-full overflow-y-auto",

            div {
                class: "max-w-2xl mx-auto w-full p-6",

                // Header
                div {
                    class: "flex items-center justify-between mb-8",
                    h1 {
                        class: "text-3xl font-bold text-[var(--color-base-content)]",
                        "Settings"
                    }
                    button {
                        onclick: move |_| on_close.call(()),
                        class: "p-2 hover:bg-[var(--color-base-300)] rounded-lg transition-colors",
                        title: "Close settings",
                        img {
                            src: asset!("/assets/close.svg"),
                            class: "w-6 h-6",
                            alt: "Close"
                        }
                    }
                }

                // Settings sections
                div {
                    class: "space-y-6",

                    // API Key settings
                    div {
                        class: "bg-[var(--color-base-200)] rounded-lg p-6 border border-[var(--color-base-300)]",

                        h2 {
                            class: "text-xl font-semibold text-[var(--color-base-content)] mb-4",
                            "OpenRouter API Key"
                        }

                        {
                            let has_api_key = app_settings.read().has_api_key();
                            let showing_api_key = *show_api_key.read();

                            rsx! {
                                div {
                                    class: "space-y-4",

                                    if has_api_key {
                                        // Show API key status
                                        div {
                                            class: "flex items-center gap-2 text-sm text-green-500",
                                            span { "✓" }
                                            span { "API Key configured" }
                                        }

                                        button {
                                            onclick: move |_| {
                                                show_api_key.set(!showing_api_key);
                                            },
                                            class: "text-sm text-[var(--color-primary)] hover:underline",
                                            if showing_api_key { "Hide API Key" } else { "Update API Key" }
                                        }
                                    } else {
                                        // No API key set
                                        div {
                                            class: "flex items-center gap-2 text-sm text-yellow-500",
                                            span { "⚠" }
                                            span { "No API Key configured - Set one to use the app" }
                                        }
                                    }

                                    if !has_api_key || showing_api_key {
                                div {
                                    class: "space-y-2",
                                    input {
                                        r#type: "password",
                                        value: "{api_key_input}",
                                        oninput: move |evt| api_key_input.set(evt.value().clone()),
                                        placeholder: "sk-or-v1-...",
                                        class: "w-full px-4 py-2 rounded-lg bg-[var(--color-base-100)] text-[var(--color-base-content)] border-2 border-[var(--color-base-300)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent",
                                    }

                                    button {
                                        onclick: move |_| {
                                            let key = api_key_input.read().clone();
                                            if !key.trim().is_empty() {
                                                on_api_key_save.call(key);
                                                api_key_input.set(String::new());
                                                show_api_key.set(false);
                                            }
                                        },
                                        disabled: api_key_input.read().trim().is_empty(),
                                        class: "px-4 py-2 rounded-lg bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm font-medium",
                                        "Save API Key"
                                    }

                                    p {
                                        class: "text-xs text-[var(--color-base-content)]/70",
                                        "Get your API key from "
                                        a {
                                            href: "https://openrouter.ai/keys",
                                            target: "_blank",
                                            class: "text-[var(--color-primary)] hover:underline",
                                            "OpenRouter"
                                        }
                                    }
                                }
                            }
                                }
                            }
                        }
                    }

                    // Input settings
                    div {
                        class: "bg-[var(--color-base-200)] rounded-lg p-6 border border-[var(--color-base-300)]",

                        h2 {
                            class: "text-xl font-semibold text-[var(--color-base-content)] mb-4",
                            "Input Settings"
                        }

                        div {
                            class: "space-y-4",

                            // Submit behavior
                            div {
                                label {
                                    class: "flex items-start gap-3 cursor-pointer",
                                    input {
                                        r#type: "checkbox",
                                        checked: current_settings.ctrl_enter_submit,
                                        onchange: move |evt| {
                                            let new_settings = InputSettings {
                                                ctrl_enter_submit: evt.checked(),
                                            };
                                            on_settings_change.call(new_settings);
                                        },
                                        class: "mt-1 w-4 h-4 accent-[var(--color-primary)]"
                                    }
                                    div {
                                        div {
                                            class: "font-medium text-[var(--color-base-content)]",
                                            "Use Ctrl+Enter to submit"
                                        }
                                        div {
                                            class: "text-sm text-[var(--color-base-content)]/70 mt-1",
                                            if current_settings.ctrl_enter_submit {
                                                "Ctrl+Enter sends messages, Enter creates new lines"
                                            } else {
                                                "Enter sends messages, Ctrl+Enter creates new lines"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Theme info (read-only)
                    div {
                        class: "bg-[var(--color-base-200)] rounded-lg p-6 border border-[var(--color-base-300)]",

                        h2 {
                            class: "text-xl font-semibold text-[var(--color-base-content)] mb-4",
                            "Theme"
                        }

                        p {
                            class: "text-sm text-[var(--color-base-content)]/70",
                            "Change theme using the controls in the header"
                        }
                    }

                    // About section
                    div {
                        class: "bg-[var(--color-base-200)] rounded-lg p-6 border border-[var(--color-base-300)]",

                        h2 {
                            class: "text-xl font-semibold text-[var(--color-base-content)] mb-4",
                            "About"
                        }

                        div {
                            class: "space-y-2 text-sm text-[var(--color-base-content)]/70",
                            p { "GTLLM - Game Theory meets Large Language Models" }
                            p { "Explore different AI interaction modes and strategies" }
                        }
                    }
                }
            }
        }
    }
}
