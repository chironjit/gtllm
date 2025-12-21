use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::utils::ChatSession;

/// Represents the full conversation history for a chat session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode")]
pub enum ChatHistory {
    #[serde(rename = "standard")]
    Standard(StandardHistory),
    #[serde(rename = "pvp")]
    PvP(PvPHistory),
    #[serde(rename = "collaborative")]
    Collaborative(CollaborativeHistory),
    #[serde(rename = "competitive")]
    Competitive(CompetitiveHistory),
    #[serde(rename = "llm_choice")]
    LLMChoice(LLMChoiceHistory),
}

/// Standard mode history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandardHistory {
    pub user_messages: Vec<String>,
    pub model_responses: Vec<Vec<ModelResponse>>,
    pub selected_models: Vec<String>,
    pub system_prompt: String,
    pub conversation_history: ConversationHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelResponse {
    pub model_id: String,
    pub content: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationHistory {
    pub single_model: Vec<(String, String)>,
    pub multi_model: HashMap<String, Vec<(String, String)>>,
}

/// PvP mode history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PvPHistory {
    pub rounds: Vec<ConversationRound>,
    pub bot_models: Vec<String>,
    pub moderator_model: Option<String>,
    pub system_prompts: SystemPrompts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationRound {
    pub user_message: String,
    pub bot1_response: BotResponse,
    pub bot2_response: BotResponse,
    pub moderator_judgment: Option<ModeratorResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BotResponse {
    pub model_id: String,
    pub content: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModeratorResponse {
    pub content: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemPrompts {
    pub bot: String,
    pub moderator: String,
}

/// Collaborative mode history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollaborativeHistory {
    pub rounds: Vec<CollaborativeRound>,
    pub selected_models: Vec<String>,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollaborativeRound {
    pub user_message: String,
    pub model_responses: Vec<ModelResponse>,
    pub final_consensus: Option<String>,
}

/// Competitive mode history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompetitiveHistory {
    pub rounds: Vec<CompetitiveRound>,
    pub selected_models: Vec<String>,
    pub prompt_templates: PromptTemplates,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompetitiveRound {
    pub user_question: String,
    pub phase1_proposals: Vec<ModelProposal>,
    pub phase2_votes: Vec<ModelVote>,
    pub vote_tallies: Vec<VoteTally>,
    pub winners: Vec<String>,
    pub current_phase: String, // "proposal", "voting", "tallying", "complete"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelProposal {
    pub model_id: String,
    pub content: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelVote {
    pub voter_id: String,
    pub voted_for: Option<String>,
    pub raw_response: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoteTally {
    pub model_id: String,
    pub vote_count: usize,
    pub voters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptTemplates {
    pub proposal: String,
    pub voting: String,
}

/// LLM Choice mode history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMChoiceHistory {
    pub rounds: Vec<LLMChoiceRound>,
    pub selected_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMChoiceRound {
    pub user_message: String,
    pub decision: String, // "collaborate" or "compete"
    pub content: Option<String>, // The actual response content
}

/// Complete session data with metadata and history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    pub session: ChatSession,
    pub history: ChatHistory,
    pub created_at: String,
    pub updated_at: String,
}

impl ChatHistory {
    /// Get the chats directory path
    pub fn chats_dir() -> Result<PathBuf, String> {
        let base_dir = crate::utils::Settings::settings_dir()?;
        Ok(base_dir.join("chats"))
    }

    /// Get the path to a specific session file
    pub fn session_path(session_id: usize) -> Result<PathBuf, String> {
        Ok(Self::chats_dir()?.join(format!("session_{}.json", session_id)))
    }

    /// List all saved sessions
    pub fn list_sessions() -> Result<Vec<SessionData>, String> {
        let chats_dir = Self::chats_dir()?;
        
        if !chats_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let entries = fs::read_dir(&chats_dir)
            .map_err(|e| format!("Failed to read chats directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if file_name.starts_with("session_") {
                        match Self::load_session_file(&path) {
                            Ok(session_data) => sessions.push(session_data),
                            Err(e) => eprintln!("Failed to load session from {:?}: {}", path, e),
                        }
                    }
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(sessions)
    }

    /// Load a session from disk
    pub fn load_session(session_id: usize) -> Result<SessionData, String> {
        let path = Self::session_path(session_id)?;
        Self::load_session_file(&path)
    }

    /// Load a session from a specific file path
    fn load_session_file(path: &PathBuf) -> Result<SessionData, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read session file: {}", e))?;
        
        let session_data: SessionData = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse session file: {}", e))?;
        
        Ok(session_data)
    }

    /// Save a session to disk
    pub fn save_session(session_data: &SessionData) -> Result<(), String> {
        let chats_dir = Self::chats_dir()?;
        
        // Create directory if it doesn't exist
        if !chats_dir.exists() {
            fs::create_dir_all(&chats_dir)
                .map_err(|e| format!("Failed to create chats directory: {}", e))?;
        }

        let path = Self::session_path(session_data.session.id)?;
        let contents = serde_json::to_string_pretty(session_data)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        fs::write(&path, contents)
            .map_err(|e| format!("Failed to write session file: {}", e))?;

        // Set proper permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?
                .permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&path, perms)
                .map_err(|e| format!("Failed to set file permissions: {}", e))?;
        }

        Ok(())
    }

    /// Delete a session from disk
    pub fn delete_session(session_id: usize) -> Result<(), String> {
        let path = Self::session_path(session_id)?;
        
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }

        Ok(())
    }

    /// Generate a unique session ID (highest existing ID + 1)
    pub fn generate_session_id() -> Result<usize, String> {
        let sessions = Self::list_sessions()?;
        let max_id = sessions.iter()
            .map(|s| s.session.id)
            .max()
            .unwrap_or(0);
        Ok(max_id + 1)
    }

    /// Format timestamp for display
    pub fn format_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Format as simple timestamp (seconds since epoch)
        // For display, we'll format it when needed
        now.to_string()
    }
    
    /// Format timestamp for human-readable display
    pub fn format_timestamp_display(timestamp: &str) -> String {
        // Try to parse as unix timestamp
        if let Ok(secs) = timestamp.parse::<u64>() {
            use std::time::{SystemTime, UNIX_EPOCH, Duration};
            let datetime = UNIX_EPOCH + Duration::from_secs(secs);
            if let Ok(datetime) = datetime.duration_since(UNIX_EPOCH) {
                // Simple format: just show relative time for now
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let diff = now.saturating_sub(secs);
                
                if diff < 60 {
                    "Just now".to_string()
                } else if diff < 3600 {
                    format!("{} minutes ago", diff / 60)
                } else if diff < 86400 {
                    format!("{} hours ago", diff / 3600)
                } else if diff < 604800 {
                    format!("{} days ago", diff / 86400)
                } else {
                    // For older dates, show the date
                    // Simple format: YYYY-MM-DD
                    let days = diff / 86400;
                    if days < 365 {
                        format!("{} days ago", days)
                    } else {
                        format!("{} years ago", days / 365)
                    }
                }
            } else {
                timestamp.to_string()
            }
        } else {
            timestamp.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        let timestamp = ChatHistory::format_timestamp();
        assert!(!timestamp.is_empty());
    }
    
    #[test]
    fn test_format_timestamp_display() {
        let now = ChatHistory::format_timestamp();
        let display = ChatHistory::format_timestamp_display(&now);
        assert!(!display.is_empty());
    }
}

