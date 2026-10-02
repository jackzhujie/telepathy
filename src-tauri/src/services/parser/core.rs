#![allow(dead_code)]
use crate::errors::AppError;
use std::fs;
use std::path::Path;
use calamine::{Reader, Data, open_workbook_auto};

fn is_likely_binary(bytes: &[u8]) -> bool {
    if bytes.len() >= 2 {
        if (bytes[0] == 0xff && bytes[1] == 0xfe) || (bytes[0] == 0xfe && bytes[1] == 0xff) {
            return false;
        }
    }

    let null_count = bytes.iter().take(1000).filter(|&&b| b == 0).count();
    if null_count > bytes.len().min(1000) / 3 {
        return true;
    }

    let control_count = bytes
        .iter()
        .take(1000)
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
        .count();
    control_count > bytes.len().min(1000) / 10
}

pub fn parse_text_file(path: &Path) -> Result<String, AppError> {
    let raw_bytes =
        fs::read(path).map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))?;

    if raw_bytes.len() >= 2 {
        if raw_bytes[0] == 0xff && raw_bytes[1] == 0xfe {
            let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&raw_bytes);
            return Ok(decoded.into_owned());
        }
        if raw_bytes[0] == 0xfe && raw_bytes[1] == 0xff {
            let (decoded, _, _) = encoding_rs::UTF_16BE.decode(&raw_bytes);
            return Ok(decoded.into_owned());
        }
    }

    if is_likely_binary(&raw_bytes) {
        return Err(AppError::Internal(
            "File appears to be binary, not a text file".to_string(),
        ));
    }

    if let Ok(text) = String::from_utf8(raw_bytes.clone()) {
        return Ok(text);
    }

    let (decoded, _, _) = encoding_rs::GB18030.decode(&raw_bytes);
    Ok(decoded.into_owned())
}

pub fn parse_pdf(path: &Path) -> Result<String, AppError> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| AppError::Internal(format!("Failed to extract PDF text: {}", e)))?;
    Ok(text)
}

pub fn parse_docx(path: &Path) -> Result<String, AppError> {
    let file = std::fs::File::open(path)
        .map_err(|e| AppError::Internal(format!("Failed to open docx file: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Internal(format!("Failed to open docx as zip: {}", e)))?;

    let mut document_xml = archive.by_name("word/document.xml")
        .map_err(|e| AppError::Internal(format!("Failed to find word/document.xml: {}", e)))?;

    let mut xml_content = String::new();
    std::io::Read::read_to_string(&mut document_xml, &mut xml_content)
        .map_err(|e| AppError::Internal(format!("Failed to read word/document.xml: {}", e)))?;

    let mut reader = quick_xml::Reader::from_str(&xml_content);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut out_text = String::new();
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                if e.name().as_ref() == b"w:t" {
                    in_text = true;
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if e.name().as_ref() == b"w:t" {
                    in_text = false;
                } else if e.name().as_ref() == b"w:p" {
                    out_text.push('\n');
                }
            }
            Ok(quick_xml::events::Event::Text(e)) => {
                if in_text {
                    match e.unescape() {
                        Ok(txt) => out_text.push_str(&txt),
                        Err(_) => {
                            if let Ok(decoded) = reader.decoder().decode(&e) {
                                out_text.push_str(&decoded);
                            }
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(AppError::Internal(format!("XML parse error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(out_text.trim().to_string())
}

pub fn parse_excel(path: &Path) -> Result<String, AppError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::Internal(format!("Failed to open Excel workbook: {}", e)))?;

    let mut out_text = String::new();
    let sheets = workbook.sheet_names().to_vec();

    for sheet_name in sheets {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            out_text.push_str(&format!("--- Sheet: {} ---\n", sheet_name));
            for row in range.rows() {
                for (i, cell) in row.iter().enumerate() {
                    if i > 0 {
                        out_text.push(',');
                    }
                    match cell {
                        Data::Empty => {}
                        Data::String(s) => out_text.push_str(s),
                        Data::Float(f) => {
                            use std::fmt::Write;
                            let _ = write!(&mut out_text, "{}", f);
                        }
                        Data::Int(i) => {
                            use std::fmt::Write;
                            let _ = write!(&mut out_text, "{}", i);
                        }
                        Data::Bool(b) => out_text.push_str(if *b { "true" } else { "false" }),
                        Data::DateTime(dt) => {
                            use std::fmt::Write;
                            let _ = write!(&mut out_text, "{}", dt);
                        }
                        Data::DateTimeIso(dt_iso) => out_text.push_str(dt_iso),
                        Data::DurationIso(dur_iso) => out_text.push_str(dur_iso),
                        Data::Error(err) => {
                            use std::fmt::Write;
                            let _ = write!(&mut out_text, "Error: {:?}", err);
                        }
                    }
                }
                out_text.push('\n');
            }
            out_text.push('\n');
        }
    }

    Ok(out_text.trim().to_string())
}

pub fn parse_pptx(path: &Path) -> Result<String, AppError> {
    let file = std::fs::File::open(path)
        .map_err(|e| AppError::Internal(format!("Failed to open pptx file: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Internal(format!("Failed to open pptx as zip: {}", e)))?;

    let mut slide_names = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name.to_string());
            }
        }
    }

    slide_names.sort_by_key(|name| {
        name.strip_prefix("ppt/slides/slide")
            .and_then(|s| s.strip_suffix(".xml"))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
    });

    let mut out_text = String::new();

    for (idx, slide_name) in slide_names.iter().enumerate() {
        let mut slide_xml = archive.by_name(slide_name)
            .map_err(|e| AppError::Internal(format!("Failed to read slide XML {}: {}", slide_name, e)))?;
        let mut xml_content = String::new();
        std::io::Read::read_to_string(&mut slide_xml, &mut xml_content)
            .map_err(|e| AppError::Internal(format!("Failed to read slide XML: {}", e)))?;

        let mut reader = quick_xml::Reader::from_str(&xml_content);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut slide_text = String::new();
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    if e.name().as_ref() == b"a:t" {
                        in_text = true;
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    if e.name().as_ref() == b"a:t" {
                        in_text = false;
                    } else if e.name().as_ref() == b"a:p" {
                        slide_text.push(' ');
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if in_text {
                        match e.unescape() {
                            Ok(txt) => slide_text.push_str(&txt),
                            Err(_) => {
                                if let Ok(decoded) = reader.decoder().decode(&e) {
                                    slide_text.push_str(&decoded);
                                }
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(AppError::Internal(format!("XML slide parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        if !slide_text.trim().is_empty() {
            out_text.push_str(&format!("--- Slide {} ---\n", idx + 1));
            out_text.push_str(&slide_text.trim());
            out_text.push_str("\n\n");
        }
    }

    Ok(out_text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_text_file() {
        let dir = std::env::temp_dir().join("telepathy_test");
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.txt");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "Hello, Telepathy!").unwrap();

        let result = parse_text_file(&path).unwrap();
        assert!(result.contains("Hello, Telepathy!"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_parse_markdown_file() {
        let dir = std::env::temp_dir().join("telepathy_test_md");
        fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# Title\nSome content").unwrap();

        let result = parse_text_file(&path).unwrap();
        assert!(result.contains("# Title"));
        assert!(result.contains("Some content"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_office_parsers_compilation() {
        let dummy_path = std::path::Path::new("non_existent_file.docx");
        let _ = parse_docx(dummy_path);
        let _ = parse_excel(dummy_path);
        let _ = parse_pptx(dummy_path);
    }
}
