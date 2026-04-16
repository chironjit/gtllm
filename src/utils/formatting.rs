#[derive(Clone, Debug, PartialEq)]
pub enum ContentSegment {
    Paragraph(String),
    Header { level: usize, text: String },
    CodeBlock { language: String, code: String },
    FormulaBlock(String),
    Table(TableBlock),
    List(ListBlock),
    Blockquote(Vec<ContentSegment>),
    HorizontalRule,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableBlock {
    pub headers: Vec<String>,
    pub alignments: Vec<TableAlignment>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListBlock {
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    pub text: String,
    pub children: Vec<ListBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InlineSegment {
    Text(String),
    Bold(Vec<InlineSegment>),
    Italic(Vec<InlineSegment>),
    BoldItalic(Vec<InlineSegment>),
    Link { text: Vec<InlineSegment>, url: String },
    InlineCode(String),
    Formula(String),
}

pub fn parse_message_content(content: &str) -> Vec<ContentSegment> {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.trim().is_empty() {
        return Vec::new();
    }

    let lines = normalized
        .split('\n')
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    let mut index = 0;
    parse_blocks(&lines, &mut index)
}

pub fn parse_inline_elements(text: &str) -> Vec<InlineSegment> {
    merge_inline_text(parse_inline_impl(text))
}

fn parse_blocks(lines: &[String], index: &mut usize) -> Vec<ContentSegment> {
    let mut segments = Vec::new();

    while *index < lines.len() {
        let line = &lines[*index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            *index += 1;
            continue;
        }

        if let Some((code_block, next_index)) = parse_fenced_code_block(lines, *index) {
            segments.push(code_block);
            *index = next_index;
            continue;
        }

        if let Some((formula_block, next_index)) = parse_formula_block(lines, *index) {
            segments.push(formula_block);
            *index = next_index;
            continue;
        }

        if let Some((table, next_index)) = parse_table(lines, *index) {
            segments.push(table);
            *index = next_index;
            continue;
        }

        if is_horizontal_rule(trimmed) {
            segments.push(ContentSegment::HorizontalRule);
            *index += 1;
            continue;
        }

        if let Some((level, text)) = parse_header(trimmed) {
            segments.push(ContentSegment::Header { level, text });
            *index += 1;
            continue;
        }

        if trimmed.starts_with('>') {
            let (blockquote, next_index) = parse_blockquote(lines, *index);
            segments.push(blockquote);
            *index = next_index;
            continue;
        }

        if let Some(marker) = parse_list_marker(line) {
            let (list, next_index) = parse_list(lines, *index, marker.indent, marker.ordered);
            segments.push(ContentSegment::List(list));
            *index = next_index;
            continue;
        }

        let (paragraph, next_index) = parse_paragraph(lines, *index);
        if !paragraph.is_empty() {
            segments.push(ContentSegment::Paragraph(paragraph));
        }
        *index = next_index;
    }

    segments
}

fn parse_fenced_code_block(lines: &[String], start: usize) -> Option<(ContentSegment, usize)> {
    let opening = lines.get(start)?.trim_start();
    if !opening.starts_with("```") {
        return None;
    }

    let language = opening
        .strip_prefix("```")
        .unwrap_or("")
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();

    let mut code_lines = Vec::new();
    let mut index = start + 1;

    while index < lines.len() {
        let line = &lines[index];
        if line.trim_start().starts_with("```") {
            return Some((
                ContentSegment::CodeBlock {
                    language,
                    code: code_lines.join("\n"),
                },
                index + 1,
            ));
        }

        code_lines.push(line.clone());
        index += 1;
    }

    Some((
        ContentSegment::CodeBlock {
            language,
            code: code_lines.join("\n"),
        },
        index,
    ))
}

fn parse_formula_block(lines: &[String], start: usize) -> Option<(ContentSegment, usize)> {
    let trimmed = lines.get(start)?.trim();
    if trimmed == "$$" {
        let mut formula_lines = Vec::new();
        let mut index = start + 1;

        while index < lines.len() {
            let line = &lines[index];
            if line.trim() == "$$" {
                return Some((
                    ContentSegment::FormulaBlock(formula_lines.join("\n").trim().to_string()),
                    index + 1,
                ));
            }

            formula_lines.push(line.clone());
            index += 1;
        }

        return Some((
            ContentSegment::FormulaBlock(formula_lines.join("\n").trim().to_string()),
            index,
        ));
    }

    if trimmed.starts_with("$$") && trimmed.ends_with("$$") && trimmed.len() > 4 {
        return Some((
            ContentSegment::FormulaBlock(trimmed[2..trimmed.len() - 2].trim().to_string()),
            start + 1,
        ));
    }

    None
}

fn parse_blockquote(lines: &[String], start: usize) -> (ContentSegment, usize) {
    let mut quote_lines = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let line = &lines[index];
        let trimmed = line.trim_start();

        if trimmed.starts_with('>') {
            let without_marker = trimmed.trim_start_matches('>');
            quote_lines.push(without_marker.strip_prefix(' ').unwrap_or(without_marker).to_string());
            index += 1;
            continue;
        }

        if trimmed.is_empty() {
            quote_lines.push(String::new());
            index += 1;
            continue;
        }

        break;
    }

    (
        ContentSegment::Blockquote(parse_message_content(&quote_lines.join("\n"))),
        index,
    )
}

fn parse_paragraph(lines: &[String], start: usize) -> (String, usize) {
    let mut paragraph_lines = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let line = &lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            break;
        }

        if index != start && is_block_start(line) {
            break;
        }

        paragraph_lines.push(trimmed.to_string());
        index += 1;
    }

