use super::common::{ChatInput, FormattedText, PromptCard, PromptEditorModal, PromptType};
use crate::utils::{
    ChatHistory, ChatMessage, ChatMode, ChatSession, InputSettings, Model, OpenRouterClient, SessionData, StreamEvent, Theme,
};
use dioxus::prelude::*;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
struct PromptTemplates {
    initial_response: String,
    cross_review: String,
    consensus: String,
}

impl Default for PromptTemplates {
    fn default() -> Self {
        Self {
            initial_response: "You are part of a collaborative AI team working together to answer questions. Provide your best answer to this question:\n\n{user_question}".to_string(),

            cross_review: "Review the following responses from other AI models. Provide constructive feedback on their strengths and areas for improvement.\n\nUser Question: {user_question}\n\nOther responses:\n{other_responses}\n\nProvide your analysis:".to_string(),

            consensus: "Based on all the initial responses and reviews below, synthesize a final collaborative answer that combines the best insights from all models.\n\nUser Question: {user_question}\n\nInitial Responses:\n{initial_responses}\n\nReviews:\n{reviews}\n\nSynthesize the best collaborative answer:".to_string(),
        }
    }
}

impl PromptTemplates {
    fn get(&self, prompt_type: PromptType) -> String {
        match prompt_type {
            PromptType::Initial => self.initial_response.clone(),
            PromptType::Review => self.cross_review.clone(),
            PromptType::Consensus => self.consensus.clone(),
        }
    }

