use super::common::{ChatInput, ModelSelector};
use crate::utils::{ChatMessage, InputSettings, OpenRouterClient, StreamEvent, Theme};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct ModelResponse {
    model_id: String,
    content: String,
    error_message: Option<String>,
}

#[derive(Props, Clone)]
pub struct StandardProps {
    theme: Signal<Theme>,
    client: Option<Arc<OpenRouterClient>>,
    input_settings: Signal<InputSettings>,
}

impl PartialEq for StandardProps {
    fn eq(&self, other: &Self) -> bool {
        self.theme == other.theme && self.input_settings == other.input_settings
        // Skip client comparison
    }
}

#[component]
pub fn Standard(props: StandardProps) -> Element {
    let theme = props.theme;
    let client = props.client.clone();
    let client_for_send = props.client;
    let input_settings = props.input_settings;
    let _ = theme.read();
    let mut selected_models = use_signal(|| Vec::<String>::new());
    let mut user_messages = use_signal(|| Vec::<String>::new());
    let mut model_responses = use_signal(|| Vec::<Vec<ModelResponse>>::new());
    let mut current_streaming_responses = use_signal(|| HashMap::<String, String>::new());
    let mut is_streaming = use_signal(|| false);

    // Handle model selection
    let on_models_selected = move |models: Vec<String>| {
        selected_models.set(models);
    };

    // Handle sending a message
    let send_message = move |text: String| {
        if text.trim().is_empty() || *is_streaming.read() {
            return;
        }

        let models = selected_models.read().clone();
        if models.is_empty() {
            return;
        }

        // Add user message
        user_messages.write().push(text.clone());

        // Start streaming from all selected models
        if let Some(client_arc) = &client_for_send {
            let client = client_arc.clone();
            let messages = vec![ChatMessage::user(text)];
            let mut is_streaming_clone = is_streaming.clone();
            let mut current_streaming_responses_clone = current_streaming_responses.clone();
            let mut model_responses_clone = model_responses.clone();

            spawn(async move {
                is_streaming_clone.set(true);
                current_streaming_responses_clone.write().clear();

                match client.stream_chat_completion_multi(models.clone(), messages).await {
                    Ok(mut rx) => {
                        let mut done_models = std::collections::HashSet::new();

                        while let Some(event) = rx.recv().await {
                            let model_id = event.model_id.clone();

                            match event.event {
                                StreamEvent::Content(content) => {
                                    let mut responses = current_streaming_responses_clone.write();
                                    responses
                                        .entry(model_id.clone())
                                        .and_modify(|s| s.push_str(&content))
                                        .or_insert(content);
                                }
                                StreamEvent::Done => {
                                    done_models.insert(model_id);

                                    // Check if all models are done
                                    if done_models.len() >= models.len() {
                                        // All models done, finalize responses
                                        let final_responses: Vec<ModelResponse> = {
                                            let responses = current_streaming_responses_clone.read();
                                            models
                                                .iter()
                                                .map(|mid| {
                                                    let content = responses.get(mid).cloned().unwrap_or_default();
                                                    // Check if content is an error message
                                                    let (actual_content, error) = if content.starts_with("Error: ") {
                                                        (String::new(), Some(content.strip_prefix("Error: ").unwrap_or(&content).to_string()))
                                                    } else {
                                                        (content, None)
                                                    };

                                                    ModelResponse {
                                                        model_id: mid.clone(),
                                                        content: actual_content,
                                                        error_message: error,
                                                    }
                                                })
                                                .collect()
                                        };

                                        model_responses_clone.write().push(final_responses);
                                        current_streaming_responses_clone.write().clear();
                                        is_streaming_clone.set(false);
                                        break;
                                    }
                                }
                                StreamEvent::Error(e) => {
                                    let mut responses = current_streaming_responses_clone.write();
                                    responses.insert(model_id.clone(), format!("Error: {}", e));
                                    done_models.insert(model_id);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        is_streaming_clone.set(false);
                        // Store error for all models
                        let mut responses = current_streaming_responses_clone.write();
                        for model_id in &models {
                            responses.insert(model_id.clone(), format!("Error: {}", e));
                        }
                        // Finalize with error responses
                        let final_responses: Vec<ModelResponse> = models
                            .iter()
                            .map(|mid| ModelResponse {
                                model_id: mid.clone(),
                                content: String::new(),
                                error_message: Some(e.clone()),
                            })
                            .collect();
                        model_responses_clone.write().push(final_responses);
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "flex flex-col h-full",

            // Show model selector if no models selected
            if selected_models.read().is_empty() {
                if let Some(client_arc) = &client {
                    ModelSelector {
                        theme,
                        client: client_arc.clone(),
                        on_models_selected,
                    }
                } else {
                    div {
                        class: "flex items-center justify-center h-full",
                        div {
                            class: "text-center",
                            p {
                                class: "text-[var(--color-base-content)]/70",
                                "No API client available"
                            }
                        }
                    }
                }
            } else {
                // Chat interface
                div {
                    class: "flex-1 overflow-y-auto p-4",

                    if user_messages.read().is_empty() {
                        // Empty state
                        div {
                            class: "flex flex-col items-center justify-center h-full",
                            div {
                                class: "text-4xl mb-4",
                                "💬"
                            }
                            h2 {
                                class: "text-xl font-bold text-[var(--color-base-content)] mb-2",
                                "Ready to Chat"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70 mb-4",
                                "{selected_models.read().len()} model(s) selected"
                            }
                            button {
                                onclick: move |_| selected_models.set(Vec::new()),
                                class: "text-sm text-[var(--color-primary)] hover:underline",
                                "Change Models"
                            }
                        }
                    } else {
                        // Messages display
                        div {
                            class: "space-y-6 max-w-6xl mx-auto",

                            for (idx, user_msg) in user_messages.read().iter().enumerate() {
                                div {
                                    key: "{idx}",

                                    // User message
                                    div {
                                        class: "flex justify-end mb-4",
                                        div {
                                            class: "max-w-[85%] bg-[var(--color-primary)] text-[var(--color-primary-content)] px-4 py-2 rounded-lg",
                                            "{user_msg}"
                                        }
                                    }

                                    // Model responses
                                    if let Some(responses) = model_responses.read().get(idx) {
                                        if selected_models.read().len() == 1 {
                                            // Single model - traditional chat display
                                            div {
                                                class: "flex justify-start",
                                                div {
                                                    class: if responses[0].error_message.is_some() {
                                                        "w-full max-w-[85%] bg-red-500/10 border-2 border-red-500/50 text-[var(--color-base-content)] px-4 py-3 rounded-lg"
                                                    } else {
                                                        "w-full max-w-[85%] bg-[var(--color-base-200)] text-[var(--color-base-content)] px-4 py-3 rounded-lg"
                                                    },
                                                    div {
                                                        class: "text-xs text-[var(--color-base-content)]/60 mb-2 flex items-center gap-1",
                                                        if responses[0].error_message.is_some() {
                                                            span { "⚠️" }
                                                        }
                                                        span { "{responses[0].model_id}" }
                                                    }
                                                    if let Some(error) = &responses[0].error_message {
                                                        div {
                                                            class: "text-sm text-[var(--color-base-content)] p-3 bg-red-500/20 rounded",
                                                            div {
                                                                class: "whitespace-pre-wrap mb-2",
                                                                "{error}"
                                                            }
                                                            if error.contains("data policy") || error.contains("data retention") {
                                                                a {
                                                                    href: "https://openrouter.ai/settings/privacy",
                                                                    target: "_blank",
                                                                    class: "text-xs text-[var(--color-primary)] hover:underline",
                                                                    "Configure Privacy Settings →"
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        div {
                                                            class: "whitespace-pre-wrap",
                                                            "{responses[0].content}"
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            // Multiple models - card grid
                                            div {
                                                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3",
                                                for response in responses {
                                                    div {
                                                        key: "{response.model_id}",
                                                        class: if response.error_message.is_some() {
                                                            "bg-red-500/10 rounded-lg p-3 border-2 border-red-500/50"
                                                        } else {
                                                            "bg-[var(--color-base-200)] rounded-lg p-3 border border-[var(--color-base-300)]"
                                                        },
                                                        div {
                                                            class: "text-xs font-semibold text-[var(--color-base-content)]/70 mb-2 truncate flex items-center gap-1",
                                                            if response.error_message.is_some() {
                                                                span { "⚠️" }
                                                            }
                                                            span { "{response.model_id}" }
                                                        }
                                                        if let Some(error) = &response.error_message {
                                                            div {
                                                                class: "text-xs text-[var(--color-base-content)] p-2 bg-red-500/20 rounded",
                                                                div {
                                                                    class: "whitespace-pre-wrap mb-2",
                                                                    "{error}"
                                                                }
                                                                if error.contains("data policy") || error.contains("data retention") {
                                                                    a {
                                                                        href: "https://openrouter.ai/settings/privacy",
                                                                        target: "_blank",
                                                                        class: "text-xs text-[var(--color-primary)] hover:underline",
                                                                        "Configure Privacy Settings →"
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            div {
                                                                class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap",
                                                                "{response.content}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else if idx == user_messages.read().len() - 1 && *is_streaming.read() {
                                        // Currently streaming responses
                                        {
                                            let streaming_responses = current_streaming_responses.read();
                                            let is_single_model = selected_models.read().len() == 1;
                                            let models = selected_models.read().clone();

                                            rsx! {
                                        if is_single_model {
                                            // Single model streaming
                                            div {
                                                class: "flex justify-start",
                                                div {
                                                    class: "w-full max-w-[85%] bg-[var(--color-base-200)] text-[var(--color-base-content)] px-4 py-3 rounded-lg",
                                                    div {
                                                        class: "text-xs text-[var(--color-base-content)]/60 mb-2 flex items-center gap-2",
                                                        span { "{models[0]}" }
                                                        span {
                                                            class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
                                                        }
                                                    }
                                                    div {
                                                        class: "whitespace-pre-wrap",
                                                        "{streaming_responses.get(&models[0]).cloned().unwrap_or_default()}"
                                                    }
                                                }
                                            }
                                        } else {
                                            // Multiple models streaming
                                            div {
                                                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3",
                                                for model_id in models.iter() {
                                                    div {
                                                        key: "{model_id}",
                                                        class: "bg-[var(--color-base-200)] rounded-lg p-3 border border-[var(--color-base-300)]",
                                                        div {
                                                            class: "text-xs font-semibold text-[var(--color-base-content)]/70 mb-2 truncate flex items-center gap-2",
                                                            span { "{model_id}" }
                                                            span {
                                                                class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
                                                            }
                                                        }
                                                        div {
                                                            class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap min-h-[3rem]",
                                                            "{streaming_responses.get(model_id).cloned().unwrap_or_default()}"
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

                // Input area
                ChatInput {
                    theme,
                    input_settings,
                    on_send: send_message,
                }
            }
        }
    }
}
