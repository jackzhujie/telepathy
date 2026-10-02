#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub merge_threshold: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            min_length: 50,
            max_length: 2000,
            merge_threshold: 500,
        }
    }
}

pub fn chunk_text(text: &str, config: &ChunkConfig) -> Vec<String> {
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut buffer = String::new();

    for paragraph in paragraphs {
        if buffer.is_empty() {
            buffer = paragraph.to_string();
        } else if buffer.len() + paragraph.len() + 2 <= config.merge_threshold {
            buffer.push_str("\n\n");
            buffer.push_str(paragraph);
        } else {
            if buffer.len() > config.max_length {
                chunks.extend(split_long_chunk(&buffer, config.max_length));
            } else if buffer.len() >= config.min_length {
                chunks.push(buffer.clone());
            } else {
                // Too short alone, try merging with next
                chunks.push(buffer.clone());
            }
            buffer = paragraph.to_string();
        }
    }

    if !buffer.is_empty() {
        if buffer.len() > config.max_length {
            chunks.extend(split_long_chunk(&buffer, config.max_length));
        } else if buffer.len() >= config.min_length {
            chunks.push(buffer);
        } else if !chunks.is_empty() {
            // Merge leftover short chunk with the last one
            let last = chunks.pop().unwrap();
            chunks.push(format!("{}\n\n{}", last, buffer));
        } else {
            chunks.push(buffer);
        }
    }

    chunks
}

fn split_long_chunk(text: &str, max_length: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let delimiters = ['。', '.', '！', '!', '？', '?', '\n'];
    let chars: Vec<char> = text.chars().collect();
    let mut char_start = 0;

    while char_start < chars.len() {
        let remaining_len: usize = chars[char_start..].iter().map(|c| c.len_utf8()).sum();
        if remaining_len <= max_length {
            let s: String = chars[char_start..].iter().collect();
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                chunks.push(trimmed);
            }
            break;
        }

        // Find the split position in character index
        let mut byte_count = 0usize;
        let mut split_char_pos = char_start;

        for (i, &c) in chars[char_start..].iter().enumerate() {
            byte_count += c.len_utf8();
            if byte_count > max_length {
                break;
            }
            if delimiters.contains(&c) {
                split_char_pos = char_start + i + 1;
            }
        }

        if split_char_pos == char_start {
            // No delimiter found, force split at max_length boundary
            byte_count = 0;
            for (i, &c) in chars[char_start..].iter().enumerate() {
                byte_count += c.len_utf8();
                if byte_count > max_length {
                    split_char_pos = char_start + i;
                    break;
                }
            }
            if split_char_pos == char_start {
                split_char_pos = chars.len();
            }
        }

        let chunk: String = chars[char_start..split_char_pos].iter().collect();
        let trimmed = chunk.trim().to_string();
        if !trimmed.is_empty() {
            chunks.push(trimmed);
        }
        char_start = split_char_pos;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let text = "第一段内容，足够长的内容用于测试分块效果。\n\n第二段内容，同样是足够的长度。\n\n第三段内容，测试第三段的分块。";
        let config = ChunkConfig::default();
        let chunks = chunk_text(text, &config);
        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|c| c.len() >= config.min_length));
    }

    #[test]
    fn test_merge_short_paragraphs() {
        let text = "短文1\n\n短文2\n\n短文3";
        let config = ChunkConfig {
            min_length: 5,
            ..Default::default()
        };
        let chunks = chunk_text(text, &config);
        assert!(chunks.len() <= 3);
    }

    #[test]
    fn test_split_long_chunk() {
        let long_text = "这是一个很长的句子，用来测试分块引擎能否正确处理超长文本。".repeat(100);
        let config = ChunkConfig::default();
        let chunks = chunk_text(&long_text, &config);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= config.max_length));
    }

    #[test]
    fn test_empty_text() {
        let config = ChunkConfig::default();
        let chunks = chunk_text("", &config);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_single_paragraph() {
        let text = "这是一个单独的段落，长度足够通过最小阈值。";
        let config = ChunkConfig::default();
        let chunks = chunk_text(text, &config);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_custom_config() {
        let text = "A\n\nB\n\nC\n\nD";
        let config = ChunkConfig {
            min_length: 1,
            max_length: 100,
            merge_threshold: 10,
        };
        let chunks = chunk_text(text, &config);
        assert!(chunks.len() >= 1);
    }
}
