use anyhow::Result;
use docx_rs::{
    DocumentChild, ParagraphChild, RunChild, TableCellContent, TableChild, TableRowChild,
};
use pptx_to_md::{PptxContainer, ParserConfig};
use pdf::{content::*, file::FileOptions};
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use calamine::{open_workbook, Reader, Xlsx,
               Data
};
use tokio::sync::RwLock;

pub type FileCache = Arc<RwLock<HashMap<String, CacheFile>>>;

#[derive(Clone)]
pub struct CacheFile {
    pub filename: String,
    pub content: String,
    pub extension : String,
}

pub fn new_file_cache() -> FileCache {
    Arc::new(RwLock::new(HashMap::new()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    TXT,
    PDF,
    DOCX,
    PPTX,
    XLSX,
    CODE,
    MD,
}

impl FileType {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "txt" => Some(FileType::TXT),
            "pdf" => Some(FileType::PDF),
            "docx" => Some(FileType::DOCX),
            "pptx" => Some(FileType::PPTX),
            "xlsx" => Some(FileType::XLSX),
            "md" => Some(FileType::MD),

            // code
            "py" | "js" | "ts" | "jsx" | "tsx" | "vue" | "svelte" |     // Web
            "rs" |                                                      // Rust
            "go" |                                                      // go
            "java" | "kt" | "scala" |                                   // java
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" |          // C/C++
            "cs" | "fs" |                                               // .NET
            "rb" | "php" | "pl" | "pm" |                                // php
            "swift" | "m" | "mm" |                                      // Apple
            "r" | "R" | "jl" |                                          // data science
            "lua" | "tcl" | "awk" | "sed" |                             // Script
            "hs" | "ml" | "elm" | "clj" | "cljs" | "ex" | "exs" |       // function
            "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1" |    // Shell
            "sql" | "prisma" | "graphql" | "gql" |                      // database
            "html" | "htm" | "css" | "scss" | "sass" | "less" |         // Web page
            "xml" | "xsl" | "xslt" |                                    // XML
            "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | // config
            "log" | "env" |                                             // log
            "makefile" | "cmake" | "dockerfile" |                       // build
            "gitignore" | "editorconfig"                                // git
            => Some(FileType::CODE),
            _ => None,
        }
    }
}

pub async fn parse_file(path: &Path, file_bytes: &[u8]) -> Result<String> {
    let extension = path.extension().unwrap().to_str().unwrap();

    let file_type = FileType::from_extension(extension).unwrap();

    let temp_dir = temp_dir();
    let temp_file = temp_dir.join(format!("upload_{}.{}", uuid::Uuid::new_v4(), extension));
    tokio::fs::write(&temp_file, file_bytes).await?;

    let result = match file_type {
        FileType::TXT => parse_directly(&temp_file).await,
        FileType::PDF => parse_pdf(&temp_file).await,
        FileType::DOCX => parse_docx(&temp_file).await,
        FileType::PPTX => parse_pptx(&temp_file).await,
        FileType::XLSX => parse_xlsx(&temp_file).await,
        FileType::CODE => parse_directly(&temp_file).await,
        FileType::MD => parse_directly(&temp_file).await
    };

    let _ = tokio::fs::remove_file(&temp_file).await;

    result
}

async fn parse_directly(path: &Path) -> Result<String> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}

