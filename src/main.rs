use dioxus::prelude::*;

mod components;
mod utils;

use components::{
    Choice, Collaborative, Competitive, Header, NewChat, PvP, Settings as SettingsView, Sidebar, Standard,
};
use utils::{AppView, ArenaMessage, ChatHistory, ChatMode, ChatSession, InputSettings, Message, OpenRouterClient, Settings, Theme};
use std::sync::Arc;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Load settings from disk on startup
    let mut app_settings = use_signal(|| {
        Settings::load().unwrap_or_else(|e| {
            eprintln!("Failed to load settings: {}", e);
            Settings::default()
        })
    });

    // Create OpenRouter client if API key exists
    let mut openrouter_client = use_signal(|| {
        app_settings.read().get_api_key().map(|key| {
            Arc::new(OpenRouterClient::new(key.to_string()))
        })
    });

    // Theme state - load from settings
    let mut theme = use_signal(|| {
        let settings = app_settings.read();
        // Parse theme from settings, default to Dracula if parsing fails
        Theme::from_str(&settings.theme).unwrap_or(Theme::Dracula)
    });

    // Sidebar state (default to open)
    let sidebar_collapsed = use_signal(|| false);

    // Current view state
    let mut current_view = use_signal(|| AppView::NewChat);

    // Messages for standard chat
    let mut messages = use_signal(|| Vec::<Message>::new());

    // Messages for arena modes
    let mut arena_messages = use_signal(|| Vec::<ArenaMessage>::new());

    // Chat sessions - load from disk on startup with summaries
    let mut sessions = use_signal(|| {
        ChatHistory::list_sessions()
            .unwrap_or_else(|e| {
                eprintln!("Failed to load sessions: {}", e);
                Vec::new()
            })
            .into_iter()
            .map(|mut sd| {
                // Generate summary from history if title is generic
                if sd.session.title.starts_with("Standard Chat") 
                    || sd.session.title.starts_with("PvP Chat")
                    || sd.session.title.starts_with("Competitive Chat") {
                    let summary = ChatHistory::generate_chat_summary(&sd.history);
                    sd.session.title = summary;
                }
                sd.session
            })
            .collect::<Vec<_>>()
    });
    let mut current_session = use_signal(|| None::<String>);

    // Input settings
    let mut input_settings = use_signal(|| InputSettings {
        ctrl_enter_submit: true,
    });

    // Message counter for IDs
    let mut message_counter = use_signal(|| 0);

    // Handler for toggling dark/light mode
    let toggle_mode = move |_| {
        let current_theme = *theme.read();
        let new_theme = if current_theme.is_dark() {
            // Switch to first light theme
            Theme::Winter
        } else {
            // Switch to first dark theme
            Theme::Dracula
        };
        theme.set(new_theme);

        // Save theme to settings
        let mut settings = app_settings.write();
        settings.theme = new_theme.to_string_id().to_string();
        settings.theme_mode = if new_theme.is_dark() {
            utils::ThemeMode::Dark
        } else {
            utils::ThemeMode::Light
        };
        let _ = settings.save();
    };

    // Handler for changing theme within mode
    let change_theme = move |new_theme: Theme| {
        theme.set(new_theme);

        // Save theme to settings
        let mut settings = app_settings.write();
        settings.theme = new_theme.to_string_id().to_string();
        settings.theme_mode = if new_theme.is_dark() {
            utils::ThemeMode::Dark
        } else {
            utils::ThemeMode::Light
        };
        let _ = settings.save();
    };

    // Handler for saving API key
    let save_api_key = move |api_key: String| {
        let mut settings = app_settings.write();
        settings.set_api_key(api_key.clone());
        if let Err(e) = settings.save() {
            eprintln!("Failed to save API key: {}", e);
            return;
        }

        // Recreate OpenRouter client with new API key
        openrouter_client.set(Some(Arc::new(OpenRouterClient::new(api_key))));
    };

    // Handler for creating new chat
    let new_chat = move |_| {
        current_view.set(AppView::NewChat);
        messages.write().clear();
        arena_messages.write().clear();
        current_session.set(None);
    };

    // Handler for selecting a session
    let select_session = move |session_id: String| {
        if let Some(session) = sessions.read().iter().find(|s| s.id == session_id) {
            current_session.set(Some(session_id.clone()));
            current_view.set(AppView::ChatMode(session.mode));
            // Messages will be loaded by the mode component itself
            messages.write().clear();
            arena_messages.write().clear();
        }
    };

    // Handler for mode selection from NewChat view
    let select_mode = move |mode: ChatMode| {
        let session_id = ChatHistory::generate_session_id();
        let new_session = ChatSession {
            id: session_id.clone(),
            title: format!("{} Chat", mode.name()),
            mode,
            timestamp: ChatHistory::format_timestamp_display(&ChatHistory::format_timestamp()),
        };
        sessions.write().push(new_session.clone());
        current_session.set(Some(session_id));
        current_view.set(AppView::ChatMode(mode));
        messages.write().clear();
        arena_messages.write().clear();
    };

    // Handler for opening settings
    let open_settings = move |_| {
        current_view.set(AppView::Settings);
    };

    // Handler for closing settings
    let close_settings = move |_| {
        if let Some(session_id) = current_session.read().clone() {
            if let Some(session) = sessions.read().iter().find(|s| s.id == session_id) {
                current_view.set(AppView::ChatMode(session.mode));
            } else {
                current_view.set(AppView::NewChat);
            }
        } else {
            current_view.set(AppView::NewChat);
        }
    };

    // Handler for input settings change
    let change_input_settings = move |new_settings: InputSettings| {
        input_settings.set(new_settings);
    };

    // Handler for sending messages in standard mode
    let mut send_message = move |text: String| {
        let id = *message_counter.read();
        message_counter.set(id + 1);

        // Add user message
        messages.write().push(Message {
            id,
            content: text.clone(),
            sender: "You".to_string(),
            is_user: true,
            timestamp: "Just now".to_string(),
        });

        // Simulate bot response
        let bot_id = id + 1;
        message_counter.set(bot_id + 1);
        messages.write().push(Message {
            id: bot_id,
            content: format!("This is a simulated response to: '{}'", text),
            sender: "Assistant".to_string(),
            is_user: false,
            timestamp: "Just now".to_string(),
        });
    };

    // Handler for sending messages in arena mode
    let mut send_arena_message = move |text: String, mode: ChatMode| {
        let base_id = *message_counter.read();

        match mode {
            ChatMode::PvP => {
                // Add user message first
                messages.write().push(Message {
                    id: base_id,
                    content: text.clone(),
                    sender: "You".to_string(),
                    is_user: true,
                    timestamp: "Just now".to_string(),
                });

                // Add responses from Bot 1, Bot 2, and Moderator
                messages.write().push(Message {
                    id: base_id + 1,
                    content: format!("Bot 1's response to: '{}'", text),
                    sender: "Bot 1".to_string(),
                    is_user: false,
                    timestamp: "Just now".to_string(),
                });

                messages.write().push(Message {
                    id: base_id + 2,
                    content: format!("Bot 2's response to: '{}'", text),
                    sender: "Bot 2".to_string(),
                    is_user: false,
                    timestamp: "Just now".to_string(),
                });

                messages.write().push(Message {
                    id: base_id + 3,
                    content: "As moderator, I find Bot 1's response more comprehensive."
                        .to_string(),
                    sender: "Moderator".to_string(),
                    is_user: false,
                    timestamp: "Just now".to_string(),
                });

                message_counter.set(base_id + 4);
            }
            ChatMode::Collaborative => {
                // Add user message
                messages.write().push(Message {
                    id: base_id,
                    content: text.clone(),
                    sender: "You".to_string(),
                    is_user: true,
                    timestamp: "Just now".to_string(),
                });

                // Add collaborative responses
                for i in 0..3 {
                    messages.write().push(Message {
                        id: base_id + i + 1,
                        content: format!("Bot {} contributes to the solution...", i + 1),
                        sender: format!("Bot {}", i + 1),
                        is_user: false,
                        timestamp: "Just now".to_string(),
                    });
                }
                message_counter.set(base_id + 4);
            }
            ChatMode::Competitive => {
                // Add user message
                messages.write().push(Message {
                    id: base_id,
                    content: text.clone(),
                    sender: "You".to_string(),
                    is_user: true,
                    timestamp: "Just now".to_string(),
                });

                // Add competitive responses
                for i in 0..3 {
                    messages.write().push(Message {
                        id: base_id + i + 1,
                        content: format!(
                            "Bot {}'s competitive solution to: '{}' (votes: {})",
                            i + 1,
                            text,
                            i % 3 + 1
                        ),
                        sender: format!("Bot {}", i + 1),
                        is_user: false,
                        timestamp: "Just now".to_string(),
                    });
                }
                message_counter.set(base_id + 4);
            }
            ChatMode::LLMChoice => {
                // Add user message
                messages.write().push(Message {
                    id: base_id,
                    content: text,
                    sender: "You".to_string(),
                    is_user: true,
                    timestamp: "Just now".to_string(),
                });

                messages.write().push(Message {
                    id: base_id + 1,
                    content: "LLMs have chosen to COLLABORATE on this task.".to_string(),
                    sender: "System".to_string(),
                    is_user: false,
                    timestamp: "Just now".to_string(),
                });
                message_counter.set(base_id + 2);
            }
            _ => {}
        }
    };

    let theme_val = *theme.read();
    let data_theme = theme_val.data_theme();

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div {
            "data-theme": "{data_theme}",
            class: "font-inter antialiased bg-[var(--color-base-100)] text-[var(--color-base-content)]",

            div {
                class: "flex h-screen overflow-hidden",

                // Sidebar
                Sidebar {
                    theme,
                    sessions,
                    current_session,
                    collapsed: sidebar_collapsed,
                    on_new_chat: new_chat,
                    on_select_session: select_session,
                }

                // Main content area
                div {
                    class: "relative flex flex-col flex-1 overflow-hidden min-h-0",

                    // Header
                    Header {
                        theme,
                        on_toggle_mode: toggle_mode,
                        on_theme_change: change_theme,
                        on_settings: open_settings,
                        client: openrouter_client,
                    }

                    // Main content based on current view
                    main {
                        class: "grow min-h-0",

                        match *current_view.read() {
                            AppView::NewChat => rsx! {
                                NewChat {
                                    theme,
                                    app_settings,
                                    on_mode_select: select_mode,
                                    on_open_settings: open_settings,
                                }
                            },
                            AppView::ChatMode(mode) => {
                                let session_id = current_session.read().clone();
                                match mode {
                                    ChatMode::Standard => rsx! {
                                        Standard {
                                            theme,
                                            client: openrouter_client.read().clone(),
                                            input_settings,
                                            session_id,
                                        }
                                    },
                                    ChatMode::PvP => rsx! {
                                        PvP {
                                            theme,
                                            client: openrouter_client.read().clone(),
                                            input_settings,
                                            session_id,
                                        }
                                    },
                                    ChatMode::Collaborative => rsx! {
                                        Collaborative {
                                            theme,
                                            client: openrouter_client.read().clone(),
                                            input_settings,
                                            session_id,
                                        }
                                    },
                                    ChatMode::Competitive => rsx! {
                                        Competitive {
                                            theme,
                                            client: openrouter_client.read().clone(),
                                            input_settings,
                                            session_id,
                                        }
                                    },
                                    ChatMode::LLMChoice => rsx! {
                                        Choice {
                                            theme,
                                            client: openrouter_client.read().clone(),
                                            input_settings,
                                            session_id,
                                        }
                                    },
                                }
                            },
                            AppView::Settings => rsx! {
                                SettingsView {
                                    theme,
                                    app_settings,
                                    input_settings,
                                    on_settings_change: change_input_settings,
                                    on_api_key_save: save_api_key,
                                    on_close: close_settings,
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
