use super::common::ChatInput;
use crate::utils::{ChatMessage, InputSettings, Model, OpenRouterClient, StreamEvent, Theme};
use dioxus::prelude::*;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct BotResponse {
    model_id: String,
    content: String,
    error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ConversationRound {
    user_message: String,
    bot1_response: BotResponse,
    bot2_response: BotResponse,
    moderator_judgment: Option<ModeratorResponse>,
}

#[derive(Clone, Debug, PartialEq)]
struct ModeratorResponse {
    content: String,
    error_message: Option<String>,
}

#[derive(Props, Clone)]
pub struct PvPProps {
    theme: Signal<Theme>,
    client: Option<Arc<OpenRouterClient>>,
    input_settings: Signal<InputSettings>,
}

impl PartialEq for PvPProps {
    fn eq(&self, other: &Self) -> bool {
        self.theme == other.theme && self.input_settings == other.input_settings
    }
}

#[component]
pub fn PvP(props: PvPProps) -> Element {
    let theme = props.theme;
    let client = props.client.clone();
    let client_for_send = props.client;
    let input_settings = props.input_settings;
    let _ = theme.read();

    // Model selection state
    let mut bot_models = use_signal(|| Vec::<String>::new());
    let mut moderator_model = use_signal(|| None::<String>);
    let mut selection_step = use_signal(|| 0); // 0 = select bots, 1 = select moderator, 2 = chat

    // Model list state
    let mut available_models = use_signal(|| None::<Result<Vec<Model>, String>>);
    let mut search_query = use_signal(|| String::new());

    // Chat state
    let mut conversation_history = use_signal(|| Vec::<ConversationRound>::new());
    let mut is_streaming_bots = use_signal(|| false);
    let mut is_streaming_moderator = use_signal(|| false);
    let mut current_bot_responses = use_signal(|| HashMap::<String, String>::new());
    let mut current_moderator_response = use_signal(|| String::new());

    // Fetch models on component mount
    let _fetch = use_hook(|| {
        if let Some(client_arc) = &client {
            let client_clone = client_arc.clone();
            let mut models_clone = available_models.clone();
            spawn(async move {
                let result = client_clone.fetch_models().await;
                models_clone.set(Some(result));
            });
        }
    });

    // Toggle bot model selection (max 2)
    let mut toggle_bot_model = move |model_id: String| {
        let mut selected = bot_models.write();
        if let Some(pos) = selected.iter().position(|id| id == &model_id) {
            selected.remove(pos);
        } else if selected.len() < 2 {
            selected.push(model_id);
        }
    };

    // Select moderator (only 1)
    let mut select_moderator = move |model_id: String| {
        let current = moderator_model.read().clone();
        if current.as_ref() == Some(&model_id) {
            moderator_model.set(None);
        } else {
            moderator_model.set(Some(model_id));
        }
    };

    // Proceed to moderator selection
    let proceed_to_moderator = move |_| {
        if bot_models.read().len() == 2 {
            selection_step.set(1);
            search_query.set(String::new());
        }
    };

    // Go back to bot selection
    let back_to_bots = move |_| {
        selection_step.set(0);
        search_query.set(String::new());
    };

    // Start chat
    let start_chat = move |_| {
        if bot_models.read().len() == 2 && moderator_model.read().is_some() {
            selection_step.set(2);
        }
    };

    // Send message handler
    let send_message = move |text: String| {
        if text.trim().is_empty() || *is_streaming_bots.read() || *is_streaming_moderator.read() {
            return;
        }

        let bot1_id = bot_models.read()[0].clone();
        let bot2_id = bot_models.read()[1].clone();
        let mod_id = moderator_model.read().clone().unwrap();

        if let Some(client_arc) = &client_for_send {
            let client = client_arc.clone();
            let user_msg = text.clone();
            let mut is_streaming_bots_clone = is_streaming_bots.clone();
            let mut is_streaming_moderator_clone = is_streaming_moderator.clone();
            let mut current_bot_responses_clone = current_bot_responses.clone();
            let mut current_moderator_response_clone = current_moderator_response.clone();
            let mut conversation_history_clone = conversation_history.clone();

            // Immediately add the user message and empty bot responses to show in UI
            conversation_history_clone.write().push(ConversationRound {
                user_message: user_msg.clone(),
                bot1_response: BotResponse {
                    model_id: bot1_id.clone(),
                    content: String::new(),
                    error_message: None,
                },
                bot2_response: BotResponse {
                    model_id: bot2_id.clone(),
                    content: String::new(),
                    error_message: None,
                },
                moderator_judgment: None,
            });

            spawn(async move {
                is_streaming_bots_clone.set(true);
                current_bot_responses_clone.write().clear();

                // Send to both bots in parallel
                let messages = vec![ChatMessage::user(user_msg.clone())];
                let bot_ids = vec![bot1_id.clone(), bot2_id.clone()];

                match client.stream_chat_completion_multi(bot_ids.clone(), messages).await {
                    Ok(mut rx) => {
                        let mut done_bots = std::collections::HashSet::new();

                        // Collect bot responses
                        while let Some(event) = rx.recv().await {
                            let model_id = event.model_id.clone();

                            match event.event {
                                StreamEvent::Content(content) => {
                                    let mut responses = current_bot_responses_clone.write();
                                    responses
                                        .entry(model_id.clone())
                                        .and_modify(|s| s.push_str(&content))
                                        .or_insert(content);
                                }
                                StreamEvent::Done => {
                                    done_bots.insert(model_id);

                                    // Check if both bots are done
                                    if done_bots.len() >= 2 {
                                        is_streaming_bots_clone.set(false);

                                        // Get final bot responses
                                        let (bot1_final, bot1_error, bot2_final, bot2_error) = {
                                            let responses = current_bot_responses_clone.read();
                                            let bot1_content = responses.get(&bot1_id).cloned().unwrap_or_default();
                                            let bot2_content = responses.get(&bot2_id).cloned().unwrap_or_default();

                                            // Check for errors
                                            let (bot1_final, bot1_error) = if bot1_content.starts_with("Error: ") {
                                                (String::new(), Some(bot1_content.strip_prefix("Error: ").unwrap_or(&bot1_content).to_string()))
                                            } else {
                                                (bot1_content, None)
                                            };

                                            let (bot2_final, bot2_error) = if bot2_content.starts_with("Error: ") {
                                                (String::new(), Some(bot2_content.strip_prefix("Error: ").unwrap_or(&bot2_content).to_string()))
                                            } else {
                                                (bot2_content, None)
                                            };

                                            (bot1_final, bot1_error, bot2_final, bot2_error)
                                        };

                                        // Update the last conversation round with bot responses
                                        if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                            last_round.bot1_response = BotResponse {
                                                model_id: bot1_id.clone(),
                                                content: bot1_final.clone(),
                                                error_message: bot1_error.clone(),
                                            };
                                            last_round.bot2_response = BotResponse {
                                                model_id: bot2_id.clone(),
                                                content: bot2_final.clone(),
                                                error_message: bot2_error.clone(),
                                            };
                                        }

                                        // Now send to moderator if both bots succeeded
                                        if bot1_error.is_none() && bot2_error.is_none() {
                                            is_streaming_moderator_clone.set(true);
                                            current_moderator_response_clone.set(String::new());

                                            let moderator_prompt = format!(
                                                "You are a moderator judging a debate between two AI models.\n\n\
                                                User Question: {}\n\n\
                                                {} Response:\n{}\n\n\
                                                {} Response:\n{}\n\n\
                                                Please evaluate both responses and determine which one is better. \
                                                Explain your reasoning and declare a winner. Be specific about what makes \
                                                one response superior to the other.",
                                                user_msg, bot1_id, bot1_final, bot2_id, bot2_final
                                            );

                                            let moderator_messages = vec![ChatMessage::user(moderator_prompt)];

                                            match client.stream_chat_completion(mod_id.clone(), moderator_messages).await {
                                                Ok(mut stream) => {
                                                    let mut mod_content = String::new();

                                                    while let Some(event) = stream.next().await {
                                                        match event {
                                                            StreamEvent::Content(content) => {
                                                                mod_content.push_str(&content);
                                                                current_moderator_response_clone.set(mod_content.clone());
                                                            }
                                                            StreamEvent::Done => {
                                                                // Update the last conversation round with moderator response
                                                                if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                                                    last_round.moderator_judgment = Some(ModeratorResponse {
                                                                        content: mod_content.clone(),
                                                                        error_message: None,
                                                                    });
                                                                }
                                                                current_moderator_response_clone.set(String::new());
                                                                is_streaming_moderator_clone.set(false);
                                                                break;
                                                            }
                                                            StreamEvent::Error(e) => {
                                                                if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                                                    last_round.moderator_judgment = Some(ModeratorResponse {
                                                                        content: String::new(),
                                                                        error_message: Some(e.clone()),
                                                                    });
                                                                }
                                                                current_moderator_response_clone.set(String::new());
                                                                is_streaming_moderator_clone.set(false);
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                                        last_round.moderator_judgment = Some(ModeratorResponse {
                                                            content: String::new(),
                                                            error_message: Some(e),
                                                        });
                                                    }
                                                    is_streaming_moderator_clone.set(false);
                                                }
                                            }
                                        } else {
                                            // If either bot had an error, don't call moderator
                                            current_bot_responses_clone.write().clear();
                                        }

                                        break;
                                    }
                                }
                                StreamEvent::Error(e) => {
                                    let mut responses = current_bot_responses_clone.write();
                                    responses.insert(model_id.clone(), format!("Error: {}", e));
                                    done_bots.insert(model_id);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        is_streaming_bots_clone.set(false);

                        // Update the last conversation round with error responses
                        if let Some(last_round) = conversation_history_clone.write().last_mut() {
                            last_round.bot1_response = BotResponse {
                                model_id: bot1_id,
                                content: String::new(),
                                error_message: Some(e.clone()),
                            };
                            last_round.bot2_response = BotResponse {
                                model_id: bot2_id,
                                content: String::new(),
                                error_message: Some(e),
                            };
                        }
                    }
                }
            });
        }
    };

    // Get filtered models for display
    let models_result = available_models.read();
    let loading = models_result.is_none();
    let (models_list, error) = match &*models_result {
        Some(Ok(models)) => (models.clone(), None),
        Some(Err(e)) => (Vec::new(), Some(e.clone())),
        None => (Vec::new(), None),
    };

    let filtered_models: Vec<Model> = {
        let search = search_query.read().to_lowercase();
        if search.is_empty() {
            models_list.clone()
        } else {
            models_list
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
            class: "flex flex-col h-full",

            // Model Selection Steps
            if *selection_step.read() < 2 {
                if let Some(client_arc) = &client {
                    // Step indicator
                    div {
                        class: "p-4 border-b border-[var(--color-base-300)]",
                        div {
                            class: "flex items-center justify-center gap-4 mb-4",
                            div {
                                class: if *selection_step.read() == 0 {
                                    "flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--color-primary)] text-[var(--color-primary-content)]"
                                } else {
                                    "flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--color-base-300)] text-[var(--color-base-content)]"
                                },
                                span { "1" }
                                span { class: "text-xs font-medium", "Select Bots" }
                            }
                            div { class: "text-[var(--color-base-content)]/30", "→" }
                            div {
                                class: if *selection_step.read() == 1 {
                                    "flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--color-primary)] text-[var(--color-primary-content)]"
                                } else {
                                    "flex items-center gap-2 px-3 py-1 rounded-full bg-[var(--color-base-300)] text-[var(--color-base-content)]"
                                },
                                span { "2" }
                                span { class: "text-xs font-medium", "Select Moderator" }
                            }
                        }

                        if *selection_step.read() == 0 {
                            h2 {
                                class: "text-lg font-bold text-[var(--color-base-content)] mb-1",
                                "Select 2 Competing Bots"
                            }
                            p {
                                class: "text-xs text-[var(--color-base-content)]/70",
                                "Choose exactly 2 AI models that will compete by answering your questions."
                            }
                        } else {
                            h2 {
                                class: "text-lg font-bold text-[var(--color-base-content)] mb-1",
                                "Select 1 Moderator"
                            }
                            p {
                                class: "text-xs text-[var(--color-base-content)]/70",
                                "Choose 1 AI model that will judge which bot gives the better response."
                            }
                        }
                    }

                    // Search box
                    if !loading && error.is_none() {
                        div {
                            class: "px-4 pt-2",
                            input {
                                r#type: "text",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value().clone()),
                                placeholder: "Search models...",
                                class: "w-full px-3 py-2 text-sm rounded bg-[var(--color-base-100)] text-[var(--color-base-content)] border border-[var(--color-base-300)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent",
                            }
                        }
                    }

                    // Model list
                    div {
                        class: "flex-1 overflow-y-auto p-4",

                        if loading {
                            div {
                                class: "flex items-center justify-center h-full",
                                div {
                                    class: "text-center",
                                    div { class: "text-4xl mb-4", "⏳" }
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
                                    div { class: "text-4xl mb-4", "⚠️" }
                                    p {
                                        class: "text-red-500 mb-2",
                                        "{err}"
                                    }
                                    p {
                                        class: "text-sm text-[var(--color-base-content)]/70",
                                        "Please check your API key in settings."
                                    }
                                }
                            }
                        } else {
                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2",

                                for model in filtered_models.iter() {
                                    {
                                        let model_id = model.id.clone();
                                        let display_name = model.display_name();

                                        if *selection_step.read() == 0 {
                                            // Bot selection
                                            let is_selected = bot_models.read().contains(&model_id);

                                            rsx! {
                                                button {
                                                    key: "{model_id}",
                                                    onclick: move |_| toggle_bot_model(model_id.clone()),
                                                    class: if is_selected {
                                                        "p-3 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/10 transition-all text-left"
                                                    } else {
                                                        "p-3 rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] hover:border-[var(--color-primary)]/50 transition-all text-left"
                                                    },
                                                    div {
                                                        class: "flex items-start gap-2",
                                                        div {
                                                            class: "flex-shrink-0 mt-0.5",
                                                            if is_selected {
                                                                span { class: "text-[var(--color-primary)] text-sm", "✓" }
                                                            } else {
                                                                span { class: "text-[var(--color-base-content)]/30 text-sm", "○" }
                                                            }
                                                        }
                                                        div {
                                                            class: "flex-1 min-w-0",
                                                            div {
                                                                class: "font-semibold text-sm text-[var(--color-base-content)] truncate",
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
                                        } else {
                                            // Moderator selection
                                            let is_selected = moderator_model.read().as_ref() == Some(&model_id);

                                            rsx! {
                                                button {
                                                    key: "{model_id}",
                                                    onclick: move |_| select_moderator(model_id.clone()),
                                                    class: if is_selected {
                                                        "p-3 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/10 transition-all text-left"
                                                    } else {
                                                        "p-3 rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] hover:border-[var(--color-primary)]/50 transition-all text-left"
                                                    },
                                                    div {
                                                        class: "flex items-start gap-2",
                                                        div {
                                                            class: "flex-shrink-0 mt-0.5",
                                                            if is_selected {
                                                                span { class: "text-[var(--color-primary)] text-sm", "✓" }
                                                            } else {
                                                                span { class: "text-[var(--color-base-content)]/30 text-sm", "○" }
                                                            }
                                                        }
                                                        div {
                                                            class: "flex-1 min-w-0",
                                                            div {
                                                                class: "font-semibold text-sm text-[var(--color-base-content)] truncate",
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
                            }

                            if filtered_models.is_empty() {
                                div {
                                    class: "text-center py-8",
                                    p {
                                        class: "text-[var(--color-base-content)]/70",
                                        "No models found."
                                    }
                                }
                            }
                        }
                    }

                    // Footer buttons
                    if !loading && error.is_none() {
                        div {
                            class: "p-4 border-t border-[var(--color-base-300)]",
                            div {
                                class: "flex items-center justify-between gap-2",

                                if *selection_step.read() == 0 {
                                    div {
                                        class: "text-sm text-[var(--color-base-content)]/70",
                                        "{bot_models.read().len()} of 2 bots selected"
                                    }
                                    button {
                                        onclick: proceed_to_moderator,
                                        disabled: bot_models.read().len() != 2,
                                        class: "px-4 py-2 text-sm rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 disabled:opacity-50 disabled:cursor-not-allowed font-medium transition-all",
                                        "Next: Select Moderator"
                                    }
                                } else {
                                    button {
                                        onclick: back_to_bots,
                                        class: "px-4 py-2 text-sm rounded border border-[var(--color-base-300)] text-[var(--color-base-content)] hover:bg-[var(--color-base-200)] font-medium transition-all",
                                        "← Back"
                                    }
                                    div {
                                        class: "flex items-center gap-3",
                                        div {
                                            class: "text-sm text-[var(--color-base-content)]/70",
                                            if moderator_model.read().is_some() {
                                                "1 moderator selected"
                                            } else {
                                                "Select a moderator"
                                            }
                                        }
                                        button {
                                            onclick: start_chat,
                                            disabled: moderator_model.read().is_none(),
                                            class: "px-4 py-2 text-sm rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 disabled:opacity-50 disabled:cursor-not-allowed font-medium transition-all",
                                            "Start PvP Chat"
                                        }
                                    }
                                }
                            }
                        }
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

                    if conversation_history.read().is_empty() {
                        // Empty state
                        div {
                            class: "flex flex-col items-center justify-center h-full",
                            h2 {
                                class: "text-xl font-bold text-[var(--color-base-content)] mb-2",
                                "PvP Arena Ready"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70 mb-2",
                                "Competitor 1: {bot_models.read()[0]}"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70 mb-2",
                                "Competitor 2: {bot_models.read()[1]}"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70 mb-4",
                                "Moderator: {moderator_model.read().as_ref().unwrap()}"
                            }
                            button {
                                onclick: move |_| { selection_step.set(0); conversation_history.write().clear(); },
                                class: "text-sm text-[var(--color-primary)] hover:underline",
                                "Change Models"
                            }
                        }
                    } else {
                        // Conversation display
                        div {
                            class: "space-y-6 max-w-6xl mx-auto",

                            for (idx, round) in conversation_history.read().iter().enumerate() {
                                div {
                                    key: "{idx}",

                                    // User message
                                    div {
                                        class: "flex justify-end mb-4",
                                        div {
                                            class: "max-w-[85%] bg-[var(--color-primary)] text-[var(--color-primary-content)] px-4 py-2 rounded-lg",
                                            "{round.user_message}"
                                        }
                                    }

                                    // Bot responses in a grid
                                    div {
                                        class: "grid grid-cols-1 md:grid-cols-2 gap-3 mb-4",

                                        // Bot 1
                                        div {
                                            class: if round.bot1_response.error_message.is_some() {
                                                "bg-red-500/10 rounded-lg p-4 border-2 border-red-500/50"
                                            } else {
                                                "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]"
                                            },
                                            div {
                                                class: "text-sm font-bold text-[var(--color-base-content)] mb-2 truncate",
                                                "{round.bot1_response.model_id}"
                                            }
                                            if let Some(error) = &round.bot1_response.error_message {
                                                div {
                                                    class: "text-sm text-red-500",
                                                    "Error: {error}"
                                                }
                                            } else {
                                                div {
                                                    class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap",
                                                    "{round.bot1_response.content}"
                                                }
                                            }
                                        }

                                        // Bot 2
                                        div {
                                            class: if round.bot2_response.error_message.is_some() {
                                                "bg-red-500/10 rounded-lg p-4 border-2 border-red-500/50"
                                            } else {
                                                "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]"
                                            },
                                            div {
                                                class: "text-sm font-bold text-[var(--color-base-content)] mb-2 truncate",
                                                "{round.bot2_response.model_id}"
                                            }
                                            if let Some(error) = &round.bot2_response.error_message {
                                                div {
                                                    class: "text-sm text-red-500",
                                                    "Error: {error}"
                                                }
                                            } else {
                                                div {
                                                    class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap",
                                                    "{round.bot2_response.content}"
                                                }
                                            }
                                        }
                                    }

                                    // Moderator judgment
                                    if let Some(judgment) = &round.moderator_judgment {
                                        div {
                                            class: if judgment.error_message.is_some() {
                                                "bg-red-500/10 rounded-lg p-4 border-2 border-red-500/50"
                                            } else {
                                                "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]"
                                            },
                                            div {
                                                class: "text-sm font-bold text-[var(--color-base-content)] mb-2",
                                                "Moderator Judgment ({moderator_model.read().as_ref().unwrap()})"
                                            }
                                            if let Some(error) = &judgment.error_message {
                                                div {
                                                    class: "text-sm text-red-500",
                                                    "Error: {error}"
                                                }
                                            } else {
                                                div {
                                                    class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap",
                                                    "{judgment.content}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Streaming indicators
                            if *is_streaming_bots.read() || *is_streaming_moderator.read() {
                                div {
                                    if *is_streaming_bots.read() {
                                        div {
                                            class: "grid grid-cols-1 md:grid-cols-2 gap-3 mb-4",

                                            // Bot 1 streaming
                                            div {
                                                class: "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]",
                                                div {
                                                    class: "text-sm font-bold text-[var(--color-base-content)] mb-2 flex items-center gap-2 truncate",
                                                    span { "{bot_models.read()[0]}" }
                                                    span {
                                                        class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse flex-shrink-0"
                                                    }
                                                }
                                                div {
                                                    class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap min-h-[3rem]",
                                                    "{current_bot_responses.read().get(&bot_models.read()[0]).cloned().unwrap_or_default()}"
                                                }
                                            }

                                            // Bot 2 streaming
                                            div {
                                                class: "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]",
                                                div {
                                                    class: "text-sm font-bold text-[var(--color-base-content)] mb-2 flex items-center gap-2 truncate",
                                                    span { "{bot_models.read()[1]}" }
                                                    span {
                                                        class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse flex-shrink-0"
                                                    }
                                                }
                                                div {
                                                    class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap min-h-[3rem]",
                                                    "{current_bot_responses.read().get(&bot_models.read()[1]).cloned().unwrap_or_default()}"
                                                }
                                            }
                                        }
                                    }

                                    if *is_streaming_moderator.read() {
                                        div {
                                            class: "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]",
                                            div {
                                                class: "text-sm font-bold text-[var(--color-base-content)] mb-2 flex items-center gap-2",
                                                span { "Moderator Judgment ({moderator_model.read().as_ref().unwrap()})" }
                                                span {
                                                    class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
                                                }
                                            }
                                            div {
                                                class: "text-sm text-[var(--color-base-content)] whitespace-pre-wrap min-h-[3rem]",
                                                "{current_moderator_response}"
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