async fn parse_pdf(path: &Path) -> Result<String> {
    let file = FileOptions::cached().open(path)?;
    let resolver = file.resolver();
    let mut text_content = String::new();

    for page_num in 0..file.num_pages() {
        if let Ok(page) = file.get_page(page_num) {
            if let Some(content) = &page.contents {
                if let Ok(ops) = content.operations(&resolver) {
                    for op in ops {
                        match op {
                            Op::TextDraw { text } => {
                                if let Ok(s) = text.to_string() {
                                    text_content.push_str(&s);
                                }
                            }
                            Op::TextDrawAdjusted { array } => {
                                for item in array {
                                    if let TextDrawAdjusted::Text(text) = item {
                                        if let Ok(s) = text.to_string() {
                                            text_content.push_str(&s);
                                        }
                                    }
                                }
                            }
                            Op::EndText => {
                                text_content.push_str("\n");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        text_content.push_str("\n\n");
    }

    let cleaned: String = text_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(cleaned)
}

async fn parse_docx(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let docx = docx_rs::read_docx(&buf)?;
    let mut text_content = String::new();

    // 遍历文档中的所有子元素
    for child in &docx.document.children {
        extract_text_from_document_child(child, &mut text_content);
    }

    // 清理多余空白
    let cleaned: String = text_content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Ok(cleaned)
}

/// 从 DocumentChild 中提取文本
fn extract_text_from_document_child(child: &DocumentChild, output: &mut String) {
    match child {
        DocumentChild::Paragraph(p) => {
            let mut para_text = String::new();
            for p_child in &p.children {
                extract_text_from_paragraph_child(p_child, &mut para_text);
            }
            if !para_text.is_empty() {
                output.push_str(&para_text);
            }
            output.push('\n');
        }
        DocumentChild::Table(table) => {
            for row in &table.rows {
                let TableChild::TableRow(tr) = row;
                let mut row_texts = Vec::new();
                for cell in &tr.cells {
                    let TableRowChild::TableCell(tc) = cell;
                    let mut cell_text = String::new();
                    for tc_child in &tc.children {
                        if let TableCellContent::Paragraph(p) = tc_child {
                            for p_child in &p.children {
                                extract_text_from_paragraph_child(p_child, &mut cell_text);
                            }
                        }
                    }
                    row_texts.push(cell_text.trim().to_string());
                }
                output.push_str(&row_texts.join("\t"));
                output.push('\n');
            }
            output.push('\n');
        }
        _ => {}
    }
}

/// 从 ParagraphChild 中提取文本
fn extract_text_from_paragraph_child(child: &ParagraphChild, output: &mut String) {
    match child {
        ParagraphChild::Run(run) => {
            for run_child in &run.children {
                match run_child {
                    RunChild::Text(t) => {
                        output.push_str(&t.text);
                    }
                    RunChild::Tab(_) => {
                        output.push('\t');
                    }
                    RunChild::Break(_) => {
                        output.push('\n');
                    }
                    _ => {}
                }
            }
        }
        ParagraphChild::Hyperlink(link) => {
            for link_child in &link.children {
                extract_text_from_paragraph_child(link_child, output);
            }
        }
        _ => {}
    }
}


async fn parse_pptx(path: &Path) -> Result<String> {
    let config = ParserConfig::builder()
        .extract_images(false)
        .include_slide_comment(false)
        .build();

    let mut pptx_container = PptxContainer::open(path, config)?;
    let slides = pptx_container.parse_all()?;

    let mut text_content = String::new();

    for (i, slide) in slides.iter().enumerate() {

        text_content.push_str(&format!("--- Slide {} ---\n", i + 1));


        if let Some(md_content) = slide.convert_to_md() {

            let plain_text = strip_markdown(&md_content);
            text_content.push_str(&plain_text);
        }

        text_content.push_str("\n\n");
    }

    let cleaned: String = text_content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Ok(cleaned)
}


fn strip_markdown(md: &str) -> String {
    let mut result = String::new();

    for line in md.lines() {
        let line = line.trim();

        if line.starts_with("![") {
            continue;
        }

        let line = line.trim_start_matches('#').trim();

        let line = line.replace("**", "").replace("__", "");
        let line = line.replace("*", "").replace("_", "");

        let line =
            if line.starts_with("- ") || line.starts_with("* ") {
                &line[2..]
            } else if line.chars().next().map_or(false, |c| c.is_ascii_digit())
                && line.contains(". ")
            {
                line.split_once(". ").map_or(&line[..], |(_, rest)| rest)
            } else {
                &line[..]
            };

        if !line.is_empty() {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}


async fn parse_xlsx(path: &Path) -> Result<String> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let mut text_content = String::new();


    let sheet_names = workbook.sheet_names().to_owned();

    for sheet_name in sheet_names {

        text_content.push_str(&format!("--- Sheet: {} ---\n", sheet_name));

        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows() {
                let row_text: Vec<String> = row
                    .iter()
                    .map(|cell| cell_to_string(cell))
                    .collect();


                if row_text.iter().all(|s| s.is_empty()) {
                    continue;
                }


                text_content.push_str(&row_text.join("\t"));
                text_content.push('\n');
            }
        }

        text_content.push('\n');
    }


    let cleaned: String = text_content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Ok(cleaned)
}


fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // 如果是整数，不显示小数点
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Data::DateTime(dt) => {
            // 使用 ExcelDateTime 的 as_datetime 方法
            if let Some(datetime) = dt.as_datetime() {
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                format!("{:?}", dt)
            }
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERR:{:?}", e),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(FileType::from_extension("txt"), Some(FileType::TXT));
        assert_eq!(FileType::from_extension("PDF"), Some(FileType::PDF));
        assert_eq!(FileType::from_extension("docx"), Some(FileType::DOCX));
        assert_eq!(FileType::from_extension("PPTX"), Some(FileType::PPTX));
        assert_eq!(FileType::from_extension("XLSX"), Some(FileType::XLSX));
        assert_eq!(FileType::from_extension("jpg"), None);
    }

    #[test]
    fn test_cell_to_string() {
        assert_eq!(cell_to_string(&Data::Empty), "");
        assert_eq!(cell_to_string(&Data::String("hello".to_string())), "hello");
        assert_eq!(cell_to_string(&Data::Int(42)), "42");
        assert_eq!(cell_to_string(&Data::Float(3.14)), "3.14");
        assert_eq!(cell_to_string(&Data::Float(100.0)), "100");
        assert_eq!(cell_to_string(&Data::Bool(true)), "TRUE");
        assert_eq!(cell_to_string(&Data::Bool(false)), "FALSE");
    }
}