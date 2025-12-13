use anyhow::Result;
use docx_rs::{
    DocumentChild, ParagraphChild, RunChild, TableCellContent, TableChild, TableRowChild,
};
use pdf::{content::*, file::FileOptions};
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type FileCache = Arc<RwLock<HashMap<String, CacheFile>>>;

#[derive(Clone)]
pub struct CacheFile {
    pub filename: String,
    pub content: String,
}

pub fn new_file_cache() -> FileCache {
    Arc::new(RwLock::new(HashMap::new()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    TXT,
    PDF,
    DOCX,
}

impl FileType {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "txt" => Some(FileType::TXT),
            "pdf" => Some(FileType::PDF),
            "docx" => Some(FileType::DOCX),
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
        FileType::TXT => parse_txt(&temp_file).await,
        FileType::PDF => parse_pdf(&temp_file).await,
        FileType::DOCX => parse_docx(&temp_file).await,
    };

    let _ = tokio::fs::remove_file(&temp_file).await;

    result
}

async fn parse_txt(path: &Path) -> Result<String> {
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
