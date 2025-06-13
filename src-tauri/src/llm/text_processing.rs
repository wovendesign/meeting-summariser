/// Split text into chunks at natural breaking points
pub fn split_text_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while current_pos < chars.len() {
        let end_pos = std::cmp::min(current_pos + max_chars, chars.len());

        // Try to find a good breaking point (sentence end, paragraph break, or whitespace)
        let mut break_pos = end_pos;
        if end_pos < chars.len() {
            break_pos = find_optimal_break_point(&chars, current_pos, end_pos);
        }

        let chunk: String = chars[current_pos..break_pos].iter().collect();
        chunks.push(chunk.trim().to_string());
        current_pos = break_pos;
    }

    chunks
}

fn find_optimal_break_point(chars: &[char], start: usize, max_end: usize) -> usize {
    let chunk_text: String = chars[start..max_end].iter().collect();

    // Look for sentence end first
    if let Some(sentence_end) = find_sentence_end(&chunk_text) {
        let prefix: String = chunk_text.chars().take(sentence_end + 1).collect();
        return start + prefix.chars().count();
    }

    // If no sentence end found, look for paragraph break
    if let Some(para_break) = chunk_text.rfind("\n\n") {
        let prefix: String = chunk_text.chars().take(para_break + 2).collect();
        return start + prefix.chars().count();
    }

    // Finally, look for any whitespace
    if let Some(space) = chunk_text.rfind(' ') {
        let prefix: String = chunk_text.chars().take(space + 1).collect();
        return start + prefix.chars().count();
    }

    max_end
}

fn find_sentence_end(text: &str) -> Option<usize> {
    text.rfind(". ")
        .or_else(|| text.rfind(".\n"))
        .or_else(|| text.rfind("? "))
        .or_else(|| text.rfind("! "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_small_text() {
        let text = "Short text";
        let chunks = split_text_into_chunks(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short text");
    }

    #[test]
    fn test_split_at_sentence_boundary() {
        let text = "First sentence. Second sentence. Third sentence.";
        let chunks = split_text_into_chunks(text, 20);
        assert!(chunks.len() > 1);
        assert!(chunks[0].ends_with('.'));
    }

    #[test]
    fn test_split_at_paragraph_boundary() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = split_text_into_chunks(text, 25);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_split_preserves_content() {
        let text = "This is a test. It has multiple sentences. Some are longer than others. The last one is short.";
        let chunks = split_text_into_chunks(text, 30);
        let reconstructed = chunks.join(" ").replace("  ", " ");
        // Remove extra spaces that might be introduced
        let normalized_original = text.replace("  ", " ");
        assert_eq!(reconstructed.trim(), normalized_original.trim());
    }

    #[test]
    fn test_split_respects_max_chars() {
        let text = "A".repeat(1000);
        let chunks = split_text_into_chunks(&text, 100);
        for chunk in &chunks {
            assert!(chunk.len() <= 100, "Chunk too long: {}", chunk.len());
        }
    }

    #[test]
    fn test_find_sentence_end() {
        assert_eq!(find_sentence_end("Hello. World"), Some(5));
        assert_eq!(find_sentence_end("Hello? World"), Some(5));
        assert_eq!(find_sentence_end("Hello! World"), Some(5));
        assert_eq!(find_sentence_end("Hello.\nWorld"), Some(5));
        assert_eq!(find_sentence_end("No sentence end"), None);
    }

    #[test]
    fn test_empty_text() {
        let chunks = split_text_into_chunks("", 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "");
    }

    #[test]
    fn test_whitespace_only() {
        let chunks = split_text_into_chunks("   \n\t  ", 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "");
    }
}
