#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppView {
    NewChat,
    ChatMode(ChatMode),
    Settings,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChatMode {
    Standard,
    PvP,
    Collaborative,
    Competitive,
    LLMChoice,
}

impl ChatMode {
    pub fn name(&self) -> &'static str {
        match self {
            ChatMode::Standard => "Standard",
            ChatMode::PvP => "PvP",
            ChatMode::Collaborative => "Collaborative",
            ChatMode::Competitive => "Competitive",
            ChatMode::LLMChoice => "LLM's Choice",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ChatMode::Standard => "Single LLM chat",
            ChatMode::PvP => "2 bots compete, 1 moderator judges",
            ChatMode::Collaborative => "Multiple bots jointly agree on best solution",
            ChatMode::Competitive => "All bots vote for the best (can't vote for their own)",
            ChatMode::LLMChoice => "LLMs decide to collaborate or compete",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub id: usize,
    pub content: String,
    pub sender: String,
    pub is_user: bool,
    pub timestamp: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArenaMessage {
    pub id: usize,
    pub content: String,
    pub bot_name: String,
    pub timestamp: String,
    pub vote_count: Option<usize>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChatSession {
    pub id: usize,
    pub title: String,
    pub mode: ChatMode,
    pub timestamp: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InputSettings {
    pub ctrl_enter_submit: bool, // true = Ctrl+Enter to submit, false = Enter to submit
}