    (paragraph_lines.join("\n"), index)
}

fn parse_table(lines: &[String], start: usize) -> Option<(ContentSegment, usize)> {
    let header_line = lines.get(start)?;
    let separator_line = lines.get(start + 1)?;

    let headers = parse_table_row(header_line)?;
    let alignments = parse_table_separator(separator_line)?;

    if headers.is_empty() || alignments.is_empty() || headers.len() != alignments.len() {
        return None;
    }

    let mut rows = Vec::new();
    let mut index = start + 2;

    while index < lines.len() {
        let line = &lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() || is_block_start(line) {
            break;
        }

        let Some(mut row) = parse_table_row(line) else {
            break;
        };

        normalize_table_cells(&mut row, headers.len());
        rows.push(row);
        index += 1;
    }

    Some((
        ContentSegment::Table(TableBlock {
            headers,
            alignments,
            rows,
        }),
        index,
    ))
}

fn parse_list(lines: &[String], start: usize, base_indent: usize, ordered: bool) -> (ListBlock, usize) {
    let mut items = Vec::new();
    let mut index = start;

    while index < lines.len() {
        let marker = match parse_list_marker(&lines[index]) {
            Some(marker) if marker.indent == base_indent && marker.ordered == ordered => marker,
            _ => break,
        };

        let mut text_parts = Vec::new();
        if !marker.content.is_empty() {
            text_parts.push(marker.content);
        }

        let mut children = Vec::new();
        index += 1;

        while index < lines.len() {
            let line = &lines[index];

            if line.trim().is_empty() {
                let next_non_empty = find_next_non_empty_line(lines, index + 1);
                match next_non_empty.and_then(|next| parse_list_marker(&lines[next])) {
                    Some(next_marker) if next_marker.indent == base_indent && next_marker.ordered == ordered => {
                        index = next_non_empty.unwrap_or(lines.len());
                        break;
                    }
                    Some(next_marker) if next_marker.indent > base_indent => {
                        index = next_non_empty.unwrap_or(lines.len());
                        continue;
                    }
                    _ => {
                        index = next_non_empty.unwrap_or(lines.len());
                        break;
                    }
                }
            }

            if let Some(next_marker) = parse_list_marker(line) {
                if next_marker.indent == base_indent && next_marker.ordered == ordered {
                    break;
                }

                if next_marker.indent > base_indent {
                    let (nested_list, next_index) =
                        parse_list(lines, index, next_marker.indent, next_marker.ordered);
                    children.push(nested_list);
                    index = next_index;
                    continue;
                }
            }

            if leading_indent_width(line) > base_indent {
                text_parts.push(line.trim().to_string());
                index += 1;
                continue;
            }

            break;
        }

        items.push(ListItem {
            text: text_parts.join(" "),
            children,
        });
    }

    (ListBlock { ordered, items }, index)
}

fn parse_inline_impl(text: &str) -> Vec<InlineSegment> {
    let mut segments = Vec::new();
    let mut buffer = String::new();
    let mut index = 0;

    while index < text.len() {
        if let Some(ch) = text[index..].chars().next() {
            if ch == '\\' {
                if let Some(next_char) = text[index + ch.len_utf8()..].chars().next() {
                    buffer.push(next_char);
                    index += ch.len_utf8() + next_char.len_utf8();
                    continue;
                }
            }
        }

        if let Some((segment, next_index)) = parse_inline_code(text, index) {
            flush_text(&mut segments, &mut buffer);
            segments.push(segment);
            index = next_index;
            continue;
        }

        if let Some((segment, next_index)) = parse_inline_formula(text, index) {
            flush_text(&mut segments, &mut buffer);
            segments.push(segment);
            index = next_index;
            continue;
        }

        if let Some((segment, next_index)) = parse_link(text, index) {
            flush_text(&mut segments, &mut buffer);
            segments.push(segment);
            index = next_index;
            continue;
        }

        if let Some((segment, next_index)) = parse_emphasis(text, index) {
            flush_text(&mut segments, &mut buffer);
            segments.push(segment);
            index = next_index;
            continue;
        }

        let ch = text[index..].chars().next().unwrap_or_default();
        buffer.push(ch);
        index += ch.len_utf8();
    }

    flush_text(&mut segments, &mut buffer);
    segments
}

