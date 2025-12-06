use crate::utils::{Model, OpenRouterClient, Theme};
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Props, Clone)]
pub struct ModelSelectorProps {
    theme: Signal<Theme>,
    client: Arc<OpenRouterClient>,
    on_models_selected: EventHandler<Vec<String>>,
}

impl PartialEq for ModelSelectorProps {
    fn eq(&self, other: &Self) -> bool {
        self.theme == other.theme
        // Skip client comparison as Arc<OpenRouterClient> doesn't implement PartialEq
    }
}

#[component]
pub fn ModelSelector(props: ModelSelectorProps) -> Element {
    let theme = props.theme;
    let client = props.client;
    let on_models_selected = props.on_models_selected;
    let _ = theme.read();
    let mut selected_models = use_signal(|| Vec::<String>::new());
    let mut search_query = use_signal(|| String::new());
    let mut models = use_signal(|| None::<Result<Vec<Model>, String>>);

    // Fetch models on component mount (run once)
    let _fetch = use_hook(|| {
        let client_clone = client.clone();
        let mut models_clone = models.clone();
        spawn(async move {
            let result = client_clone.fetch_models().await;
            models_clone.set(Some(result));
        });
    });

    let mut toggle_model = move |model_id: String| {
        let mut selected = selected_models.write();
        if let Some(pos) = selected.iter().position(|id| id == &model_id) {
            selected.remove(pos);
        } else {
            selected.push(model_id);
        }
    };

    let start_chat = move |_| {
        let selected = selected_models.read().clone();
        if !selected.is_empty() {
            on_models_selected.call(selected);
        }
    };

    // Get models from signal
    let models_result = models.read();
    let loading = models_result.is_none();
    let (available_models, error) = match &*models_result {
        Some(Ok(models)) => (models.clone(), None),
        Some(Err(e)) => (Vec::new(), Some(e.clone())),
        None => (Vec::new(), None),
    };

    // Filter models based on search query
    let filtered_models: Vec<Model> = {
        let search = search_query.read().to_lowercase();

        if search.is_empty() {
            available_models.clone()
        } else {
            available_models
                .iter()
                .filter(|m| {
                    m.display_name().to_lowercase().contains(&search)
                        || m.id.to_lowercase().contains(&search)
                })
                .cloned()
                .collect()
        }
    };

    rsx! {
        div {
            class: "flex flex-col h-full p-3",

            // Header
            div {
                class: "mb-2",
                h2 {
                    class: "text-lg font-bold text-[var(--color-base-content)] mb-1",
                    "Select Models"
                }
                p {
                    class: "text-xs text-[var(--color-base-content)]/70 mb-1.5",
                    "Choose one or more models to chat with."
                }
                div {
                    class: "flex items-start gap-1.5 p-1.5 bg-blue-500/10 border border-blue-500/30 rounded text-xs text-[var(--color-base-content)]/80",
                    span {
                        class: "text-blue-500",
                        "ℹ️"
                    }
                    div {
                        span {
                            class: "font-semibold",
                            "Note: "
                        }
                        "Some models may not work with your "
                        a {
                            href: "https://openrouter.ai/settings/privacy",
                            target: "_blank",
                            class: "text-[var(--color-primary)] hover:underline font-medium",
                            "privacy settings"
                        }
                        ". If a model fails, try adjusting your data retention policy or selecting a different model."
                    }
                }
            }

            // Search box
            if !loading && error.is_none() {
                div {
                    class: "mb-2",
                    input {
                        r#type: "text",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value().clone()),
                        placeholder: "Search models...",
                        class: "w-full px-2 py-1 text-xs rounded bg-[var(--color-base-100)] text-[var(--color-base-content)] border border-[var(--color-base-300)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)] focus:border-transparent",
                    }
                }
            }

            // Content
            div {
                class: "flex-1 overflow-y-auto",

                if loading {
                    div {
                        class: "flex items-center justify-center h-full",
                        div {
                            class: "text-center",
                            div {
                                class: "flex justify-center mb-4",
                                img {
                                    src: asset!("/assets/loading.svg"),
                                    class: "w-16 h-16 animate-spin",
                                    alt: "Loading",
                                }
                            }
                            p {
                                class: "text-[var(--color-base-content)]/70",
                                "Loading available models..."
                            }
                        }
                    }
                } else if let Some(err) = &error {
                    div {
                        class: "flex items-center justify-center h-full",
                        div {
                            class: "text-center max-w-md",
                            div {
                                class: "flex justify-center mb-4",
                                img {
                                    src: asset!("/assets/alert.svg"),
                                    class: "w-16 h-16",
                                    alt: "Error",
                                }
                            }
                            p {
                                class: "text-red-500 mb-2",
                                "{err}"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70",
                                "Please check your API key in settings and try again."
                            }
                        }
                    }
                } else {
                    div {
                        class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-1.5",

                        for model in filtered_models.iter() {
                            {
                                let model_id = model.id.clone();
                                let is_selected = selected_models.read().contains(&model_id);
                                let display_name = model.display_name();
                                let context_length = model.context_length.map(|c| format!("{}K context", c / 1000));

                                rsx! {
                                    button {
                                        key: "{model_id}",
                                        onclick: move |_| toggle_model(model_id.clone()),
                                        class: if is_selected {
                                            "p-1.5 rounded border border-[var(--color-primary)] bg-[var(--color-primary)]/10 transition-all duration-200 text-left"
                                        } else {
                                            "p-1.5 rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] hover:border-[var(--color-primary)]/50 transition-all duration-200 text-left"
                                        },

                                        div {
                                            class: "flex items-start gap-1.5",
                                            div {
                                                class: "flex-shrink-0",
                                                if is_selected {
                                                    span {
                                                        class: "text-[var(--color-primary)] text-xs",
                                                        "✓"
                                                    }
                                                } else {
                                                    span {
                                                        class: "text-[var(--color-base-content)]/30 text-xs",
                                                        "○"
                                                    }
                                                }
                                            }
                                            div {
                                                class: "flex-1 min-w-0",
                                                div {
                                                    class: "font-semibold text-xs text-[var(--color-base-content)] truncate",
                                                    "{display_name}"
                                                }
                                                div {
                                                    class: "text-xs text-[var(--color-base-content)]/50 truncate",
                                                    "{model.id}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if filtered_models.is_empty() {
                        div {
                            class: "text-center py-8",
                            p {
                                class: "text-[var(--color-base-content)]/70",
                                "No models found matching your search."
                            }
                        }
                    }
                }
            }

            // Footer with selected count and start button
            if !loading && error.is_none() {
                div {
                    class: "mt-2 pt-2 border-t border-[var(--color-base-300)]",
                    div {
                        class: "flex items-center justify-between gap-2",
                        div {
                            class: "text-xs text-[var(--color-base-content)]/70",
                            if selected_models.read().len() > 0 {
                                "{selected_models.read().len()} selected"
                            } else {
                                "Select a model"
                            }
                        }
                        button {
                            onclick: start_chat,
                            disabled: selected_models.read().is_empty(),
                            class: "px-3 py-1 text-xs rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 disabled:opacity-50 disabled:cursor-not-allowed font-medium transition-all duration-200",
                            "Start"
                        }
                    }
                }
            }
        }
    }
}
