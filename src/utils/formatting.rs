use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub enum ContentSegment {
    Text(String),
    InlineCode(String),
    CodeBlock { language: String, code: String },
}

pub fn parse_message_content(content: &str) -> Vec<ContentSegment> {
    let mut segments = Vec::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        // Try to match code block first (triple backticks)
        if let Some(code_block) = extract_code_block(remaining) {
            if !code_block.prefix.is_empty() {
                segments.push(ContentSegment::Text(code_block.prefix));
            }
            segments.push(ContentSegment::CodeBlock {
                language: code_block.language,
                code: code_block.code,
            });
            remaining = &remaining[code_block.end..];
        } else if let Some(inline_code) = extract_inline_code(remaining) {
            if !inline_code.prefix.is_empty() {
                segments.push(ContentSegment::Text(inline_code.prefix));
            }
            segments.push(ContentSegment::InlineCode(inline_code.code));
            remaining = &remaining[inline_code.end..];
        } else {
            segments.push(ContentSegment::Text(remaining.to_string()));
            break;
        }
    }

    segments
}

struct CodeBlockMatch {
    prefix: String,
    language: String,
    code: String,
    end: usize,
}

fn extract_code_block(text: &str) -> Option<CodeBlockMatch> {
    let re = Regex::new(r"^([\s\S]*?)```(\w*)\n([\s\S]*?)\n```").ok()?;
    let captures = re.captures(text)?;

    let prefix = captures
        .get(1)
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let language = captures
        .get(2)
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let code = captures
        .get(3)
        .map(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let end = captures.get(0).map(|m| m.end()).unwrap_or(0);

    Some(CodeBlockMatch {
        prefix,
        language,
        code,
        end,
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
