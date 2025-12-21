use super::common::{ChatInput, Modal, ModelSelector};
use crate::utils::{ChatMessage, ChatHistory, ChatMode, ChatSession, InputSettings, OpenRouterClient, SessionData, StandardHistory, StreamEvent, Theme};
use dioxus::prelude::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct ModelResponse {
    model_id: String,
    content: String,
    error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ConversationHistory {
    // For single model: Vec<(user_msg, assistant_msg)>
    single_model: Vec<(String, String)>,
    // For multi-model: HashMap<model_id, Vec<(user_msg, assistant_msg)>>
    multi_model: HashMap<String, Vec<(String, String)>>,
}

#[derive(Props, Clone)]
pub struct StandardProps {
    theme: Signal<Theme>,
    client: Option<Arc<OpenRouterClient>>,
    input_settings: Signal<InputSettings>,
    session_id: Option<String>,
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
    
    // System prompt state
    let mut system_prompt = use_signal(|| "You are a helpful AI assistant.".to_string());
    let mut system_prompt_editor_open = use_signal(|| false);
    let mut temp_system_prompt = use_signal(|| String::new());
    
    // Conversation history (per model for multi-model mode)
    let mut conversation_history = use_signal(|| ConversationHistory {
        single_model: Vec::new(),
        multi_model: HashMap::new(),
    });
    
    // Load history if session_id is provided
    let session_id = props.session_id.clone();
    use_hook(|| {
        if let Some(sid) = session_id {
            if let Ok(session_data) = ChatHistory::load_session(&sid) {
                if let ChatHistory::Standard(history) = session_data.history {
                    selected_models.set(history.selected_models);
                    user_messages.set(history.user_messages);
                    system_prompt.set(history.system_prompt);
                    
                    // Convert ModelResponse from history to internal format
                    let converted_responses: Vec<Vec<ModelResponse>> = history.model_responses
                        .into_iter()
                        .map(|responses| {
                            responses.into_iter()
                                .map(|r| ModelResponse {
                                    model_id: r.model_id,
                                    content: r.content,
                                    error_message: r.error_message,
                                })
                                .collect()
                        })
                        .collect();
                    model_responses.set(converted_responses);
                    
                    // Convert ConversationHistory
                    conversation_history.set(ConversationHistory {
                        single_model: history.conversation_history.single_model,
                        multi_model: history.conversation_history.multi_model,
                    });
                }
            }
        }
    });
    

    // Handle model selection
    let on_models_selected = move |models: Vec<String>| {
        selected_models.set(models.clone());
        // Initialize conversation history for each model
        let mut history = conversation_history.write();
        history.multi_model.clear();
        for model_id in &models {
            history.multi_model.insert(model_id.clone(), Vec::new());
        }
    };
    
    // System prompt editor handlers
    let open_system_prompt_editor = move |_| {
        temp_system_prompt.set(system_prompt());
        system_prompt_editor_open.set(true);
    };
    
    let save_system_prompt = move |_| {
        system_prompt.set(temp_system_prompt());
        system_prompt_editor_open.set(false);
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
            let is_single_model = models.len() == 1;
            let sys_prompt = system_prompt();
            let mut is_streaming_clone = is_streaming.clone();
            let mut current_streaming_responses_clone = current_streaming_responses.clone();
            let mut model_responses_clone = model_responses.clone();
            let mut conversation_history_clone = conversation_history.clone();
            let session_id_for_save = props.session_id.clone();

            spawn(async move {
                is_streaming_clone.set(true);
                current_streaming_responses_clone.write().clear();

                // For single model, use its history directly
                // For multiple models, we need to stream each separately with their own history
                // Since we can't use stream_chat_completion_multi with different messages per model,
                // we'll stream each model individually and aggregate results
                
                let mut final_results = HashMap::new();
                
                if is_single_model {
                    // Single model with shared history
                    let history = conversation_history_clone.read();
                    let mut messages = vec![ChatMessage::system(sys_prompt.clone())];
                    for (user_msg, assistant_msg) in &history.single_model {
                        messages.push(ChatMessage::user(user_msg.clone()));
                        messages.push(ChatMessage::assistant(assistant_msg.clone()));
                    }
                    messages.push(ChatMessage::user(text.clone()));
                    drop(history);
                    
                    let model_id = models[0].clone();
                    match client.stream_chat_completion(model_id.clone(), messages).await {
                        Ok(mut stream) => {
                            let mut content = String::new();
                            while let Some(event) = stream.next().await {
                                match event {
                                    StreamEvent::Content(chunk) => {
                                        content.push_str(&chunk);
                                        let mut responses = current_streaming_responses_clone.write();
                                        responses.insert(model_id.clone(), content.clone());
                                    }
                                    StreamEvent::Done => {
                                        final_results.insert(model_id.clone(), (content.clone(), None));
                                        break;
                                    }
                                    StreamEvent::Error(e) => {
                                        final_results.insert(model_id.clone(), (String::new(), Some(e)));
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            final_results.insert(model_id, (String::new(), Some(e)));
                        }
                    }
                } else {
                    // Multiple models, each with separate history
                    for model_id in &models {
                        let history = conversation_history_clone.read();
                        let mut messages = vec![ChatMessage::system(sys_prompt.clone())];
                        if let Some(model_history) = history.multi_model.get(model_id) {
                            for (user_msg, assistant_msg) in model_history {
                                messages.push(ChatMessage::user(user_msg.clone()));
                                messages.push(ChatMessage::assistant(assistant_msg.clone()));
                            }
                        }
                        messages.push(ChatMessage::user(text.clone()));
                        drop(history);
                        
                        match client.stream_chat_completion(model_id.clone(), messages).await {
                            Ok(mut stream) => {
                                let mut content = String::new();
                                while let Some(event) = stream.next().await {
                                    match event {
                                        StreamEvent::Content(chunk) => {
                                            content.push_str(&chunk);
                                            let mut responses = current_streaming_responses_clone.write();
                                            responses.insert(model_id.clone(), content.clone());
                                        }
                                        StreamEvent::Done => {
                                            final_results.insert(model_id.clone(), (content.clone(), None));
                                            break;
                                        }
                                        StreamEvent::Error(e) => {
                                            final_results.insert(model_id.clone(), (String::new(), Some(e)));
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                final_results.insert(model_id.clone(), (String::new(), Some(e)));
                            }
                        }
                    }
                }
                
                // Build final responses
                let mut final_responses: Vec<ModelResponse> = models
                    .iter()
                    .map(|model_id| {
                        let (content, error) = final_results.get(model_id)
                            .cloned()
                            .unwrap_or_else(|| (String::new(), Some("No response received".to_string())));
                        ModelResponse {
                            model_id: model_id.clone(),
                            content,
                            error_message: error,
                        }
                    })
                    .collect();
                
                // Update conversation history
                {
                    let mut history = conversation_history_clone.write();
                    if is_single_model {
                        if let Some(response) = final_responses.first() {
                            if response.error_message.is_none() {
                                history.single_model.push((text.clone(), response.content.clone()));
                            }
                        }
                    } else {
                        for response in &final_responses {
                            if response.error_message.is_none() {
                                if let Some(model_history) = history.multi_model.get_mut(&response.model_id) {
                                    model_history.push((text.clone(), response.content.clone()));
                                }
                            }
                        }
                    }
                }
                
                model_responses_clone.write().push(final_responses);
                current_streaming_responses_clone.write().clear();
                is_streaming_clone.set(false);
                
                // Auto-save if session_id is provided
                if let Some(sid) = session_id_for_save {
                    // Reconstruct history for saving
                    let history = StandardHistory {
                        user_messages: user_messages.read().clone(),
                        model_responses: model_responses_clone.read().iter()
                            .map(|responses| {
                                responses.iter()
                                    .map(|r| crate::utils::ModelResponse {
                                        model_id: r.model_id.clone(),
                                        content: r.content.clone(),
                                        error_message: r.error_message.clone(),
                                    })
                                    .collect()
                            })
                            .collect(),
                        selected_models: selected_models.read().clone(),
                        system_prompt: system_prompt.read().clone(),
                        conversation_history: crate::utils::ConversationHistory {
                            single_model: conversation_history_clone.read().single_model.clone(),
                            multi_model: conversation_history_clone.read().multi_model.clone(),
                        },
                    };
                    
                    let summary = ChatHistory::generate_chat_summary(&ChatHistory::Standard(history.clone()));
                    let session = ChatSession {
                        id: sid.clone(),
                        title: summary,
                        mode: ChatMode::Standard,
                        timestamp: ChatHistory::format_timestamp(),
                    };
                    
                    let session_data = SessionData {
                        session,
                        history: ChatHistory::Standard(history),
                        created_at: ChatHistory::format_timestamp(),
                        updated_at: ChatHistory::format_timestamp(),
                    };
                    
                    if let Err(e) = ChatHistory::save_session(&session_data) {
                        eprintln!("Failed to save session: {}", e);
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
                // System prompt header
                div {
                    class: "p-3 border-b border-[var(--color-base-300)] bg-[var(--color-base-100)]",
                    div {
                        class: "flex items-center justify-between",
                        h3 {
                            class: "text-sm font-semibold text-[var(--color-base-content)]",
                            "System Prompt"
                        }
                        div {
                            class: "flex items-center gap-2",
                            button {
                                onclick: open_system_prompt_editor,
                                class: "text-xs text-[var(--color-primary)] hover:underline",
                                "Edit"
                            }
                            button {
                                onclick: move |_| selected_models.set(Vec::new()),
                                class: "text-xs text-[var(--color-primary)] hover:underline",
                                "Change Models"
                            }
                        }
                    }
                    div {
                        class: "text-xs text-[var(--color-base-content)]/70 mt-1 truncate",
                        "{system_prompt}"
                    }
                }
                
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
            
            // System Prompt Editor Modal
            Modal {
                theme,
                open: system_prompt_editor_open,
                on_close: move |_| {
                    temp_system_prompt.set(system_prompt());
                    system_prompt_editor_open.set(false);
                },
                
                div {
                    class: "p-6",
                    
                    // Header
                    div {
                        class: "flex items-start justify-between mb-4",
                        div {
                            h2 {
                                class: "text-xl font-bold text-[var(--color-base-content)]",
                                "Edit System Prompt"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70 mt-1",
                                "The system prompt sets the behavior and personality of the AI assistant."
                            }
                        }
                        button {
                            class: "text-2xl text-[var(--color-base-content)]/70 hover:text-[var(--color-base-content)] transition-colors",
                            onclick: move |_| {
                                temp_system_prompt.set(system_prompt());
                                system_prompt_editor_open.set(false);
                            },
                            "×"
                        }
                    }
                    
                    // Prompt editor textarea
                    div {
                        class: "mb-4",
                        textarea {
                            value: "{temp_system_prompt}",
                            oninput: move |evt| temp_system_prompt.set(evt.value()),
                            rows: "10",
                            class: "w-full p-3 border-2 rounded-lg font-mono text-sm bg-[var(--color-base-100)] text-[var(--color-base-content)] border-[var(--color-base-300)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent resize-y min-h-[200px]",
                            placeholder: "Enter system prompt...",
                            autofocus: true,
                        }
                    }
                    
                    // Character count
                    div {
                        class: "text-xs text-[var(--color-base-content)]/50 mb-4 text-right",
                        "{temp_system_prompt.read().len()} characters"
                    }
                    
                    // Action buttons
                    div {
                        class: "flex justify-between items-center gap-3",
                        button {
                            onclick: move |_| {
                                temp_system_prompt.set("You are a helpful AI assistant.".to_string());
                            },
                            class: "px-4 py-2 text-sm rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] text-[var(--color-base-content)] hover:bg-[var(--color-base-300)] transition-colors",
                            "Reset to Default"
                        }
                        div {
                            class: "flex gap-2",
                            button {
                                onclick: move |_| {
                                    temp_system_prompt.set(system_prompt());
                                    system_prompt_editor_open.set(false);
                                },
                                class: "px-4 py-2 text-sm rounded border border-[var(--color-base-300)] bg-[var(--color-base-200)] text-[var(--color-base-content)] hover:bg-[var(--color-base-300)] transition-colors",
                                "Cancel"
                            }
                            button {
                                onclick: save_system_prompt,
                                class: "px-4 py-2 text-sm rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 transition-colors font-medium",
                                "Save Prompt"
                            }
                        }
                    }
                }
            }
        }
    }
}
