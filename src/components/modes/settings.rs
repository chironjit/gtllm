use crate::utils::{InputSettings, Theme};
use dioxus::prelude::*;

#[component]
pub fn Settings(
    theme: Signal<Theme>,
    input_settings: Signal<InputSettings>,
    on_settings_change: EventHandler<InputSettings>,
    on_close: EventHandler<()>,
) -> Element {
    let _ = theme.read();
    let current_settings = *input_settings.read();

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