    fn set(&mut self, prompt_type: PromptType, value: String) {
        match prompt_type {
            PromptType::Initial => self.initial_response = value,
            PromptType::Review => self.cross_review = value,
            PromptType::Consensus => self.consensus = value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ModelResponse {
    model_id: String,
    content: String,
    error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct CollaborativeRound {
    user_question: String,
    phase1_responses: Vec<ModelResponse>,
    phase2_reviews: Vec<ModelResponse>,
    phase3_consensus: Option<ModelResponse>,
    current_phase: CollaborativePhase,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CollaborativePhase {
    Initial,
    Review,
    Consensus,
    Complete,
}

impl CollaborativePhase {
    fn name(&self) -> &'static str {
        match self {
            CollaborativePhase::Initial => "Phase 1: Initial Responses",
            CollaborativePhase::Review => "Phase 2: Cross-Review",
            CollaborativePhase::Consensus => "Phase 3: Consensus",
            CollaborativePhase::Complete => "Complete",
        }
    }

    fn badge_color(&self) -> &'static str {
        match self {
            CollaborativePhase::Initial => "bg-blue-500",
            CollaborativePhase::Review => "bg-purple-500",
            CollaborativePhase::Consensus => "bg-green-500",
            CollaborativePhase::Complete => "bg-gray-500",
        }
    }
}

// ============================================================================
// Component Props
// ============================================================================

#[derive(Props, Clone)]
pub struct CollaborativeProps {
    theme: Signal<Theme>,
    client: Option<Arc<OpenRouterClient>>,
    input_settings: Signal<InputSettings>,
    session_id: Option<String>,
}

impl PartialEq for CollaborativeProps {
    fn eq(&self, other: &Self) -> bool {
        self.theme == other.theme && self.input_settings == other.input_settings
    }
}

// ============================================================================
// Main Component
// ============================================================================

#[component]
pub fn Collaborative(props: CollaborativeProps) -> Element {
    let theme = props.theme;
    let client = props.client.clone();
    let client_for_send = props.client;
    let input_settings = props.input_settings;
    let _ = theme.read();

    // Prompt templates
    let mut prompt_templates = use_signal(|| PromptTemplates::default());
    let mut prompt_editor_open = use_signal(|| false);
    let mut editing_prompt_type = use_signal(|| PromptType::Initial);
    let mut temp_prompt = use_signal(|| String::new());

    // Model selection state
    let mut selected_models = use_signal(|| Vec::<String>::new());
    let mut selection_step = use_signal(|| 0); // 0 = select models, 1 = chat

    // Model list state
    let available_models = use_signal(|| None::<Result<Vec<Model>, String>>);
    let mut search_query = use_signal(|| String::new());

    // Chat state
    let mut conversation_history = use_signal(|| Vec::<CollaborativeRound>::new());
    let current_phase = use_signal(|| CollaborativePhase::Initial);
    let is_processing = use_signal(|| false);
    let current_streaming_responses = use_signal(|| HashMap::<String, String>::new());

    // Load history if session_id is provided
    let session_id = props.session_id.clone();
    use_hook(|| {
        if let Some(sid) = session_id {
            if let Ok(session_data) = ChatHistory::load_session(&sid) {
                if let ChatHistory::Collaborative(history) = session_data.history {
                    selected_models.set(history.selected_models.clone());
                    // Convert history rounds to internal format
                    let converted_rounds: Vec<CollaborativeRound> = history.rounds
                        .into_iter()
                        .map(|r| {
                            // Convert chat_history::ModelResponse to internal ModelResponse
                            let phase1_responses: Vec<ModelResponse> = r.model_responses
                                .into_iter()
                                .map(|mr| ModelResponse {
                                    model_id: mr.model_id,
                                    content: mr.content,
                                    error_message: mr.error_message,
                                })
                                .collect();
                            // For now, we'll reconstruct from the simplified history format
                            // The history only stores final consensus, so we'll mark as complete
                            CollaborativeRound {
                                user_question: r.user_message,
                                phase1_responses,
                                phase2_reviews: vec![], // Not stored in simplified format
                                phase3_consensus: r.final_consensus.as_ref().map(|consensus| ModelResponse {
                                    model_id: "consensus".to_string(),
                                    content: consensus.clone(),
                                    error_message: None,
                                }),
                                current_phase: CollaborativePhase::Complete,
                            }
                        })
                        .collect();
                    conversation_history.set(converted_rounds);
                    if !history.selected_models.is_empty() {
                        selection_step.set(1);
                    }
                }
            }
        }
    });

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

    // Toggle model selection
    let mut toggle_model = move |model_id: String| {
        let mut selected = selected_models.write();
        if let Some(pos) = selected.iter().position(|id| id == &model_id) {
            selected.remove(pos);
        } else {
            selected.push(model_id);
        }
    };

    // Start chat
    let start_chat = move |_| {
        if selected_models.read().len() >= 2 {
            selection_step.set(1);
        }
    };

    // Edit prompt handlers
    let mut open_prompt_editor = move |ptype: PromptType| {
        editing_prompt_type.set(ptype);
        temp_prompt.set(prompt_templates.read().get(ptype));
        prompt_editor_open.set(true);
    };

    let save_prompt = move |new_prompt: String| {
        let mut templates = prompt_templates.write();
        templates.set(*editing_prompt_type.read(), new_prompt);
    };

    // Send message handler
    let send_message = move |text: String| {
        if text.trim().is_empty() || *is_processing.read() {
            return;
        }

        let models = selected_models.read().clone();
        if models.is_empty() {
            return;
        }

        if let Some(client_arc) = &client_for_send {
            let client = client_arc.clone();
            let user_msg = text.clone();
            let mut is_processing_clone = is_processing.clone();
            let mut current_phase_clone = current_phase.clone();
            let mut current_streaming_clone = current_streaming_responses.clone();
            let mut conversation_history_clone = conversation_history.clone();
            let templates = prompt_templates.read().clone();
            let session_id_for_save = props.session_id.clone();
            let selected_models_for_save = selected_models.read().clone();

            // Initialize new round
            conversation_history_clone.write().push(CollaborativeRound {
                user_question: user_msg.clone(),
                phase1_responses: vec![],
                phase2_reviews: vec![],
                phase3_consensus: None,
                current_phase: CollaborativePhase::Initial,
            });

            spawn(async move {
                is_processing_clone.set(true);
                current_phase_clone.set(CollaborativePhase::Initial);
                current_streaming_clone.write().clear();

                // ========================================================
                // PHASE 1: Initial Responses (Parallel)
                // ========================================================

                let initial_prompt = templates.initial_response
                    .replace("{user_question}", &user_msg);

                let messages = vec![ChatMessage::user(initial_prompt)];

                match client.stream_chat_completion_multi(models.clone(), messages).await {
                    Ok(mut rx) => {
                        let mut done_models = std::collections::HashSet::new();
                        let mut phase1_results: HashMap<String, ModelResponse> = HashMap::new();

                        let mut last_update = std::time::Instant::now();
                        while let Some(event) = rx.recv().await {
                            let model_id = event.model_id.clone();

                            match event.event {
                                StreamEvent::Content(content) => {
                                    let mut responses = current_streaming_clone.write();
                                    responses
                                        .entry(model_id.clone())
                                        .and_modify(|s| s.push_str(&content))
                                        .or_insert(content);
                                    
                                    // Yield to UI thread every ~16ms (60fps) to prevent blocking
                                    if last_update.elapsed().as_millis() >= 16 {
                                        tokio::task::yield_now().await;
                                        last_update = std::time::Instant::now();
                                    }
                                }
                                StreamEvent::Done => {
                                    let final_content = current_streaming_clone
                                        .read()
                                        .get(&model_id)
                                        .cloned()
                                        .unwrap_or_default();

                                    phase1_results.insert(
                                        model_id.clone(),
                                        ModelResponse {
                                            model_id: model_id.clone(),
                                            content: final_content,
                                            error_message: None,
                                        },
                                    );
                                    done_models.insert(model_id.clone());

                                    if done_models.len() >= models.len() {
                                        break;
                                    }
                                }
                                StreamEvent::Error(e) => {
                                    phase1_results.insert(
                                        model_id.clone(),
                                        ModelResponse {
                                            model_id: model_id.clone(),
                                            content: String::new(),
                                            error_message: Some(e),
                                        },
                                    );
                                    done_models.insert(model_id);
                                }
                            }
                        }

                        // Update conversation with Phase 1 results
                        if let Some(last_round) = conversation_history_clone.write().last_mut() {
                            last_round.phase1_responses = models
                                .iter()
                                .filter_map(|id| phase1_results.get(id).cloned())
                                .collect();
                        }

                        current_streaming_clone.write().clear();

                        // ========================================================
                        // PHASE 2: Cross-Review (Sequential per model)
                        // ========================================================

                        current_phase_clone.set(CollaborativePhase::Review);
                        if let Some(last_round) = conversation_history_clone.write().last_mut() {
                            last_round.current_phase = CollaborativePhase::Review;
                        }

                        let successful_phase1: Vec<_> = phase1_results
                            .values()
                            .filter(|r| r.error_message.is_none())
                            .collect();

                        if successful_phase1.len() >= 2 {
                            let mut phase2_results = Vec::new();

                            for model_id in &models {
                                // Build "other responses" text
                                let other_responses: String = successful_phase1
                                    .iter()
                                    .filter(|r| &r.model_id != model_id)
                                    .map(|r| format!("{}: {}", r.model_id, r.content))
                                    .collect::<Vec<_>>()
                                    .join("\n\n");

                                let review_prompt = templates.cross_review
                                    .replace("{user_question}", &user_msg)
                                    .replace("{other_responses}", &other_responses);

                                let review_messages = vec![ChatMessage::user(review_prompt)];

                                match client.stream_chat_completion(model_id.clone(), review_messages).await {
                                    Ok(mut stream) => {
                                        let mut review_content = String::new();
                                        let mut last_update = std::time::Instant::now();

                                        while let Some(event) = stream.next().await {
                                            match event {
                                                StreamEvent::Content(content) => {
                                                    review_content.push_str(&content);
                                                    current_streaming_clone.write().insert(
                                                        model_id.clone(),
                                                        review_content.clone(),
                                                    );
                                                    
                                                    // Yield to UI thread every ~16ms (60fps) to prevent blocking
                                                    if last_update.elapsed().as_millis() >= 16 {
                                                        tokio::task::yield_now().await;
                                                        last_update = std::time::Instant::now();
                                                    }
                                                }
                                                StreamEvent::Done => {
                                                    phase2_results.push(ModelResponse {
                                                        model_id: model_id.clone(),
                                                        content: review_content.clone(),
                                                        error_message: None,
                                                    });
                                                    break;
                                                }
                                                StreamEvent::Error(e) => {
                                                    phase2_results.push(ModelResponse {
                                                        model_id: model_id.clone(),
                                                        content: String::new(),
                                                        error_message: Some(e),
                                                    });
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        phase2_results.push(ModelResponse {
                                            model_id: model_id.clone(),
                                            content: String::new(),
                                            error_message: Some(e),
                                        });
                                    }
                                }

                                current_streaming_clone.write().remove(model_id);
                            }

                            // Update conversation with Phase 2 results
                            if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                last_round.phase2_reviews = phase2_results;
                            }

                            // ========================================================
                            // PHASE 3: Consensus Synthesis
                            // ========================================================

                            current_phase_clone.set(CollaborativePhase::Consensus);
                            if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                last_round.current_phase = CollaborativePhase::Consensus;
                            }

                            // Use first model as synthesizer
                            let synthesizer_id = &models[0];

                            let initial_responses_text: String = successful_phase1
                                .iter()
                                .map(|r| format!("{}: {}", r.model_id, r.content))
                                .collect::<Vec<_>>()
                                .join("\n\n");

                            let reviews_text: String = if let Some(round) = conversation_history_clone.read().last() {
                                round.phase2_reviews
                                    .iter()
                                    .filter(|r| r.error_message.is_none())
                                    .map(|r| format!("{}: {}", r.model_id, r.content))
                                    .collect::<Vec<_>>()
                                    .join("\n\n")
                            } else {
                                String::new()
                            };

                            let consensus_prompt = templates.consensus
                                .replace("{user_question}", &user_msg)
                                .replace("{initial_responses}", &initial_responses_text)
                                .replace("{reviews}", &reviews_text);

                            let consensus_messages = vec![ChatMessage::user(consensus_prompt)];

                            match client.stream_chat_completion(synthesizer_id.clone(), consensus_messages).await {
                                Ok(mut stream) => {
                                    let mut consensus_content = String::new();
                                    let mut last_update = std::time::Instant::now();

                                    while let Some(event) = stream.next().await {
                                        match event {
                                            StreamEvent::Content(content) => {
                                                consensus_content.push_str(&content);
                                                current_streaming_clone.write().insert(
                                                    "consensus".to_string(),
                                                    consensus_content.clone(),
                                                );
                                                
                                                // Yield to UI thread every ~16ms (60fps) to prevent blocking
                                                if last_update.elapsed().as_millis() >= 16 {
                                                    tokio::task::yield_now().await;
                                                    last_update = std::time::Instant::now();
                                                }
                                            }
                                            StreamEvent::Done => {
                                                if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                                    last_round.phase3_consensus = Some(ModelResponse {
                                                        model_id: synthesizer_id.clone(),
                                                        content: consensus_content,
                                                        error_message: None,
                                                    });
                                                    last_round.current_phase = CollaborativePhase::Complete;
                                                }
                                                break;
                                            }
                                            StreamEvent::Error(e) => {
                                                if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                                    last_round.phase3_consensus = Some(ModelResponse {
                                                        model_id: synthesizer_id.clone(),
                                                        content: String::new(),
                                                        error_message: Some(e),
                                                    });
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    if let Some(last_round) = conversation_history_clone.write().last_mut() {
                                        last_round.phase3_consensus = Some(ModelResponse {
                                            model_id: synthesizer_id.clone(),
                                            content: String::new(),
                                            error_message: Some(e),
                                        });
                                    }
                                }
                            }
                        }

                        current_streaming_clone.write().clear();
                        current_phase_clone.set(CollaborativePhase::Complete);
                        is_processing_clone.set(false);
                        
                        // Auto-save if session_id is provided
                        if let Some(sid) = session_id_for_save {
                            let history_rounds: Vec<crate::utils::CollaborativeRound> = conversation_history_clone.read()
                                .iter()
                                .map(|r| {
                                    // Convert internal format to history format
                                    // Use phase1_responses as model_responses, consensus content as final_consensus
                                    let model_responses: Vec<crate::utils::ModelResponse> = r.phase1_responses.iter()
                                        .map(|mr| crate::utils::ModelResponse {
                                            model_id: mr.model_id.clone(),
                                            content: mr.content.clone(),
                                            error_message: mr.error_message.clone(),
                                        })
                                        .collect();
                                    let final_consensus = r.phase3_consensus.as_ref().map(|c| c.content.clone());
                                    crate::utils::CollaborativeRound {
                                        user_message: r.user_question.clone(),
                                        model_responses,
                                        final_consensus,
                                    }
                                })
                                .collect();
                            
                            let history = crate::utils::CollaborativeHistory {
                                rounds: history_rounds,
                                selected_models: selected_models_for_save.clone(),
                                system_prompt: String::new(), // Collaborative doesn't use a single system prompt
                            };
                            
                            let summary = ChatHistory::generate_chat_summary(&ChatHistory::Collaborative(history.clone()));
                            let session = ChatSession {
                                id: sid.clone(),
                                title: summary,
                                mode: ChatMode::Collaborative,
                                timestamp: ChatHistory::format_timestamp(),
                            };
                            
                            let session_data = SessionData {
                                session,
                                history: ChatHistory::Collaborative(history),
                                created_at: ChatHistory::format_timestamp(),
                                updated_at: ChatHistory::format_timestamp(),
                            };
                            
                            if let Err(e) = ChatHistory::save_session(&session_data) {
                                eprintln!("Failed to save collaborative session {}: {}", sid, e);
                            }
                        }
                    }
                    Err(e) => {
                        // Handle error
                        if let Some(last_round) = conversation_history_clone.write().last_mut() {
                            last_round.phase1_responses = models
                                .iter()
                                .map(|id| ModelResponse {
                                    model_id: id.clone(),
                                    content: String::new(),
                                    error_message: Some(e.clone()),
                                })
                                .collect();
                        }
                        is_processing_clone.set(false);
                        
                        // Auto-save even on error
                        if let Some(sid) = session_id_for_save {
                            let history_rounds: Vec<crate::utils::CollaborativeRound> = conversation_history_clone.read()
                                .iter()
                                .map(|r| {
                                    let model_responses: Vec<crate::utils::ModelResponse> = r.phase1_responses.iter()
                                        .map(|mr| crate::utils::ModelResponse {
                                            model_id: mr.model_id.clone(),
                                            content: mr.content.clone(),
                                            error_message: mr.error_message.clone(),
                                        })
                                        .collect();
                                    let final_consensus = r.phase3_consensus.as_ref().map(|c| c.content.clone());
                                    crate::utils::CollaborativeRound {
                                        user_message: r.user_question.clone(),
                                        model_responses,
                                        final_consensus,
                                    }
                                })
                                .collect();
                            
                            let history = crate::utils::CollaborativeHistory {
                                rounds: history_rounds,
                                selected_models: selected_models_for_save.clone(),
                                system_prompt: String::new(),
                            };
                            
                            let summary = ChatHistory::generate_chat_summary(&ChatHistory::Collaborative(history.clone()));
                            let session = ChatSession {
                                id: sid.clone(),
                                title: summary,
                                mode: ChatMode::Collaborative,
                                timestamp: ChatHistory::format_timestamp(),
                            };
                            
                            let session_data = SessionData {
                                session,
                                history: ChatHistory::Collaborative(history),
                                created_at: ChatHistory::format_timestamp(),
                                updated_at: ChatHistory::format_timestamp(),
                            };
                            
                            if let Err(e) = ChatHistory::save_session(&session_data) {
                                eprintln!("Failed to save collaborative session {}: {}", sid, e);
                            }
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

            // Model Selection Screen
            if *selection_step.read() == 0 {
                if let Some(_client_arc) = &client {
                    // Header
                    div {
                        class: "p-4 border-b border-[var(--color-base-300)]",
                        h2 {
                            class: "text-lg font-bold text-[var(--color-base-content)] mb-1",
                            "Select Models for Collaboration"
                        }
                        p {
                            class: "text-xs text-[var(--color-base-content)]/70",
                            "Choose 2 or more AI models that will work together to answer your questions."
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
                                        let is_selected = selected_models.read().contains(&model_id);

                                        rsx! {
                                            button {
                                                key: "{model_id}",
                                                onclick: move |_| toggle_model(model_id.clone()),
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

                    // Footer
                    if !loading && error.is_none() {
                        div {
                            class: "p-4 border-t border-[var(--color-base-300)]",
                            div {
                                class: "flex items-center justify-between gap-2",
                                div {
                                    class: "text-sm text-[var(--color-base-content)]/70",
                                    "{selected_models.read().len()} models selected (minimum 2)"
                                }
                                button {
                                    onclick: start_chat,
                                    disabled: selected_models.read().len() < 2,
                                    class: "px-4 py-2 text-sm rounded bg-[var(--color-primary)] text-[var(--color-primary-content)] hover:bg-[var(--color-primary)]/90 disabled:opacity-50 disabled:cursor-not-allowed font-medium transition-all",
                                    "Start Collaborative Chat"
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
                // Chat Interface

                // Prompt Templates Section
                div {
                    class: "p-3 border-b border-[var(--color-base-300)] bg-[var(--color-base-100)]",

                    div {
                        class: "flex items-center justify-between mb-2",
                        h3 {
                            class: "text-sm font-semibold text-[var(--color-base-content)]",
                            "Prompt Templates (Click to customize)"
                        }
                        button {
                            onclick: move |_| {
                                selection_step.set(0);
                                conversation_history.write().clear();
                            },
                            class: "text-xs text-[var(--color-primary)] hover:underline",
                            "Change Models"
                        }
                    }

                    div {
                        class: "grid grid-cols-1 md:grid-cols-3 gap-2",

                        PromptCard {
                            theme,
                            title: "Initial Response".to_string(),
                            phase_number: 1,
                            prompt: prompt_templates.read().initial_response.clone(),
                            on_edit: move |_| open_prompt_editor(PromptType::Initial),
                        }

                        PromptCard {
                            theme,
                            title: "Review Feedback".to_string(),
                            phase_number: 2,
                            prompt: prompt_templates.read().cross_review.clone(),
                            on_edit: move |_| open_prompt_editor(PromptType::Review),
                        }

                        PromptCard {
                            theme,
                            title: "Consensus Synthesis".to_string(),
                            phase_number: 3,
                            prompt: prompt_templates.read().consensus.clone(),
                            on_edit: move |_| open_prompt_editor(PromptType::Consensus),
                        }
                    }
                }

                // Chat area
                div {
                    class: "flex-1 overflow-y-auto p-4",

                    if conversation_history.read().is_empty() {
                        // Empty state
                        div {
                            class: "flex flex-col items-center justify-center h-full",
                            h2 {
                                class: "text-xl font-bold text-[var(--color-base-content)] mb-2",
                                "🤝 Collaborative Mode Ready"
                            }
                            p {
                                class: "text-sm text-[var(--color-base-content)]/70 mb-4 text-center max-w-md",
                                "Selected models will work together through three phases: initial responses, cross-review, and consensus synthesis."
                            }
                            div {
                                class: "text-xs text-[var(--color-base-content)]/60 space-y-1",
                                for (idx, model_id) in selected_models.read().iter().enumerate() {
                                    p { key: "{idx}", "• {model_id}" }
                                }
                            }
                        }
                    } else {
                        // Conversation display
                        div {
                            class: "space-y-8 max-w-6xl mx-auto",

                            for (round_idx, round) in conversation_history.read().iter().enumerate() {
                                div {
                                    key: "{round_idx}",

                                    // User message
                                    div {
                                        class: "flex justify-end mb-4",
                                        div {
                                            class: "max-w-[85%] bg-[var(--color-primary)] text-[var(--color-primary-content)] px-4 py-2 rounded-lg",
                                            FormattedText {
                                                theme,
                                                content: round.user_question.clone(),
                                            }
                                        }
                                    }

                                    // Phase 1: Initial Responses
                                    if !round.phase1_responses.is_empty() {
                                        div {
                                            class: "mb-6",

                                            // Phase header
                                            div {
                                                class: "flex items-center gap-2 mb-3",
                                                span {
                                                    class: "inline-block px-2 py-1 rounded text-xs font-semibold text-white {CollaborativePhase::Initial.badge_color()}",
                                                    "{CollaborativePhase::Initial.name()}"
                                                }
                                            }

                                            // Responses grid
                                            div {
                                                class: "grid grid-cols-1 md:grid-cols-2 gap-3",

                                                for response in round.phase1_responses.iter() {
                                                    div {
                                                        key: "{response.model_id}",
                                                        class: if response.error_message.is_some() {
                                                            "bg-red-500/10 rounded-lg p-4 border-2 border-red-500/50"
                                                        } else {
                                                            "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]"
                                                        },

                                                        div {
                                                            class: "text-sm font-bold text-[var(--color-base-content)] mb-2 truncate",
                                                            "{response.model_id}"
                                                        }

                                                        if let Some(error) = &response.error_message {
                                                            div {
                                                                class: "text-sm text-red-500",
                                                                "Error: {error}"
                                                            }
                                                        } else {
                                                            div {
                                                                class: "text-sm text-[var(--color-base-content)]",
                                                                FormattedText {
                                                                    theme,
                                                                    content: response.content.clone(),
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Phase 2: Cross-Review
                                    if !round.phase2_reviews.is_empty() {
                                        div {
                                            class: "mb-6",

                                            div {
                                                class: "flex items-center gap-2 mb-3",
                                                span {
                                                    class: "inline-block px-2 py-1 rounded text-xs font-semibold text-white {CollaborativePhase::Review.badge_color()}",
                                                    "{CollaborativePhase::Review.name()}"
                                                }
                                            }

                                            div {
                                                class: "grid grid-cols-1 md:grid-cols-2 gap-3",

                                                for review in round.phase2_reviews.iter() {
                                                    div {
                                                        key: "{review.model_id}",
                                                        class: if review.error_message.is_some() {
                                                            "bg-red-500/10 rounded-lg p-4 border-2 border-red-500/50"
                                                        } else {
                                                            "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]"
                                                        },

                                                        div {
                                                            class: "text-sm font-bold text-[var(--color-base-content)] mb-2 truncate",
                                                            "{review.model_id}'s Review"
                                                        }

                                                        if let Some(error) = &review.error_message {
                                                            div {
                                                                class: "text-sm text-red-500",
                                                                "Error: {error}"
                                                            }
                                                        } else {
                                                            div {
                                                                class: "text-sm text-[var(--color-base-content)]",
                                                                FormattedText {
                                                                    theme,
                                                                    content: review.content.clone(),
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Phase 3: Consensus
                                    if let Some(consensus) = &round.phase3_consensus {
                                        div {
                                            class: "mb-6",

                                            div {
                                                class: "flex items-center gap-2 mb-3",
                                                span {
                                                    class: "inline-block px-2 py-1 rounded text-xs font-semibold text-white {CollaborativePhase::Consensus.badge_color()}",
                                                    "{CollaborativePhase::Consensus.name()}"
                                                }
                                            }

                                            div {
                                                class: if consensus.error_message.is_some() {
                                                    "bg-red-500/10 rounded-lg p-4 border-2 border-red-500/50"
                                                } else {
                                                    "bg-green-500/10 rounded-lg p-4 border-2 border-green-500/50"
                                                },

                                                div {
                                                    class: "text-sm font-bold text-[var(--color-base-content)] mb-2",
                                                    "🎯 Collaborative Answer (synthesized by {consensus.model_id})"
                                                }

                                                if let Some(error) = &consensus.error_message {
                                                    div {
                                                        class: "text-sm text-red-500",
                                                        "Error: {error}"
                                                    }
                                                } else {
                                                    div {
                                                        class: "text-sm text-[var(--color-base-content)]",
                                                        FormattedText {
                                                            theme,
                                                            content: consensus.content.clone(),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Streaming indicators
                            if *is_processing.read() && !current_streaming_responses.read().is_empty() {
                                div {
                                    class: "mb-6",

                                    div {
                                        class: "flex items-center gap-2 mb-3",
                                        span {
                                            class: "inline-block px-2 py-1 rounded text-xs font-semibold text-white {current_phase.read().badge_color()}",
                                            "{current_phase.read().name()}"
                                        }
                                        span {
                                            class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
                                        }
                                    }

                                    div {
                                        class: if *current_phase.read() == CollaborativePhase::Consensus {
                                            "bg-green-500/10 rounded-lg p-4 border-2 border-green-500/50"
                                        } else {
                                            "grid grid-cols-1 md:grid-cols-2 gap-3"
                                        },

                                        for (model_id, content) in current_streaming_responses.read().iter() {
                                            div {
                                                key: "{model_id}",
                                                class: if *current_phase.read() != CollaborativePhase::Consensus {
                                                    "bg-[var(--color-base-200)] rounded-lg p-4 border border-[var(--color-base-300)]"
                                                } else {
                                                    ""
                                                },

                                                div {
                                                    class: "text-sm font-bold text-[var(--color-base-content)] mb-2 flex items-center gap-2",
                                                    if model_id == "consensus" {
                                                        span { "🎯 Synthesizing Collaborative Answer..." }
                                                    } else {
                                                        span { "{model_id}" }
                                                    }
                                                    span {
                                                        class: "inline-block w-2 h-2 bg-[var(--color-primary)] rounded-full animate-pulse"
                                                    }
                                                }

                                                div {
                                                    class: "text-sm text-[var(--color-base-content)] min-h-[3rem]",
                                                    FormattedText {
                                                        theme,
                                                        content: content.clone(),
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

            // Prompt Editor Modal
            PromptEditorModal {
                theme,
                open: prompt_editor_open,
                prompt_type: *editing_prompt_type.read(),
                current_prompt: temp_prompt(),
                default_prompt: PromptTemplates::default().get(*editing_prompt_type.read()),
                on_save: save_prompt,
            }
        }
    }
}