fn parse_inline_code(text: &str, start: usize) -> Option<(InlineSegment, usize)> {
    if !text[start..].starts_with('`') {
        return None;
    }

    let end = find_unescaped(text, start + 1, "`", false)?;
    if end <= start + 1 {
        return None;
    }

    Some((
        InlineSegment::InlineCode(text[start + 1..end].to_string()),
        end + 1,
    ))
}

fn parse_inline_formula(text: &str, start: usize) -> Option<(InlineSegment, usize)> {
    if text[start..].starts_with("$$") {
        let end = find_unescaped(text, start + 2, "$$", false)?;
        let content = text[start + 2..end].trim();
        if content.is_empty() {
            return None;
        }

        return Some((InlineSegment::Formula(content.to_string()), end + 2));
    }

    if !text[start..].starts_with('$') {
        return None;
    }

    let end = find_unescaped(text, start + 1, "$", false)?;
    let content = text[start + 1..end].trim();
    if content.is_empty() {
        return None;
    }

    Some((InlineSegment::Formula(content.to_string()), end + 1))
}

fn parse_link(text: &str, start: usize) -> Option<(InlineSegment, usize)> {
    if !text[start..].starts_with('[') {
        return None;
    }

    let close_bracket = find_matching_delimiter(text, start, b'[', b']')?;
    if text.as_bytes().get(close_bracket + 1) != Some(&b'(') {
        return None;
    }

    let close_paren = find_matching_delimiter(text, close_bracket + 1, b'(', b')')?;
    let link_text = &text[start + 1..close_bracket];
    let url = text[close_bracket + 2..close_paren].trim();

    if url.is_empty() {
        return None;
    }

    Some((
        InlineSegment::Link {
            text: parse_inline_elements(link_text),
            url: url.to_string(),
        },
        close_paren + 1,
    ))
}

fn parse_emphasis(text: &str, start: usize) -> Option<(InlineSegment, usize)> {
    const MARKERS: [(&str, usize); 6] = [
        ("***", 3),
        ("___", 3),
        ("**", 2),
        ("__", 2),
        ("*", 1),
        ("_", 1),
    ];

    for (marker, strength) in MARKERS {
        if !text[start..].starts_with(marker) {
            continue;
        }

        if marker.starts_with('_') && !is_underscore_emphasis_boundary(text, start, marker.len()) {
            continue;
        }

        let end = find_unescaped(text, start + marker.len(), marker, false)?;
        let inner = text[start + marker.len()..end].trim();
        if inner.is_empty() {
            continue;
        }

        let children = parse_inline_elements(inner);
        let segment = match strength {
            3 => InlineSegment::BoldItalic(children),
            2 => InlineSegment::Bold(children),
            _ => InlineSegment::Italic(children),
        };

        return Some((segment, end + marker.len()));
    }

    None
}

fn parse_header(line: &str) -> Option<(usize, String)> {
    let mut level = 0;

    for ch in line.chars() {
        if ch == '#' && level < 6 {
            level += 1;
        } else {
            break;
        }
    }

    if level == 0 || level > 6 {
        return None;
    }

    if !line[level..]
        .chars()
        .next()
        .map(|ch| ch.is_whitespace())
        .unwrap_or(false)
    {
        return None;
    }

    let text = line[level..]
        .trim()
        .trim_end_matches('#')
        .trim()
        .to_string();

    if text.is_empty() {
        return None;
    }

    Some((level, text))
}

fn is_horizontal_rule(line: &str) -> bool {
    let compact = line.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    matches!(compact.as_str(), "---" | "***" | "___")
        || (compact.len() >= 3 && compact.chars().all(|ch| ch == '-') )
        || (compact.len() >= 3 && compact.chars().all(|ch| ch == '*') )
        || (compact.len() >= 3 && compact.chars().all(|ch| ch == '_') )
}

fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return true;
    }

    trimmed.starts_with("```")
        || trimmed == "$$"
        || (trimmed.starts_with("$$") && trimmed.ends_with("$$") && trimmed.len() > 4)
        || trimmed.starts_with('>')
        || is_horizontal_rule(trimmed)
        || parse_header(trimmed).is_some()
        || parse_table_separator(trimmed).is_some()
        || parse_list_marker(line).is_some()
}

#[derive(Clone, Debug, PartialEq)]
struct ListMarker {
    indent: usize,
    ordered: bool,
    content: String,
}

fn parse_list_marker(line: &str) -> Option<ListMarker> {
    let indent = leading_indent_width(line);
    let rest = line.trim_start_matches([' ', '\t']);

    if rest.len() >= 2 {
        let marker = &rest[..1];
        if matches!(marker, "-" | "*" | "+")
            && rest[1..]
                .chars()
                .next()
                .map(|ch| ch.is_whitespace())
                .unwrap_or(false)
        {
            return Some(ListMarker {
                indent,
                ordered: false,
                content: rest[2..].trim().to_string(),
            });
        }
    }

    let digits_len = rest.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits_len == 0 {
        return None;
    }

    let marker_end = digits_len;
    let marker = rest.as_bytes().get(marker_end)?;
    if *marker != b'.' && *marker != b')' {
        return None;
    }

    if !rest[marker_end + 1..]
        .chars()
        .next()
        .map(|ch| ch.is_whitespace())
        .unwrap_or(false)
    {
        return None;
    }

    Some(ListMarker {
        indent,
        ordered: true,
        content: rest[marker_end + 2..].trim().to_string(),
    })
}

fn leading_indent_width(line: &str) -> usize {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .map(|ch| if ch == '\t' { 4 } else { 1 })
        .sum()
}

fn parse_table_row(line: &str) -> Option<Vec<String>> {
    if !line.contains('|') {
        return None;
    }

    let mut cells = split_table_row(line)
        .into_iter()
        .map(|cell| normalize_table_cell_content(cell.trim()))
        .collect::<Vec<_>>();

    if line.trim_start().starts_with('|') && !cells.is_empty() {
        cells.remove(0);
    }

    if line.trim_end().ends_with('|') && !cells.is_empty() {
        cells.pop();
    }

    if cells.is_empty() {
        return None;
    }

    Some(cells)
}

fn parse_table_separator(line: &str) -> Option<Vec<TableAlignment>> {
    let cells = parse_table_row(line)?;
    let mut alignments = Vec::with_capacity(cells.len());

    for cell in cells {
        let trimmed = cell.trim();
        if trimmed.is_empty() || !trimmed.chars().all(|ch| matches!(ch, '-' | ':' | ' ')) {
            return None;
        }

        let dash_count = trimmed.chars().filter(|ch| *ch == '-').count();
        if dash_count < 3 {
            return None;
        }

        let alignment = match (trimmed.starts_with(':'), trimmed.ends_with(':')) {
            (true, true) => TableAlignment::Center,
            (true, false) => TableAlignment::Left,
            (false, true) => TableAlignment::Right,
            (false, false) => TableAlignment::None,
        };
        alignments.push(alignment);
    }

    Some(alignments)
}

fn normalize_table_cells(cells: &mut Vec<String>, expected_len: usize) {
    if cells.len() < expected_len {
        cells.resize(expected_len, String::new());
    } else if cells.len() > expected_len {
        cells.truncate(expected_len);
    }
}

fn split_table_row(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    let mut code_fence_len = 0;
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if escaped {
            current.push(ch);
            escaped = false;
            index += 1;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            current.push(ch);
            index += 1;
            continue;
        }

        if ch == '`' {
            let mut run_len = 1;
            while index + run_len < chars.len() && chars[index + run_len] == '`' {
                run_len += 1;
            }

            for _ in 0..run_len {
                current.push('`');
            }

            if code_fence_len == 0 {
                code_fence_len = run_len;
            } else if code_fence_len == run_len {
                code_fence_len = 0;
            }

            index += run_len;
            continue;
        }

        if ch == '|' && code_fence_len == 0 {
            parts.push(std::mem::take(&mut current));
            index += 1;
            continue;
        }

        current.push(ch);
        index += 1;
    }

    parts.push(current);
    parts
}

fn normalize_table_cell_content(cell: &str) -> String {
    cell.replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
}

fn find_next_non_empty_line(lines: &[String], start: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, line)| !line.trim().is_empty())
        .map(|(index, _)| index)
}

fn flush_text(segments: &mut Vec<InlineSegment>, buffer: &mut String) {
    if !buffer.is_empty() {
        segments.push(InlineSegment::Text(std::mem::take(buffer)));
    }
}

