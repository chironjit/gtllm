use dioxus::prelude::*;

mod components;
mod utils;

use components::{
    Choice, Collaborative, Competitive, Header, NewChat, PvP, Settings, Sidebar, Standard,
};
use utils::{AppView, ArenaMessage, ChatMode, ChatSession, InputSettings, Message, Theme};

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Theme state (default to Dracula)
    let mut theme = use_signal(|| Theme::Dracula);

    // Current view state
    let mut current_view = use_signal(|| AppView::NewChat);

    // Messages for standard chat
    let mut messages = use_signal(|| Vec::<Message>::new());

    // Messages for arena modes
    let mut arena_messages = use_signal(|| Vec::<ArenaMessage>::new());

    // Chat sessions
    let mut sessions = use_signal(|| Vec::<ChatSession>::new());
    let mut current_session = use_signal(|| None::<usize>);

    // Input settings
    let mut input_settings = use_signal(|| InputSettings {
        ctrl_enter_submit: true,
    });

    // Message counter for IDs
    let mut message_counter = use_signal(|| 0);

    // Handler for toggling dark/light mode
    let toggle_mode = move |_| {
        let current_theme = *theme.read();
        if current_theme.is_dark() {
            // Switch to first light theme
            theme.set(Theme::Winter);
        } else {
            // Switch to first dark theme
            theme.set(Theme::Dracula);
        }
    };

    // Handler for changing theme within mode
    let change_theme = move |new_theme: Theme| {
        theme.set(new_theme);
    };

    // Handler for creating new chat
    let new_chat = move |_| {
        current_view.set(AppView::NewChat);
        messages.write().clear();
        arena_messages.write().clear();
        current_session.set(None);
    };

    // Handler for selecting a session
    let select_session = move |session_id: usize| {
        if let Some(session) = sessions.read().iter().find(|s| s.id == session_id) {
            current_session.set(Some(session_id));
            current_view.set(AppView::ChatMode(session.mode));
            // In a real app, you would load the session's messages here
            messages.write().clear();
            arena_messages.write().clear();
        }
    };

    // Handler for mode selection from NewChat view
    let select_mode = move |mode: ChatMode| {
        let session_id = sessions.read().len();
        let new_session = ChatSession {
            id: session_id,
            title: format!("{} Chat {}", mode.name(), session_id + 1),
            mode,
            timestamp: "Just now".to_string(),
        };
        sessions.write().push(new_session);
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
        if let Some(session_id) = *current_session.read() {
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
                    }

                    // Main content based on current view
                    main {
                        class: "grow min-h-0",

                        match *current_view.read() {
                            AppView::NewChat => rsx! {
                                NewChat {
                                    theme,
                                    on_mode_select: select_mode,
                                }
                            },
                            AppView::ChatMode(mode) => {
                                match mode {
                                    ChatMode::Standard => rsx! {
                                        Standard {
                                            theme,
                                            messages,
                                            input_settings,
                                            on_send: move |text| send_message(text),
                                        }
                                    },
                                    ChatMode::PvP => rsx! {
                                        PvP {
                                            theme,
                                            messages,
                                            input_settings,
                                            on_send: move |text| send_arena_message(text, ChatMode::PvP),
                                        }
                                    },
                                    ChatMode::Collaborative => rsx! {
                                        Collaborative {
                                            theme,
                                            messages,
                                            input_settings,
                                            on_send: move |text| send_arena_message(text, ChatMode::Collaborative),
                                        }
                                    },
                                    ChatMode::Competitive => rsx! {
                                        Competitive {
                                            theme,
                                            messages,
                                            input_settings,
                                            on_send: move |text| send_arena_message(text, ChatMode::Competitive),
                                        }
                                    },
                                    ChatMode::LLMChoice => rsx! {
                                        Choice {
                                            theme,
                                            messages,
                                            input_settings,
                                            on_send: move |text| send_arena_message(text, ChatMode::LLMChoice),
                                        }
                                    },
                                }
                            },
                            AppView::Settings => rsx! {
                                Settings {
                                    theme,
                                    input_settings,
                                    on_settings_change: change_input_settings,
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
