use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
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
    pub fn session_path(session_id: &str) -> Result<PathBuf, String> {
        Ok(Self::chats_dir()?.join(format!("{}.json", session_id)))
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
                    // Only accept UUID format
                    if Uuid::parse_str(file_name).is_ok() {
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
    pub fn load_session(session_id: &str) -> Result<SessionData, String> {
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

    /// Save a session to disk using atomic write (temp file + rename)
    pub fn save_session(session_data: &SessionData) -> Result<(), String> {
        let chats_dir = Self::chats_dir()?;
        
        // Create directory if it doesn't exist
        if !chats_dir.exists() {
            fs::create_dir_all(&chats_dir)
                .map_err(|e| format!("Failed to create chats directory: {}", e))?;
        }

        let path = Self::session_path(&session_data.session.id)?;
        let temp_path = path.with_extension("tmp");
        
        let contents = serde_json::to_string_pretty(session_data)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        // Write to temporary file first
        fs::write(&temp_path, contents)
            .map_err(|e| format!("Failed to write temporary session file: {}", e))?;

        // Set proper permissions on Unix-like systems before rename
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path)
                .map_err(|e| format!("Failed to get temp file metadata: {}", e))?
                .permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&temp_path, perms)
                .map_err(|e| format!("Failed to set temp file permissions: {}", e))?;
        }

        // Atomic rename (replaces existing file safely)
        fs::rename(&temp_path, &path)
            .map_err(|e| format!("Failed to rename temporary file to session file: {}", e))?;

        Ok(())
    }

    /// Delete a session from disk
    pub fn delete_session(session_id: &str) -> Result<(), String> {
        let path = Self::session_path(session_id)?;
        
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }

        Ok(())
    }

    /// Generate a unique session ID using UUID v4
    pub fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Generate a chat summary from the first user message
    pub fn generate_chat_summary(history: &ChatHistory) -> String {
        let first_message = match history {
            ChatHistory::Standard(h) => h.user_messages.first(),
            ChatHistory::PvP(h) => h.rounds.first().map(|r| &r.user_message),
            ChatHistory::Collaborative(h) => h.rounds.first().map(|r| &r.user_message),
            ChatHistory::Competitive(h) => h.rounds.first().map(|r| &r.user_question),
            ChatHistory::LLMChoice(h) => h.rounds.first().map(|r| &r.user_message),
        };
        
        if let Some(msg) = first_message {
            // Take first 60 characters, truncate at word boundary if possible
            let trimmed = msg.trim();
            if trimmed.len() <= 60 {
                trimmed.to_string()
            } else {
                // Try to find a word boundary
                let truncated = &trimmed[..60];
                if let Some(last_space) = truncated.rfind(' ') {
                    format!("{}...", &trimmed[..last_space])
                } else {
                    format!("{}...", truncated)
                }
            }
        } else {
            "New Chat".to_string()
        }
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
    
    /// Format timestamp for human-readable display (relative time)
    pub fn format_timestamp_display(timestamp: &str) -> String {
        // Try to parse as unix timestamp
        if let Ok(secs) = timestamp.parse::<u64>() {
            use std::time::{SystemTime, UNIX_EPOCH, Duration};
            let datetime = UNIX_EPOCH + Duration::from_secs(secs);
            if let Ok(_datetime) = datetime.duration_since(UNIX_EPOCH) {
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
    
    /// Format timestamp as "Sat, 12th Dec" format
    pub fn format_timestamp_date(timestamp: &str) -> String {
        // Try to parse as unix timestamp
        if let Ok(secs) = timestamp.parse::<u64>() {
            use std::time::{SystemTime, UNIX_EPOCH, Duration};
            let datetime = UNIX_EPOCH + Duration::from_secs(secs);
            
            // Convert to chrono-compatible format
            // We'll use a simple manual approach to avoid adding chrono dependency
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Calculate days since epoch
            let days_since_epoch = secs / 86400;
            let now_days = now / 86400;
            let days_diff = now_days.saturating_sub(days_since_epoch);
            
            // For dates within the last week, show relative
            if days_diff == 0 {
                "Today".to_string()
            } else if days_diff == 1 {
                "Yesterday".to_string()
            } else if days_diff < 7 {
                format!("{} days ago", days_diff)
            } else {
                // For older dates, format as "Day, Nth Month"
                // This is a simplified version - for full date formatting, consider adding chrono
                let weekday_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", 
                                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
                
                // Calculate day of week (simplified - Jan 1, 1970 was a Thursday)
                let day_of_week = (days_since_epoch + 4) % 7;
                let weekday = weekday_names[day_of_week as usize];
                
                // Approximate month and day (simplified calculation)
                // This is a rough approximation - for accuracy, use chrono
                let year = 1970 + (days_since_epoch / 365);
                let day_of_year = days_since_epoch % 365;
                let month_idx = (day_of_year / 30).min(11);
                let month = month_names[month_idx as usize];
                let day = (day_of_year % 30) + 1;
                
                // Add ordinal suffix
                let suffix = match day {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };
                
                format!("{}, {}{} {}", weekday, day, suffix, month)
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