fn merge_inline_text(segments: Vec<InlineSegment>) -> Vec<InlineSegment> {
    let mut merged = Vec::new();

    for segment in segments {
        match segment {
            InlineSegment::Text(text) if text.is_empty() => {}
            InlineSegment::Text(text) => {
                if let Some(InlineSegment::Text(existing)) = merged.last_mut() {
                    existing.push_str(&text);
                } else {
                    merged.push(InlineSegment::Text(text));
                }
            }
            other => merged.push(other),
        }
    }

    merged
}

fn find_unescaped(text: &str, start: usize, needle: &str, allow_newlines: bool) -> Option<usize> {
    let mut search = start;

    while search <= text.len() {
        let remainder = text.get(search..)?;
        let relative = remainder.find(needle)?;
        let found = search + relative;

        if !allow_newlines && text[start..found].contains('\n') {
            return None;
        }

        if !is_escaped(text, found) {
            return Some(found);
        }

        search = found + needle.len();
    }

    None
}

fn is_escaped(text: &str, index: usize) -> bool {
    let bytes = text.as_bytes();
    let mut slash_count = 0;
    let mut cursor = index;

    while cursor > 0 {
        cursor -= 1;
        if bytes[cursor] == b'\\' {
            slash_count += 1;
        } else {
            break;
        }
    }

    slash_count % 2 == 1
}

fn find_matching_delimiter(text: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    if *bytes.get(start)? != open {
        return None;
    }

    let mut depth = 0;
    let mut index = start;

    while index < bytes.len() {
        if is_escaped(text, index) {
            index += 1;
            continue;
        }

        match bytes[index] {
            value if value == open => depth += 1,
            value if value == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }

        index += 1;
    }

    None
}

fn is_underscore_emphasis_boundary(text: &str, start: usize, marker_len: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[start + marker_len..].chars().next();

    before.map(|ch| !ch.is_alphanumeric()).unwrap_or(true)
        && after.map(|ch| !ch.is_whitespace()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_inline_elements, parse_message_content, ContentSegment, InlineSegment, ListBlock,
        TableAlignment,
    };

    #[test]
    fn parses_block_elements() {
        let content = "# Title\n\n| Name | Score |\n| :--- | ---: |\n| Ada | 10 |\n\n- one\n  - nested\n- two\n\n```rust\nlet x = 1;\n```\n\n$$\na^2 + b^2 = c^2\n$$";
        let segments = parse_message_content(content);

        assert!(matches!(segments[0], ContentSegment::Header { level: 1, .. }));
        assert!(matches!(segments[1], ContentSegment::Table(_)));
        assert!(matches!(segments[2], ContentSegment::List(ListBlock { ordered: false, .. })));
        assert!(matches!(segments[3], ContentSegment::CodeBlock { .. }));
        assert!(matches!(segments[4], ContentSegment::FormulaBlock(_)));
    }

    #[test]
    fn parses_table_alignment_and_rows() {
        let content = "| Name | Value |\n| :--- | ---: |\n| left | right |\n| x | y |";
        let segments = parse_message_content(content);

        match &segments[0] {
            ContentSegment::Table(table) => {
                assert_eq!(table.headers, vec!["Name", "Value"]);
                assert_eq!(
                    table.alignments,
                    vec![TableAlignment::Left, TableAlignment::Right]
                );
                assert_eq!(table.rows.len(), 2);
                assert_eq!(table.rows[0], vec!["left", "right"]);
            }
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn preserves_escaped_pipes_and_code_pipes_in_table_cells() {
        let content = "| Name | Note |\n| --- | --- |\n| a \\| b | `x | y` and more |";
        let segments = parse_message_content(content);

        match &segments[0] {
            ContentSegment::Table(table) => {
                assert_eq!(table.rows[0][0], "a \\| b");
                assert_eq!(table.rows[0][1], "`x | y` and more");
            }
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn normalizes_html_breaks_in_table_cells() {
        let content = "| Name | Note |\n| --- | --- |\n| Ada | line one<br>line two |";
        let segments = parse_message_content(content);

        match &segments[0] {
            ContentSegment::Table(table) => {
                assert_eq!(table.rows[0][1], "line one\nline two");
            }
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn parses_inline_elements_recursively() {
        let segments = parse_inline_elements("**bold _italic_** and `code` with $x+y$");

        assert!(matches!(segments[0], InlineSegment::Bold(_)));
        assert!(matches!(segments[2], InlineSegment::InlineCode(_)));
        assert!(matches!(segments[4], InlineSegment::Formula(_)));
    }
}
