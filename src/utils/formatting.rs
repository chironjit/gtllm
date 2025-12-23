use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub enum ContentSegment {
    Text(String),
    InlineCode(String),
    CodeBlock { language: String, code: String },
    Header { level: usize, text: String },
    ListItem(String),
    Blockquote(String),
    HorizontalRule,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InlineSegment {
    Text(String),
    Bold(String),
    Italic(String),
    Link { text: String, url: String },
    InlineCode(String),
}

pub fn parse_message_content(content: &str) -> Vec<ContentSegment> {
    let mut segments = Vec::new();
    let mut remaining = content;
    let mut in_code_block = false;
    let mut code_block_start = 0;

    // First pass: extract code blocks (they have highest priority)
    while !remaining.is_empty() {
        if let Some(code_block) = extract_code_block(remaining) {
            if !code_block.prefix.is_empty() {
                // Process the prefix for other markdown elements
                let prefix_segments = parse_text_segments(&code_block.prefix);
                segments.extend(prefix_segments);
            }
            segments.push(ContentSegment::CodeBlock {
                language: code_block.language,
                code: code_block.code,
            });
            remaining = &remaining[code_block.end..];
        } else {
            // No more code blocks, process remaining text
            if !remaining.is_empty() {
                let text_segments = parse_text_segments(remaining);
                segments.extend(text_segments);
            }
            break;
        }
    }

    segments
}

fn parse_text_segments(text: &str) -> Vec<ContentSegment> {
    let mut segments = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check for horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            segments.push(ContentSegment::HorizontalRule);
            i += 1;
            continue;
        }

        // Check for headers (###, ##, #) - use trimmed to handle indented headers
        if let Some(header) = extract_header(trimmed) {
            segments.push(ContentSegment::Header {
                level: header.level,
                text: header.text,
            });
            i += 1;
            continue;
        }

        // Check for blockquote
        if trimmed.starts_with('>') {
            let quote_text = trimmed.strip_prefix('>').unwrap_or("").trim().to_string();
            if !quote_text.is_empty() {
                segments.push(ContentSegment::Blockquote(quote_text));
            }
            i += 1;
            continue;
        }

        // Check for list items (-, *, 1.) - use trimmed to handle indented lists
        if let Some(list_item) = extract_list_item(trimmed) {
            segments.push(ContentSegment::ListItem(list_item));
            i += 1;
            continue;
        }

        // Regular text line - store as text (inline markdown will be parsed during rendering)
        // Join consecutive text lines to preserve paragraphs
        if !line.is_empty() {
            // Check if previous segment was also text, and if so, append to it
            if let Some(ContentSegment::Text(ref mut last_text)) = segments.last_mut() {
                // Append with a space to separate lines
                last_text.push_str(" ");
                last_text.push_str(line);
            } else {
                segments.push(ContentSegment::Text(line.to_string()));
            }
        } else {
            // Empty line - add as text to preserve spacing, but don't merge with previous
            segments.push(ContentSegment::Text("\n".to_string()));
        }
        i += 1;
    }

    segments
}

fn parse_inline_markdown(text: &str) -> Vec<ContentSegment> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Try inline code first (highest priority for inline)
        if let Some(inline_code) = extract_inline_code(remaining) {
            if !inline_code.prefix.is_empty() {
                let inline_segments = parse_inline_elements(&inline_code.prefix);
                for seg in inline_segments {
                    match seg {
                        InlineSegment::Text(t) => segments.push(ContentSegment::Text(t)),
                        InlineSegment::Bold(t) => segments.push(ContentSegment::Text(format!("**{}**", t))),
                        InlineSegment::Italic(t) => segments.push(ContentSegment::Text(format!("*{}*", t))),
                        InlineSegment::Link { text: t, url: u } => segments.push(ContentSegment::Text(format!("[{}]({})", t, u))),
                        InlineSegment::InlineCode(c) => segments.push(ContentSegment::InlineCode(c)),
                    }
                }
            }
            segments.push(ContentSegment::InlineCode(inline_code.code));
            remaining = &remaining[inline_code.end..];
        } else {
            // Parse inline elements in remaining text
            let inline_segments = parse_inline_elements(remaining);
            for seg in inline_segments {
                match seg {
                    InlineSegment::Text(t) => segments.push(ContentSegment::Text(t)),
                    InlineSegment::Bold(t) => segments.push(ContentSegment::Text(format!("**{}**", t))),
                    InlineSegment::Italic(t) => segments.push(ContentSegment::Text(format!("*{}*", t))),
                    InlineSegment::Link { text: t, url: u } => segments.push(ContentSegment::Text(format!("[{}]({})", t, u))),
                    InlineSegment::InlineCode(c) => segments.push(ContentSegment::InlineCode(c)),
                }
            }
            break;
        }
    }

    segments
}

