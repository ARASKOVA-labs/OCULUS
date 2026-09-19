use crate::models::{OcrBlock, OcrResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TableResult {
    pub rows: Vec<Vec<String>>,
    pub column_count: usize,
    pub row_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StructuredDocument {
    pub document_id: String,
    pub language: String,
    pub fields: HashMap<String, String>,
    pub tables: Vec<TableResult>,
    pub raw_text: String,
}

pub struct DocumentIntelligence;

impl DocumentIntelligence {
    /// Deterministic spatial table extraction based on horizontal/vertical bounding box alignments.
    pub fn extract_tables(blocks: &[OcrBlock]) -> Vec<TableResult> {
        if blocks.is_empty() {
            return Vec::new();
        }

        // 1. Sort blocks by y coordinate (rows), then x coordinate (columns)
        let mut sorted_blocks = blocks.to_vec();
        sorted_blocks.sort_by(|a, b| {
            if (a.bbox.y - b.bbox.y).abs() < 12 {
                a.bbox.x.cmp(&b.bbox.x)
            } else {
                a.bbox.y.cmp(&b.bbox.y)
            }
        });

        // 2. Group into lines/rows based on vertical proximity threshold
        let mut rows: Vec<Vec<OcrBlock>> = Vec::new();
        for block in sorted_blocks {
            if let Some(last_row) = rows.last_mut() {
                let row_y = last_row[0].bbox.y;
                if (block.bbox.y - row_y).abs() < 14 {
                    last_row.push(block);
                    continue;
                }
            }
            rows.push(vec![block]);
        }

        // 3. Detect multi-column rows (candidates for table structure)
        let multi_col_rows: Vec<Vec<String>> = rows
            .into_iter()
            .filter(|r| r.len() >= 2) // At least 2 columns
            .map(|row| row.into_iter().map(|b| b.text.trim().to_string()).collect())
            .collect();

        if multi_col_rows.len() >= 2 {
            let col_count = multi_col_rows.iter().map(|r| r.len()).max().unwrap_or(0);
            let row_count = multi_col_rows.len();
            vec![TableResult {
                rows: multi_col_rows,
                column_count: col_count,
                row_count,
            }]
        } else {
            Vec::new()
        }
    }

    /// Deterministic Key-Value field extraction with English and Malayalam pattern support.
    pub fn extract_fields(full_text: &str) -> HashMap<String, String> {
        let mut fields = HashMap::new();

        // Common key-value separator patterns: "Key: Value", "Key：Value", "Key = Value", "Key - Value"
        for line in full_text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let pair = if let Some((k, v)) = line.split_once(':') {
                Some((k, v))
            } else if let Some((k, v)) = line.split_once('\u{FF1A}') {
                Some((k, v))
            } else if let Some((k, v)) = line.split_once('=') {
                Some((k, v))
            } else if let Some((k, v)) = line.split_once(" - ") {
                Some((k, v))
            } else {
                None
            };

            if let Some((k, v)) = pair {
                let key = k.trim();
                let val = v.trim();
                if !key.is_empty() && !val.is_empty() && key.len() < 50 {
                    fields.insert(key.to_string(), val.to_string());
                }
            }
        }

        fields
    }

    /// Extract spatial key-value pairs from adjacent bounding box blocks
    pub fn extract_spatial_fields(blocks: &[OcrBlock]) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        if blocks.len() < 2 {
            return fields;
        }

        for (i, b1) in blocks.iter().enumerate() {
            let t1 = b1.text.trim();
            if t1.ends_with(':') || t1.ends_with('\u{FF1A}') {
                let key = t1.trim_end_matches([':', '\u{FF1A}']).trim();
                if key.is_empty() || key.len() > 50 {
                    continue;
                }

                // Look for horizontally adjacent block to the right on the same line
                for (j, b2) in blocks.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let y_diff = (b1.bbox.y - b2.bbox.y).abs();
                    let x_dist = b2.bbox.x - (b1.bbox.x + b1.bbox.width);

                    if y_diff < 14 && (0..150).contains(&x_dist) {
                        let val = b2.text.trim();
                        if !val.is_empty() {
                            fields.insert(key.to_string(), val.to_string());
                            break;
                        }
                    }
                }
            }
        }

        fields
    }

    /// Process complete OCR result into structured intelligence document.
    pub fn analyze(result: &OcrResult) -> StructuredDocument {
        let mut all_blocks = Vec::new();
        let mut text_parts = Vec::new();

        for page in &result.pages {
            all_blocks.extend(page.blocks.clone());
            text_parts.push(page.text.clone());
        }

        let full_text = text_parts.join("\n\n");
        let tables = Self::extract_tables(&all_blocks);
        let mut fields = Self::extract_fields(&full_text);

        // Supplement with spatial key-value pairs from bounding boxes
        let spatial_fields = Self::extract_spatial_fields(&all_blocks);
        for (k, v) in spatial_fields {
            fields.entry(k).or_insert(v);
        }

        StructuredDocument {
            document_id: result.document_id.clone(),
            language: result.language.clone(),
            fields,
            tables,
            raw_text: full_text,
        }
    }
}
