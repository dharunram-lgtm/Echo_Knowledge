use std::collections::HashSet;

/// Blacklist of generic words that must never be created as standalone tags
pub fn is_blacklisted_tag(raw: &str) -> bool {
    let lower = raw.trim().to_lowercase();
    if lower.len() < 2 || lower.len() > 50 {
        return true;
    }

    let blacklist: HashSet<&'static str> = [
        "document",
        "documents",
        "note",
        "notes",
        "info",
        "information",
        "file",
        "files",
        "text",
        "texts",
        "page",
        "pages",
        "data",
        "summary",
        "summaries",
        "content",
        "contents",
        "overview",
        "introduction",
        "chapter",
        "section",
        "sections",
        "untitled",
        "sample",
        "test",
        "example",
        "examples",
        "guide",
        "misc",
        "miscellaneous",
        "general",
        "pdf",
        "txt",
        "markdown",
        "doc",
    ]
    .into_iter()
    .collect();

    blacklist.contains(lower.as_str())
}

/// Normalizes raw tag or concept names into canonical `(display_name, canonical_id)`
pub fn normalize_tag_name(raw: &str) -> (String, String) {
    let trimmed = raw
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'' || c == '`' || c == '*' || c == '#');

    let cleaned_lower = trimmed
        .to_lowercase()
        .replace('_', " ")
        .replace('-', " ");
    let single_spaced = cleaned_lower
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    // 1. Alias Dictionary for common technical terms, data structures, and algorithms
    match single_spaced.as_str() {
        "dp" | "dynamic programming" => (
            "Dynamic Programming".to_string(),
            "dynamic-programming".to_string(),
        ),
        "linkedlist" | "linked lists" | "linked list" | "singly linked list" | "doubly linked list" => (
            "Linked Lists".to_string(),
            "linked-lists".to_string(),
        ),
        "greedy" | "greedy algorithm" | "greedy algorithms" => (
            "Greedy Algorithms".to_string(),
            "greedy-algorithms".to_string(),
        ),
        "binary search" | "binarysearch" | "bs" => (
            "Binary Search".to_string(),
            "binary-search".to_string(),
        ),
        "bst" | "binary search tree" => (
            "Binary Search Tree".to_string(),
            "binary-search-tree".to_string(),
        ),
        "dsa" | "data structures and algorithms" | "data structures & algorithms" => (
            "Data Structures & Algorithms".to_string(),
            "data-structures-algorithms".to_string(),
        ),
        "graph" | "graphs" | "graph algorithms" => (
            "Graph Algorithms".to_string(),
            "graph-algorithms".to_string(),
        ),
        "tree" | "trees" => (
            "Trees".to_string(),
            "trees".to_string(),
        ),
        "recursion" | "recursive algorithms" => (
            "Recursion".to_string(),
            "recursion".to_string(),
        ),
        "sorting" | "sorting algorithms" => (
            "Sorting Algorithms".to_string(),
            "sorting-algorithms".to_string(),
        ),
        "bfs" | "breadth first search" | "breadth-first search" => (
            "Breadth-First Search".to_string(),
            "breadth-first-search".to_string(),
        ),
        "dfs" | "depth first search" | "depth-first search" => (
            "Depth-First Search".to_string(),
            "depth-first-search".to_string(),
        ),
        "js" | "javascript" => ("JavaScript".to_string(), "javascript".to_string()),
        "ts" | "typescript" => ("TypeScript".to_string(), "typescript".to_string()),
        "py" | "python" => ("Python".to_string(), "python".to_string()),
        "rs" | "rust" | "rustlang" => ("Rust".to_string(), "rust".to_string()),
        "java" => ("Java".to_string(), "java".to_string()),
        "cpp" | "c++" => ("C++".to_string(), "cpp".to_string()),
        "csharp" | "c#" => ("C#".to_string(), "csharp".to_string()),
        "sql" | "sqlite" => ("SQLite".to_string(), "sqlite".to_string()),
        "rag" | "retrieval augmented generation" | "retrieval-augmented generation" => (
            "Retrieval-Augmented Generation".to_string(),
            "retrieval-augmented-generation".to_string(),
        ),
        "ai" | "artificial intelligence" => (
            "Artificial Intelligence".to_string(),
            "artificial-intelligence".to_string(),
        ),
        "ml" | "machine learning" => (
            "Machine Learning".to_string(),
            "machine-learning".to_string(),
        ),
        "nlp" | "natural language processing" => (
            "Natural Language Processing".to_string(),
            "natural-language-processing".to_string(),
        ),
        "llm" | "llms" | "large language models" | "large language model" => (
            "Large Language Models".to_string(),
            "large-language-models".to_string(),
        ),
        "os" | "operating system" | "operating systems" => (
            "Operating Systems".to_string(),
            "operating-systems".to_string(),
        ),
        "db" | "database" | "databases" => (
            "Databases".to_string(),
            "databases".to_string(),
        ),
        "tauri" | "tauri 2" => ("Tauri".to_string(), "tauri".to_string()),
        "react" | "react 19" | "reactjs" => ("React".to_string(), "react".to_string()),
        "vector search" | "vector embeddings" | "embeddings" => (
            "Vector Embeddings".to_string(),
            "vector-embeddings".to_string(),
        ),
        _ => {
            // 2. Generic Title Case & Canonical Slugifier
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            let mut title_cased = Vec::new();
            let mut slug_parts = Vec::new();

            for word in words {
                let clean_word: String = word
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '+' || *c == '#')
                    .collect();

                if clean_word.is_empty() {
                    continue;
                }

                // Preserve all-caps acronyms like SQL, API, HTML, CSS, CPU
                let word_upper = clean_word.to_uppercase();
                let is_acronym = (clean_word.len() <= 4 && clean_word == word_upper)
                    || ["API", "SDK", "CLI", "JSON", "HTTP", "REST", "URL", "UUID", "CRUD"].contains(&clean_word.as_str());

                if is_acronym {
                    title_cased.push(clean_word.to_uppercase());
                } else {
                    let mut chars = clean_word.chars();
                    let first = chars.next().unwrap_or(' ').to_uppercase().to_string();
                    let rest: String = chars.collect::<String>().to_lowercase();
                    title_cased.push(format!("{}{}", first, rest));
                }

                slug_parts.push(clean_word.to_lowercase());
            }

            let display_name = if title_cased.is_empty() {
                trimmed.to_string()
            } else {
                title_cased.join(" ")
            };

            let canonical_id = if slug_parts.is_empty() {
                trimmed.to_lowercase().replace(' ', "-")
            } else {
                slug_parts.join("-")
            };

            (display_name, canonical_id)
        }
    }
}