pub fn parse_inline_elements(text: &str) -> Vec<InlineSegment> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Try link first [text](url)
        if let Some(link) = extract_link(remaining) {
            if !link.prefix.is_empty() {
                segments.push(InlineSegment::Text(link.prefix.to_string()));
            }
            segments.push(InlineSegment::Link {
                text: link.text,
                url: link.url,
            });
            remaining = &remaining[link.end..];
        }
        // Try bold **text** or __text__
        else if let Some(bold) = extract_bold(remaining) {
            if !bold.prefix.is_empty() {
                segments.push(InlineSegment::Text(bold.prefix.to_string()));
            }
            segments.push(InlineSegment::Bold(bold.text));
            remaining = &remaining[bold.end..];
        }
        // Try italic *text* or _text_
        else if let Some(italic) = extract_italic(remaining) {
            if !italic.prefix.is_empty() {
                segments.push(InlineSegment::Text(italic.prefix.to_string()));
            }
            segments.push(InlineSegment::Italic(italic.text));
            remaining = &remaining[italic.end..];
        }
        // Try inline code `code`
        else if let Some(code) = extract_inline_code(remaining) {
            if !code.prefix.is_empty() {
                segments.push(InlineSegment::Text(code.prefix.to_string()));
            }
            segments.push(InlineSegment::InlineCode(code.code));
            remaining = &remaining[code.end..];
        } else {
            segments.push(InlineSegment::Text(remaining.to_string()));
            break;
        }
    }

    segments
}

struct LinkMatch {
    prefix: String,
    text: String,
    url: String,
    end: usize,
}

fn extract_link(text: &str) -> Option<LinkMatch> {
    let re = Regex::new(r"^([\s\S]*?)\[([^\]]+)\]\(([^)]+)\)").ok()?;
    let captures = re.captures(text)?;

    let prefix = captures.get(1)?.as_str().to_string();
    let link_text = captures.get(2)?.as_str().to_string();
    let url = captures.get(3)?.as_str().to_string();
    let end = captures.get(0)?.end();

    Some(LinkMatch {
        prefix,
        text: link_text,
        url,
        end,
    })
}

struct BoldMatch {
    prefix: String,
    text: String,
    end: usize,
}

fn extract_bold(text: &str) -> Option<BoldMatch> {
    // Match **text** or __text__
    // Try **text** first - match any characters (including <, >, :, etc.) between ** markers
    // Pattern: (prefix)(**)(content)(**)
    // Use [\s\S] to match any character including newlines
    // Non-greedy matching ensures we match the first closing **
    if let Some(captures) = Regex::new(r"^(.*?)\*\*([^\*]+?)\*\*").ok()?.captures(text) {
        let prefix = captures.get(1)?.as_str().to_string();
        let text_content = captures.get(2)?.as_str().to_string();
        let end = captures.get(0)?.end();
        // Only return if we actually matched some content (not empty)
        if !text_content.is_empty() {
            return Some(BoldMatch {
                prefix,
                text: text_content,
                end,
            });
        }
    }
    // Try **text** with [\s\S] for cases where content might contain newlines or special chars
    // This is a fallback for the above pattern
    if let Some(captures) = Regex::new(r"^(.*?)\*\*([\s\S]+?)\*\*").ok()?.captures(text) {
        let prefix = captures.get(1)?.as_str().to_string();
        let text_content = captures.get(2)?.as_str().to_string();
        let end = captures.get(0)?.end();
        // Only return if we actually matched some content (not empty)
        if !text_content.is_empty() {
            return Some(BoldMatch {
                prefix,
                text: text_content,
                end,
            });
        }
    }
    // Try __text__
    if let Some(captures) = Regex::new(r"^(.*?)__([^_]+?)__").ok()?.captures(text) {
        let prefix = captures.get(1)?.as_str().to_string();
        let text_content = captures.get(2)?.as_str().to_string();
        let end = captures.get(0)?.end();
        // Only return if we actually matched some content (not empty)
        if !text_content.is_empty() {
            return Some(BoldMatch {
                prefix,
                text: text_content,
                end,
            });
        }
    }
    // Try __text__ with [\s\S] as fallback
    if let Some(captures) = Regex::new(r"^(.*?)__([\s\S]+?)__").ok()?.captures(text) {
        let prefix = captures.get(1)?.as_str().to_string();
        let text_content = captures.get(2)?.as_str().to_string();
        let end = captures.get(0)?.end();
        // Only return if we actually matched some content (not empty)
        if !text_content.is_empty() {
            return Some(BoldMatch {
                prefix,
                text: text_content,
                end,
            });
        }
    }
    None
}

