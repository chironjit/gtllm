use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::utils::{ChatSession, ChatMode};

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

    /// Sanitize a title for use in filename
    /// Removes invalid filesystem characters and limits length to 100 chars
    fn sanitize_filename(title: &str) -> String {
        let mut sanitized = String::with_capacity(title.len().min(100));
        
        for ch in title.chars() {
            // Invalid filesystem characters: / \ : * ? " < > |
            if matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                sanitized.push('_');
            } else if ch.is_whitespace() {
                sanitized.push('_');
            } else {
                sanitized.push(ch);
            }
            
            // Limit to 100 characters
            if sanitized.len() >= 100 {
                break;
            }
        }
        
        // Remove trailing underscores
        sanitized = sanitized.trim_end_matches('_').to_string();
        
        // If empty after sanitization, use default
        if sanitized.is_empty() {
            "New_Chat".to_string()
        } else {
            sanitized
        }
    }

    /// Generate filename from session metadata
    /// Format: <mode>_<timestamp>_<sanitized-title>.json
    fn generate_filename(mode: ChatMode, timestamp: &str, title: &str) -> String {
        let mode_str = match mode {
            ChatMode::Standard => "standard",
            ChatMode::PvP => "pvp",
            ChatMode::Collaborative => "collaborative",
            ChatMode::Competitive => "competitive",
            ChatMode::LLMChoice => "llm_choice",
        };
        let sanitized_title = Self::sanitize_filename(title);
        format!("{}_{}_{}.json", mode_str, timestamp, sanitized_title)
    }

    /// Parse filename to extract metadata
    /// Returns (mode, timestamp, title) if successful
    fn parse_filename(filename: &str) -> Option<(ChatMode, String, String)> {
        // Remove .json extension if present
        let stem = filename.strip_suffix(".json").unwrap_or(filename);
        
        // Split by underscore - format is: mode_timestamp_title
        // But title may contain underscores, so we need to split carefully
        let parts: Vec<&str> = stem.split('_').collect();
        
        if parts.len() < 3 {
            return None;
        }
        
        // Check for "llm_choice" mode first (two-word mode)
        let (mode, timestamp_idx) = if parts.len() >= 3 && parts[0] == "llm" && parts[1] == "choice" {
            (ChatMode::LLMChoice, 2)
        } else if parts.len() >= 3 {
            // Single-word modes
            let mode = match parts[0] {
                "standard" => ChatMode::Standard,
                "pvp" => ChatMode::PvP,
                "collaborative" => ChatMode::Collaborative,
                "competitive" => ChatMode::Competitive,
                _ => return None,
            };
            (mode, 1)
        } else {
            return None;
        };
        
        // Timestamp is at timestamp_idx
        let timestamp = parts[timestamp_idx].to_string();
        
        // Rest is title (may contain underscores)
        let title = parts[timestamp_idx + 1..].join("_");
        
        Some((mode, timestamp, title))
    }

    /// Get the path to a specific session file
    /// session_id is now the full filename (without .json extension)
    pub fn session_path(session_id: &str) -> Result<PathBuf, String> {
        Ok(Self::chats_dir()?.join(format!("{}.json", session_id)))
    }

    /// List all saved sessions by parsing filenames (fast, no file I/O)
    /// Returns only ChatSession objects, not full SessionData
    pub fn list_sessions() -> Result<Vec<ChatSession>, String> {
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
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    // Parse filename to extract metadata
                    if let Some((mode, timestamp, title)) = Self::parse_filename(file_name) {
                        // Create session from filename metadata (no file I/O!)
                        let session_id = file_name.strip_suffix(".json").unwrap_or(file_name).to_string();
                        let session = ChatSession {
                            id: session_id,
                            title,
                            mode,
                            timestamp,
                        };
                        sessions.push(session);
                    }
                }
            }
        }

        // Sort by timestamp descending (most recent first)
        sessions.sort_by(|a, b| {
            // Parse timestamps as numbers for proper sorting
            let a_ts = a.timestamp.parse::<u64>().unwrap_or(0);
            let b_ts = b.timestamp.parse::<u64>().unwrap_or(0);
            b_ts.cmp(&a_ts)
        });
        
        Ok(sessions)
    }

    /// Load a session from disk
    /// session_id is the filename without .json extension
    pub fn load_session(session_id: &str) -> Result<SessionData, String> {
        let path = Self::session_path(session_id)?;
        let mut session_data = Self::load_session_file(&path)?;
        
        // Update session.id to match filename (in case it was changed)
        session_data.session.id = session_id.to_string();
        
        Ok(session_data)
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
    /// The filename is generated from mode, timestamp, and title
    /// If the title changed, the file will be renamed
    /// Returns the new session ID (filename without .json) if it changed
    pub fn save_session(session_data: &SessionData) -> Result<Option<String>, String> {
        let chats_dir = Self::chats_dir()?;
        
        // Create directory if it doesn't exist
        if !chats_dir.exists() {
            fs::create_dir_all(&chats_dir)
                .map_err(|e| format!("Failed to create chats directory: {}", e))?;
        }

        // Generate new filename from session data
        let new_filename = Self::generate_filename(
            session_data.session.mode,
            &session_data.updated_at,
            &session_data.session.title,
        );
        let new_session_id = new_filename.strip_suffix(".json").unwrap_or(&new_filename).to_string();
        let new_path = chats_dir.join(&new_filename);
        let temp_path = new_path.with_extension("tmp");
        
        // Check if we need to rename (if session.id doesn't match new filename)
        let old_filename = format!("{}.json", session_data.session.id);
        let needs_rename = old_filename != new_filename;
        
        if needs_rename {
            // Title or timestamp changed - need to rename
            let old_path = chats_dir.join(&old_filename);
            if old_path.exists() && old_path != new_path {
                // Delete old file (we'll create new one with correct name)
                fs::remove_file(&old_path)
                    .map_err(|e| format!("Failed to remove old session file: {}", e))?;
            }
        }
        
        // Create updated session data with correct session.id
        let mut updated_session_data = session_data.clone();
        updated_session_data.session.id = new_session_id.clone();
        
        let contents = serde_json::to_string_pretty(&updated_session_data)
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
        fs::rename(&temp_path, &new_path)
            .map_err(|e| format!("Failed to rename temporary file to session file: {}", e))?;

        // Return new session ID if it changed
        if needs_rename {
            Ok(Some(new_session_id))
        } else {
            Ok(None)
        }
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

    /// Generate a session ID (filename) from mode, timestamp, and title
    /// Returns the filename without .json extension
    pub fn generate_session_id(mode: ChatMode, timestamp: &str, title: &str) -> String {
        let filename = Self::generate_filename(mode, timestamp, title);
        filename.strip_suffix(".json").unwrap_or(&filename).to_string()
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
    
    /// Check if a year is a leap year
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Get the number of days in a month, accounting for leap years
    fn days_in_month(month: usize, is_leap: bool) -> u32 {
        match month {
            1 => if is_leap { 29 } else { 28 }, // February
            0 | 2 | 4 | 6 | 7 | 9 | 11 => 31,    // Jan, Mar, May, Jul, Aug, Oct, Dec
            3 | 5 | 8 | 10 => 30,                // Apr, Jun, Sep, Nov
            _ => 30,
        }
    }

    /// Convert days since epoch to (year, month, day)
    fn days_to_date(mut days: u64) -> (i32, usize, u32) {
        // Jan 1, 1970 was a Thursday (day 0)
        let mut year = 1970i32;
        
        // Handle negative days (before epoch) - shouldn't happen with unix timestamps
        // but handle gracefully
        if days == 0 {
            return (year, 0, 1); // Jan 1, 1970
        }
        
        // Calculate year
        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }
        
        // Calculate month and day
        let is_leap = Self::is_leap_year(year);
        let mut month = 0usize; // 0 = January
        let mut day = days as u32;
        
        while month < 12 {
            let days_in_this_month = Self::days_in_month(month, is_leap);
            if day < days_in_this_month {
                break;
            }
            day -= days_in_this_month;
            month += 1;
        }
        
        // day is 0-indexed, so add 1
        (year, month, day + 1)
    }

    /// Format timestamp as "Sat, 12th Dec" format
    pub fn format_timestamp_date(timestamp: &str) -> String {
        // Try to parse as unix timestamp
        if let Ok(secs) = timestamp.parse::<u64>() {
            use std::time::{SystemTime, UNIX_EPOCH};
            
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
                let weekday_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", 
                                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
                
                // Calculate day of week (Jan 1, 1970 was a Thursday = 4)
                let day_of_week = ((days_since_epoch + 4) % 7) as usize;
                let weekday = weekday_names[day_of_week];
                
                // Calculate accurate date
                let (_year, month, day) = Self::days_to_date(days_since_epoch);
                let month_name = month_names[month];
                
                // Add ordinal suffix
                let suffix = match day {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };
                
                format!("{}, {}{} {}", weekday, day, suffix, month_name)
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