struct ItalicMatch {
    prefix: String,
    text: String,
    end: usize,
}

fn extract_italic(text: &str) -> Option<ItalicMatch> {
    // Match *text* (but not **text**)
    // Try *text* first
    if let Some(captures) = Regex::new(r"^([\s\S]*?)(?<!\*)\*([^*\n]+?)\*(?!\*)").ok()?.captures(text) {
        let prefix = captures.get(1)?.as_str().to_string();
        let text_content = captures.get(2)?.as_str().to_string();
        let end = captures.get(0)?.end();
        return Some(ItalicMatch {
            prefix,
            text: text_content,
            end,
        });
    }
    // Try _text_ (but not __text__)
    if let Some(captures) = Regex::new(r"^([\s\S]*?)(?<!_)_([^_\n]+?)_(?!_)").ok()?.captures(text) {
        let prefix = captures.get(1)?.as_str().to_string();
        let text_content = captures.get(2)?.as_str().to_string();
        let end = captures.get(0)?.end();
        return Some(ItalicMatch {
            prefix,
            text: text_content,
            end,
        });
    }
    None
}

struct HeaderMatch {
    level: usize,
    text: String,
}

fn extract_header(line: &str) -> Option<HeaderMatch> {
    // Match headers with optional leading whitespace: ### Header or   ### Header
    let trimmed = line.trim();
    let re = Regex::new(r"^(#{1,6})\s+(.+)$").ok()?;
    let captures = re.captures(trimmed)?;
    
    let level = captures.get(1)?.as_str().len();
    let text = captures.get(2)?.as_str().trim().to_string();
    
    if text.is_empty() {
        return None;
    }
    
    Some(HeaderMatch { level, text })
}

fn extract_list_item(line: &str) -> Option<String> {
    // Match unordered lists (-, *, +) or ordered lists (1., 2., etc.)
    // Handle optional leading whitespace
    let trimmed = line.trim();
    let re = Regex::new(r"^[-*+]\s+(.+)$|^\d+\.\s+(.+)$").ok()?;
    let captures = re.captures(trimmed)?;
    
    if let Some(m) = captures.get(1) {
        let text = m.as_str().trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    } else if let Some(m) = captures.get(2) {
        let text = m.as_str().trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

struct CodeBlockMatch {
    prefix: String,
    language: String,
    code: String,
    end: usize,
}

fn extract_code_block(text: &str) -> Option<CodeBlockMatch> {
    // Find the opening ``` with optional language
    let opening_pos = text.find("```")?;
    let prefix = text[..opening_pos].to_string();
    let after_opening = &text[opening_pos + 3..];
    
    // Extract language (if present) - it's the word immediately after ```
    let (language, code_start) = if let Some(lang_end) = after_opening.find(|c: char| c == '\n' || c.is_whitespace() && c != ' ') {
        let lang = after_opening[..lang_end].trim().to_string();
        let start = lang_end + 1;
        (lang, start)
    } else {
        // No language, code starts immediately or after whitespace
        let trimmed = after_opening.trim_start();
        let lang_start_offset = after_opening.len() - trimmed.len();
        ("".to_string(), lang_start_offset)
    };
    
    // Find the closing ``` - use non-greedy matching
    let code_and_closing = &after_opening[code_start..];
    let closing_pos = code_and_closing.find("```")?;
    let code = code_and_closing[..closing_pos]
        .trim_start_matches('\n')
        .trim_end_matches('\n')
        .to_string();
    
    let total_end = opening_pos + 3 + code_start + closing_pos + 3;

    Some(CodeBlockMatch {
        prefix,
        language,
        code,
        end: total_end,
    })
}

struct InlineCodeMatch {
    prefix: String,
    code: String,
    end: usize,
}

fn extract_inline_code(text: &str) -> Option<InlineCodeMatch> {
    let re = Regex::new(r"^([\s\S]*?)`([^`]+)`").ok()?;
    let captures = re.captures(text)?;

    let prefix = captures
        .get(1)
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let code = captures
        .get(2)
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let end = captures.get(0).map(|m| m.end()).unwrap_or(0);

    Some(InlineCodeMatch { prefix, code, end })
}
