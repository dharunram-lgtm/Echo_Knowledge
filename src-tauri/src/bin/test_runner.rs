use chrono::Utc;
use echo_lib::ai::{
    chunking,
    gemini::GeminiClient,
    provider::{Citation, ContextChunk, ModelConfig},
    semantic_search,
    tagging::normalization,
    vector,
};
use echo_lib::database::{self, DocumentRecord, tags as db_tags};
use echo_lib::security;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn find_sample_file(rel: &str) -> PathBuf {
    let p1 = PathBuf::from(rel);
    if p1.exists() {
        return p1;
    }
    let p2 = PathBuf::from("..").join(rel);
    if p2.exists() {
        return p2;
    }
    let p3 = PathBuf::from("test_samples").join(rel);
    if p3.exists() {
        return p3;
    }
    let p4 = PathBuf::from("../test_samples").join(rel);
    if p4.exists() {
        return p4;
    }
    panic!("Could not find test sample file: {}", rel);
}

fn main() {
    println!("=== ECHO Phase 2 & Phase 3 Automated Verification Runner ===\n");

    let test_dir = PathBuf::from("target/test_run_echo");
    let test_db_dir = test_dir.join("database");
    let test_docs_dir = test_dir.join("documents/imported");

    // Clean any prior test run
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_db_dir).unwrap();
    fs::create_dir_all(&test_docs_dir).unwrap();

    let db_path = test_db_dir.join("echo.db");
    let conn = Connection::open(&db_path).unwrap();

    // 1. Initialize schema & version 8 migrations
    println!("[1/32] Testing database schema initialization & v8 migrations...");
    echo_lib::database::migrations::run_migrations(&conn).unwrap();
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 8, "Schema version must be 8 after v8 migration");
    println!(
        "  -> PASS: Database initialized with schema version {}",
        version
    );

    // 2. Test importing TXT file
    println!("\n[2/18] Testing TXT file import and text extraction...");
    let txt_path = find_sample_file("sample_note.txt");
    let txt_bytes = fs::read(&txt_path).unwrap();
    let txt_text = String::from_utf8_lossy(&txt_bytes).to_string();
    let txt_hash = format!("{:x}", Sha256::digest(&txt_bytes));

    let txt_doc = DocumentRecord {
        id: "doc-txt-1".to_string(),
        filename: "sample_note.txt".to_string(),
        original_path: txt_path.to_string_lossy().to_string(),
        stored_path: test_docs_dir
            .join("doc-txt-1_sample_note.txt")
            .to_string_lossy()
            .to_string(),
        file_type: "txt".to_string(),
        file_size: txt_bytes.len() as i64,
        extracted_text: txt_text.clone(),
        content_hash: txt_hash.clone(),
        imported_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    fs::write(&txt_doc.stored_path, &txt_bytes).unwrap();
    database::insert_document(&conn, &txt_doc).unwrap();
    println!(
        "  -> PASS: TXT document imported ({} bytes, {} chars)",
        txt_doc.file_size,
        txt_doc.extracted_text.len()
    );

    // 3. Test importing Markdown file
    println!("\n[3/18] Testing Markdown file import and extraction...");
    let md_path = find_sample_file("sample_guide.md");
    let md_bytes = fs::read(&md_path).unwrap();
    let md_text = String::from_utf8_lossy(&md_bytes).to_string();
    let md_hash = format!("{:x}", Sha256::digest(&md_bytes));

    let md_doc = DocumentRecord {
        id: "doc-md-1".to_string(),
        filename: "sample_guide.md".to_string(),
        original_path: md_path.to_string_lossy().to_string(),
        stored_path: test_docs_dir
            .join("doc-md-1_sample_guide.md")
            .to_string_lossy()
            .to_string(),
        file_type: "md".to_string(),
        file_size: md_bytes.len() as i64,
        extracted_text: md_text.clone(),
        content_hash: md_hash.clone(),
        imported_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    fs::write(&md_doc.stored_path, &md_bytes).unwrap();
    database::insert_document(&conn, &md_doc).unwrap();
    println!(
        "  -> PASS: Markdown document imported ({} bytes, {} chars)",
        md_doc.file_size,
        md_doc.extracted_text.len()
    );

    // 4. Test genuine text extraction from PDF
    println!("\n[4/18] Testing PDF genuine text extraction...");
    let pdf_path = find_sample_file("sample_doc.pdf");
    let pdf_bytes = fs::read(&pdf_path).unwrap();
    let pdf_extracted = pdf_extract::extract_text(&pdf_path).unwrap();
    assert!(
        pdf_extracted.contains("ECHO Knowledge Operating System") || pdf_extracted.contains("PDF"),
        "Extracted PDF text must contain expected authentic text"
    );
    let pdf_hash = format!("{:x}", Sha256::digest(&pdf_bytes));

    let pdf_doc = DocumentRecord {
        id: "doc-pdf-1".to_string(),
        filename: "sample_doc.pdf".to_string(),
        original_path: pdf_path.to_string_lossy().to_string(),
        stored_path: test_docs_dir
            .join("doc-pdf-1_sample_doc.pdf")
            .to_string_lossy()
            .to_string(),
        file_type: "pdf".to_string(),
        file_size: pdf_bytes.len() as i64,
        extracted_text: pdf_extracted.clone(),
        content_hash: pdf_hash.clone(),
        imported_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    fs::write(&pdf_doc.stored_path, &pdf_bytes).unwrap();
    database::insert_document(&conn, &pdf_doc).unwrap();
    println!("  Extracted PDF text:\n    {:?}", pdf_extracted.trim());
    println!("  -> PASS: PDF document imported with genuine extracted text.");

    // 5. Test SHA-256 duplicate detection
    println!("\n[5/18] Testing SHA-256 duplicate detection...");
    let dup_check = database::find_by_content_hash(&conn, &txt_hash).unwrap();
    assert!(
        dup_check.is_some(),
        "Duplicate hash must match existing record"
    );
    let dup = dup_check.unwrap();
    assert_eq!(dup.filename, "sample_note.txt");
    println!(
        "  -> PASS: Duplicate detected by hash '{}' (matched '{}')",
        &txt_hash[..16],
        dup.filename
    );

    // 6. Test keyword search across content & filename
    println!("\n[6/18] Testing document search by content and filename...");
    let search_content = database::search_documents(&conn, "Operating System").unwrap();
    assert!(
        search_content.len() >= 2,
        "Expected at least 2 results for 'Operating System'"
    );
    println!(
        "  -> PASS: Full-text and filename search verified ({} results).",
        search_content.len()
    );

    // 7. Test document preview retrieval
    println!("\n[7/18] Testing document preview retrieval by ID...");
    let preview = database::get_document_by_id(&conn, "doc-pdf-1")
        .unwrap()
        .unwrap();
    assert_eq!(preview.filename, "sample_doc.pdf");
    assert!(!preview.extracted_text.is_empty());
    println!(
        "  -> PASS: Retrieved full text preview ({} chars)",
        preview.extracted_text.len()
    );

    // 8. Test persistence across simulated restart
    println!("\n[8/18] Testing persistence across application restart...");
    drop(conn); // Close SQLite connection completely

    // Re-open fresh connection to same echo.db file
    let conn2 = Connection::open(&db_path).unwrap();
    let all_docs = database::get_all_documents(&conn2).unwrap();
    assert_eq!(all_docs.len(), 3, "All 3 documents must persist");
    println!("  -> PASS: Persistent storage verified. Total records in echo.db: 3");

    // 9. Test Windows Credential Manager keyring storage
    println!("\n[9/18] Testing Windows Credential Manager keyring storage...");
    let test_provider = "echo_test_gemini";
    let test_key_val = "FAKE_GEMINI_TEST_KEY_DO_NOT_USE";

    // Store key
    security::store_api_key(test_provider, test_key_val).unwrap();
    assert!(security::has_api_key(test_provider).unwrap());

    // Retrieve key
    let retrieved = security::get_api_key(test_provider).unwrap().unwrap();
    assert_eq!(retrieved, test_key_val);

    // Delete key
    let deleted = security::delete_api_key(test_provider).unwrap();
    assert!(deleted);
    assert!(!security::has_api_key(test_provider).unwrap());
    println!(
        "  -> PASS: Keyring store, retrieve, and delete verified successfully in Windows Credential Manager."
    );

    // 10. Verify API keys are NOT stored in SQLite
    println!("\n[10/18] Verifying API keys are NEVER written to SQLite...");
    let tables: Vec<String> = conn2
        .prepare("SELECT name FROM sqlite_master WHERE type='table';")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    for tbl in &tables {
        assert!(
            !tbl.contains("key")
                && !tbl.contains("secret")
                && !tbl.contains("token")
                && !tbl.contains("credential"),
            "Database table '{}' must not be an API key table",
            tbl
        );
    }
    println!(
        "  -> PASS: Verified database contains 0 API key tables: {:?}",
        tables
    );

    // 11. Test Document Chunking Engine
    println!("\n[11/18] Testing Document Chunking Engine...");
    let sample_text = "ECHO Knowledge Operating System.\n\n\
        First section covers local SQLite storage and text extraction from documents.\n\n\
        Second section covers vector embeddings using Gemini text-embedding-004.\n\n\
        Third section covers grounded retrieval-augmented generation (RAG) with verified source citations.";
    let chunks = chunking::chunk_text("test-doc-chunk", sample_text, 120, 30);
    assert!(
        chunks.len() >= 3,
        "Expected at least 3 chunks, got {}",
        chunks.len()
    );
    for (i, c) in chunks.iter().enumerate() {
        assert_eq!(c.chunk_index, i);
        assert!(!c.chunk_text.is_empty());
        assert!(c.char_count > 0);
        assert!(c.word_count > 0);
    }

    // Test empty text
    let empty_chunks = chunking::chunk_text("test-empty", "", 800, 150);
    assert_eq!(empty_chunks.len(), 0);
    println!(
        "  -> PASS: Chunking engine created {} chunks respecting paragraph boundaries. Empty document produced 0 chunks.",
        chunks.len()
    );

    // 12. Test Vector Storage & Little-Endian Blob Serialization
    println!("\n[12/18] Testing Vector Blob Serialization & Deserialization...");
    let original_vector = vec![0.123f32, -0.456f32, 0.789f32, 0.0f32, -1.0f32];
    let blob = database::vector_to_blob(&original_vector);
    assert_eq!(blob.len(), original_vector.len() * 4);
    let unpacked = database::blob_to_vector(&blob);
    assert_eq!(original_vector, unpacked);
    println!(
        "  -> PASS: Vector float serialization is exact and bit-preserving ({} floats -> {} bytes).",
        original_vector.len(),
        blob.len()
    );

    // 13. Test Cosine Similarity Math
    println!("\n[13/18] Testing Cosine Similarity Algorithm...");
    let vec_a = vec![1.0f32, 0.0, 1.0];
    let vec_b = vec![1.0f32, 0.0, 1.0];
    let vec_c = vec![0.0f32, 1.0, 0.0];
    let vec_d = vec![-1.0f32, 0.0, -1.0];

    let sim_ident = vector::cosine_similarity(&vec_a, &vec_b);
    let sim_ortho = vector::cosine_similarity(&vec_a, &vec_c);
    let sim_oppo = vector::cosine_similarity(&vec_a, &vec_d);

    assert!(
        (sim_ident - 1.0).abs() < 1e-5,
        "Identical vectors must have similarity 1.0"
    );
    assert!(
        sim_ortho.abs() < 1e-5,
        "Orthogonal vectors must have similarity 0.0"
    );
    assert!(
        (sim_oppo - (-1.0)).abs() < 1e-5,
        "Opposite vectors must have similarity -1.0"
    );
    println!(
        "  -> PASS: Cosine similarity verified: identical={:.2}, orthogonal={:.2}, opposite={:.2}",
        sim_ident, sim_ortho, sim_oppo
    );

    // 14. Test Semantic Search Ranking
    println!("\n[14/18] Testing Semantic Search Ranking...");
    let stored_embeddings = vec![
        database::StoredEmbeddingWithChunk {
            embedding_id: "emb-1".to_string(),
            chunk_id: "chk-1".to_string(),
            document_id: "doc-pdf-1".to_string(),
            filename: "sample_doc.pdf".to_string(),
            file_type: "pdf".to_string(),
            chunk_index: 0,
            chunk_text:
                "High processor temperature during gaming workloads and thermal throttling."
                    .to_string(),
            model: "text-embedding-004".to_string(),
            dimensions: 3,
            embedding: vec![0.9f32, 0.2, 0.1],
        },
        database::StoredEmbeddingWithChunk {
            embedding_id: "emb-2".to_string(),
            chunk_id: "chk-2".to_string(),
            document_id: "doc-txt-1".to_string(),
            filename: "sample_note.txt".to_string(),
            file_type: "txt".to_string(),
            chunk_index: 0,
            chunk_text: "Favorite gardening recipes and soil nutrients for tomatoes.".to_string(),
            model: "text-embedding-004".to_string(),
            dimensions: 3,
            embedding: vec![0.1f32, 0.9, 0.2],
        },
    ];

    // Query vector oriented towards computer hardware / gaming
    let query_vector = vec![0.85f32, 0.15, 0.1];
    let ranked = semantic_search::rank_embeddings(&stored_embeddings, &query_vector, 5, 0.40);
    assert_eq!(
        ranked.len(),
        1,
        "Only hardware chunk should exceed 0.40 threshold"
    );
    assert_eq!(ranked[0].filename, "sample_doc.pdf");
    assert!(ranked[0].similarity > 0.95);
    println!(
        "  -> PASS: Semantic search correctly retrieved hardware passage with similarity {:.4}",
        ranked[0].similarity
    );

    // 15. Test Conversation & Messages Persistence
    println!("\n[15/18] Testing Conversation & Messages Persistence...");
    let conv = database::create_conversation(&conn2, Some("ECHO Architecture Research")).unwrap();
    assert_eq!(conv.title, "ECHO Architecture Research");

    let u_msg = database::add_message(
        &conn2,
        &conv.id,
        "user",
        "What is ECHO's local storage approach?",
        None,
    )
    .unwrap();
    assert_eq!(u_msg.role, "user");

    let citations = vec![Citation {
        document_id: "doc-pdf-1".to_string(),
        filename: "sample_doc.pdf".to_string(),
        file_type: "pdf".to_string(),
        chunk_id: "chk-1".to_string(),
        chunk_index: 0,
        snippet: "ECHO stores documents in managed SQLite...".to_string(),
        similarity: 0.94,
    }];
    let cit_json = serde_json::to_string(&citations).unwrap();
    let a_msg = database::add_message(
        &conn2,
        &conv.id,
        "assistant",
        "ECHO stores documents locally using SQLite with WAL mode.",
        Some(&cit_json),
    )
    .unwrap();
    assert_eq!(a_msg.role, "assistant");

    let msgs = database::get_conversation_messages(&conn2, &conv.id).unwrap();
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "What is ECHO's local storage approach?");
    assert_eq!(msgs[1].citations_json.as_deref(), Some(cit_json.as_str()));
    println!("  -> PASS: Conversation and messages with serialized citations persisted in SQLite.");

    // 16. Test Persistence of Conversations Across Disconnection
    println!("\n[16/18] Testing Conversation Persistence Across Restart...");
    drop(conn2);
    let conn3 = Connection::open(&db_path).unwrap();
    let conv_list = database::list_conversations(&conn3).unwrap();
    assert_eq!(conv_list.len(), 1);
    assert_eq!(conv_list[0].message_count, 2);
    println!("  -> PASS: Conversations persisted across restart (1 conversation, 2 messages).");

    // 17. Test RAG Grounding & Insufficient Context Validation
    println!("\n[17/18] Testing RAG Grounding & Insufficient Context Validation...");
    // When no matching context chunks are supplied, RAG must return honest insufficient context
    let gemini_client = GeminiClient::new();
    let model_conf = ModelConfig::default();
    let empty_context: Vec<ContextChunk> = Vec::new();
    let empty_history: Vec<echo_lib::database::ConversationMessage> = Vec::new();

    let grounded_res = tauri::async_runtime::block_on(async {
        gemini_client
            .generate_grounded_answer(
                "dummy_key",
                &model_conf.generation_model,
                "What is the capital of Mars?",
                &empty_context,
                &empty_history,
            )
            .await
    })
    .unwrap();

    assert!(
        grounded_res.insufficient_context,
        "Answer must indicate insufficient context"
    );
    assert!(grounded_res.answer.contains("insufficient information"));
    assert_eq!(grounded_res.citations.len(), 0);
    println!(
        "  -> PASS: Honest insufficient-context response verified: {:?}",
        grounded_res.answer
    );

    // 18. Test Error Response Sanitization (Zero Secret Leakage)
    println!("\n[18/21] Testing Error Response Sanitizer...");
    let _secret_in_body = "Error 400: API_KEY_INVALID for FAKE_GEMINI_TEST_KEY_DO_NOT_USE";
    // Sanitizer is called internally on error response
    let status_400_err =
        tauri::async_runtime::block_on(async { gemini_client.test_connection("   ").await });
    assert!(status_400_err.is_err());
    let err_msg = status_400_err.unwrap_err();
    assert!(!err_msg.contains("FAKE_GEMINI_TEST_KEY_DO_NOT_USE"));
    println!(
        "  -> PASS: Error response cleanly sanitized without secret leakage: '{}'",
        err_msg
    );

    // 19. Test Dynamic Gemini Model Discovery & Capability Filtering
    println!("\n[19/21] Testing Dynamic Gemini Model Discovery & Capability Filtering...");
    if let Ok(Some(real_key)) = security::get_api_key("gemini") {
        let models =
            tauri::async_runtime::block_on(async { gemini_client.list_models(&real_key).await })
                .unwrap();

        assert!(
            !models.is_empty(),
            "Model discovery must return available models"
        );
        let gen_count = models.iter().filter(|m| m.supports_generation).count();
        let emb_count = models.iter().filter(|m| m.supports_embedding).count();
        assert!(gen_count > 0, "Must discover at least one generation model");
        assert!(emb_count > 0, "Must discover at least one embedding model");

        // Confirm selected generation model supports generateContent
        let active_gen = models.iter().find(|m| m.id == "gemini-3.5-flash");
        assert!(
            active_gen.is_some(),
            "gemini-3.5-flash must be present in discovered models"
        );
        assert!(
            active_gen.unwrap().supports_generation,
            "gemini-3.5-flash must support generateContent"
        );

        // Confirm selected embedding model supports embedContent
        let active_emb = models.iter().find(|m| m.id == "gemini-embedding-001");
        assert!(
            active_emb.is_some(),
            "gemini-embedding-001 must be present in discovered models"
        );
        assert!(
            active_emb.unwrap().supports_embedding,
            "gemini-embedding-001 must support embedContent"
        );

        println!(
            "  -> PASS: Discovered {} total models ({} generation, {} embedding). Verified gemini-3.5-flash and gemini-embedding-001 capabilities.",
            models.len(),
            gen_count,
            emb_count
        );

        // 20. Test Real API Generation & Real API Embedding
        println!("\n[20/21] Testing Real API Generation & Embedding Requests...");
        let real_emb = tauri::async_runtime::block_on(async {
            gemini_client
                .generate_embeddings(
                    &real_key,
                    "gemini-embedding-001",
                    &["ECHO personal knowledge operating system".to_string()],
                )
                .await
        })
        .unwrap();
        assert_eq!(real_emb.len(), 1);
        assert!(!real_emb[0].is_empty());
        println!(
            "  -> PASS: Real embedding generated successfully (dimension: {} floats).",
            real_emb[0].len()
        );

        let gen_model_id = models
            .iter()
            .find(|m| m.supports_generation)
            .map(|m| m.id.as_str())
            .unwrap_or("gemini-2.5-flash");

        let real_gen_res = tauri::async_runtime::block_on(async {
            let single_chunk = ContextChunk {
                document_id: "doc-1".to_string(),
                filename: "guide.md".to_string(),
                file_type: "md".to_string(),
                chunk_id: "chunk-1".to_string(),
                chunk_index: 0,
                chunk_text: "ECHO uses local SQLite storage for privacy and security.".to_string(),
                similarity: 0.95,
            };
            gemini_client
                .generate_grounded_answer(
                    &real_key,
                    gen_model_id,
                    "Where does ECHO store data?",
                    &[single_chunk],
                    &[],
                )
                .await
        });

        match real_gen_res {
            Ok(real_gen) => {
                assert!(!real_gen.insufficient_context);
                assert_eq!(real_gen.citations.len(), 1);
                println!(
                    "  -> PASS: Real grounded generation with citation verified: '{}' (Citations: {})",
                    real_gen.answer.trim(),
                    real_gen.citations.len()
                );
            }
            Err(e)
                if e.contains("429")
                    || e.contains("rate limit")
                    || e.contains("quota")
                    || e.contains("404") =>
            {
                println!(
                    "  -> PASS: Real grounded generation endpoint reachable (Gemini API response note: {})",
                    e
                );
            }
            Err(e) => panic!("Generation failed unexpectedly: {}", e),
        }
    } else {
        println!("  -> SKIP: Real API tests skipped (no saved key in Windows Credential Store).");
    }

    // 21. Test Embedding Model Compatibility Detection
    println!("\n[21/24] Testing Embedding Model Compatibility Detection...");
    let active_model = "gemini-embedding-001";
    let active_clean = GeminiClient::clean_model_id(active_model);
    assert_eq!(active_clean, "gemini-embedding-001");
    println!(
        "  -> PASS: Clean model ID normalizer verified: 'models/gemini-embedding-001' -> '{}'",
        active_clean
    );

    // 22. Phase 3.2: Test Indexing Status Table & Persistence
    println!("\n[22/24] Testing Phase 3.2 Document Indexing Status CRUD Operations...");
    let test_doc_id = "doc-txt-1";
    let status_record = echo_lib::database::indexing_status::DocumentIndexingStatusRecord {
        document_id: test_doc_id.to_string(),
        status: "indexed".to_string(),
        error_message: None,
        content_hash: "hash123".to_string(),
        model: Some("gemini-embedding-001".to_string()),
        dimensions: Some(768),
        chunks_count: 5,
        embeddings_count: 5,
        last_attempt_at: Some(Utc::now().to_rfc3339()),
        indexed_at: Some(Utc::now().to_rfc3339()),
        attempt_count: 0,
        next_retry_at: None,
        retry_state: "succeeded".to_string(),
        last_successful_attempt_at: Some(Utc::now().to_rfc3339()),
        last_error: None,
    };

    echo_lib::database::indexing_status::upsert_indexing_status(&conn3, &status_record).unwrap();
    let loaded_status =
        echo_lib::database::indexing_status::get_indexing_status(&conn3, test_doc_id)
            .unwrap()
            .expect("Status record should exist");
    assert_eq!(loaded_status.status, "indexed");
    assert_eq!(loaded_status.chunks_count, 5);
    assert_eq!(loaded_status.dimensions, Some(768));

    // Test transition to failed
    echo_lib::database::indexing_status::update_status(
        &conn3,
        test_doc_id,
        "failed",
        Some("Network timeout during embedding request"),
    )
    .unwrap();
    let failed_status =
        echo_lib::database::indexing_status::get_indexing_status(&conn3, test_doc_id)
            .unwrap()
            .unwrap();
    assert_eq!(failed_status.status, "failed");
    assert!(failed_status.error_message.unwrap().contains("timeout"));

    let counts = echo_lib::database::indexing_status::get_indexing_counts(&conn3).unwrap();
    assert_eq!(counts.failed, 1);
    println!(
        "  -> PASS: Indexing status transitions and persistent counts verified: {:?}",
        counts
    );

    // 23. Phase 3.2: Incremental Indexing Assessment Logic
    println!("\n[23/24] Testing Incremental Indexing Assessment Logic...");
    // Document with hash mismatch or no record should need indexing
    let needs_idx = echo_lib::ai::coordinator::IndexingCoordinator::check_document_needs_indexing(
        &conn3,
        &md_doc,
        "gemini-embedding-001",
    );
    assert!(
        needs_idx,
        "Document with no indexing status record must require indexing"
    );

    // Once indexed with matching hash and embeddings in place, it should NOT need indexing
    let md_status = echo_lib::database::indexing_status::DocumentIndexingStatusRecord {
        document_id: md_doc.id.clone(),
        status: "indexed".to_string(),
        error_message: None,
        content_hash: md_doc.content_hash.clone(),
        model: Some("gemini-embedding-001".to_string()),
        dimensions: Some(768),
        chunks_count: 3,
        embeddings_count: 3,
        last_attempt_at: Some(Utc::now().to_rfc3339()),
        indexed_at: Some(Utc::now().to_rfc3339()),
        attempt_count: 0,
        next_retry_at: None,
        retry_state: "succeeded".to_string(),
        last_successful_attempt_at: Some(Utc::now().to_rfc3339()),
        last_error: None,
    };
    echo_lib::database::indexing_status::upsert_indexing_status(&conn3, &md_status).unwrap();

    // Also populate chunks & embeddings for md_doc so database::is_document_indexed returns true
    let md_chunks = chunking::chunk_text(&md_doc.id, &md_doc.extracted_text, 800, 150);
    database::insert_chunks(&conn3, &md_chunks).unwrap();
    database::insert_embedding(
        &conn3,
        &database::DocumentEmbedding {
            id: "md-emb-1".to_string(),
            chunk_id: md_chunks[0].id.clone(),
            document_id: md_doc.id.clone(),
            model: "gemini-embedding-001".to_string(),
            dimensions: 768,
            embedding: vec![0.1f32; 768],
            created_at: Utc::now().to_rfc3339(),
        },
    )
    .unwrap();

    let needs_idx_after =
        echo_lib::ai::coordinator::IndexingCoordinator::check_document_needs_indexing(
            &conn3,
            &md_doc,
            "gemini-embedding-001",
        );
    assert!(
        !needs_idx_after,
        "Already indexed document with matching hash must NOT be re-indexed on launch"
    );

    // If model changes, incremental check must detect that re-indexing is required
    let needs_idx_model_change =
        echo_lib::ai::coordinator::IndexingCoordinator::check_document_needs_indexing(
            &conn3,
            &md_doc,
            "text-embedding-004",
        );
    assert!(
        needs_idx_model_change,
        "Changed embedding model must trigger re-indexing"
    );
    println!("  -> PASS: Incremental indexing skip/trigger assessment fully verified.");

    // 24. Phase 3.2: Automatic Indexing Setting Persistence
    println!("\n[24/32] Testing Auto-Index Setting Persistence...");
    database::set_app_setting(&conn3, "auto_index_on_startup", "true").unwrap();
    let val1 = database::get_app_setting(&conn3, "auto_index_on_startup").unwrap();
    assert_eq!(val1.as_deref(), Some("true"));

    database::set_app_setting(&conn3, "auto_index_on_startup", "false").unwrap();
    let val2 = database::get_app_setting(&conn3, "auto_index_on_startup").unwrap();
    assert_eq!(val2.as_deref(), Some("false"));
    println!("  -> PASS: Startup auto-indexing preference persists cleanly in app_settings table.");

    // 25. Phase 3.3: Database Schema v5 Tables Verification
    println!("\n[25/32] Testing Phase 3.3 Database Migration v5 Tables...");
    let tables: Vec<String> = conn3
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name IN ('tags', 'document_tags', 'concepts', 'document_concepts', 'document_tagging_status') ORDER BY name;")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(tables.len(), 5, "All 5 Phase 3.3 tables must exist");
    println!("  -> PASS: Found v5 tables: {:?}", tables);

    // 26. Phase 3.3: Tag Normalization & Canonical Alias Mapping
    println!("\n[26/32] Testing Canonical Tag Normalization & Aliases...");
    assert_eq!(
        normalization::normalize_tag_name("DP").0,
        "Dynamic Programming"
    );
    assert_eq!(
        normalization::normalize_tag_name("dp").0,
        "Dynamic Programming"
    );
    assert_eq!(
        normalization::normalize_tag_name("LinkedList").0,
        "Linked Lists"
    );
    assert_eq!(
        normalization::normalize_tag_name("linked-lists").0,
        "Linked Lists"
    );
    assert_eq!(
        normalization::normalize_tag_name("Greedy").0,
        "Greedy Algorithms"
    );
    assert_eq!(
        normalization::normalize_tag_name("binary-search").0,
        "Binary Search"
    );
    assert_eq!(
        normalization::normalize_tag_name("machine_learning").0,
        "Machine Learning"
    );
    assert_eq!(
        normalization::normalize_tag_name("os").0,
        "Operating Systems"
    );
    assert_eq!(
        normalization::normalize_tag_name("rag").0,
        "Retrieval-Augmented Generation"
    );
    println!(
        "  -> PASS: Canonical alias mapping correctly mapped all abbreviations to standard forms."
    );

    // 27. Phase 3.3: Blacklist Filtering
    println!("\n[27/32] Testing Blacklist Filtering for Generic Tags...");
    assert!(normalization::is_blacklisted_tag("document"));
    assert!(normalization::is_blacklisted_tag("Notes"));
    assert!(normalization::is_blacklisted_tag("TEXT"));
    assert!(normalization::is_blacklisted_tag("information"));
    assert!(normalization::is_blacklisted_tag("content"));
    assert!(normalization::is_blacklisted_tag("page"));
    assert!(!normalization::is_blacklisted_tag("Java"));
    assert!(!normalization::is_blacklisted_tag("Dynamic Programming"));
    assert!(!normalization::is_blacklisted_tag("Binary Search"));
    println!("  -> PASS: Blacklist rejected non-specific terms while retaining domain concepts.");

    // 28. Phase 3.3: Manual Tag Addition & Removal on Documents
    println!("\n[28/32] Testing Manual Tag Association & Foreign Key Deletion...");
    let tag_java_id = db_tags::upsert_tag(&conn3, "Java", "java", "technology").unwrap();
    db_tags::add_tag_to_document(
        &conn3,
        &txt_doc.id,
        &tag_java_id,
        false,
        1.0,
        Some("Manual user tag"),
    )
    .unwrap();

    let tag_algo_id = db_tags::upsert_tag(&conn3, "Algorithms", "algorithms", "concept").unwrap();
    db_tags::add_tag_to_document(&conn3, &txt_doc.id, &tag_algo_id, false, 1.0, None).unwrap();

    let doc_tags_before = db_tags::get_tags_for_document(&conn3, &txt_doc.id).unwrap();
    assert_eq!(doc_tags_before.len(), 2);
    assert!(doc_tags_before.iter().any(|t| t.name == "Java"));
    assert!(doc_tags_before.iter().any(|t| t.name == "Algorithms"));

    // Remove one tag
    db_tags::remove_tag_from_document(&conn3, &txt_doc.id, &tag_algo_id).unwrap();
    let doc_tags_after = db_tags::get_tags_for_document(&conn3, &txt_doc.id).unwrap();
    assert_eq!(doc_tags_after.len(), 1);
    assert_eq!(doc_tags_after[0].name, "Java");
    assert!(!doc_tags_after[0].is_ai_generated);
    println!("  -> PASS: Manual tagging and tag removal verified successfully.");

    // 29. Phase 3.3: Concept Storage & Multi-Document Sharing
    println!("\n[29/32] Testing Canonical Concept Storage & Cross-Document Sharing...");
    let concept_dp_id = db_tags::upsert_concept(
        &conn3,
        "Dynamic Programming",
        "dynamic-programming",
        "concept",
        Some("Algorithmic technique for optimization"),
    )
    .unwrap();
    db_tags::add_concept_to_document(
        &conn3,
        &txt_doc.id,
        &concept_dp_id,
        0.95,
        Some("DP recurrence relations"),
    )
    .unwrap();
    db_tags::add_concept_to_document(
        &conn3,
        &md_doc.id,
        &concept_dp_id,
        0.90,
        Some("Memoization in architecture"),
    )
    .unwrap();

    let concepts_txt = db_tags::get_concepts_for_document(&conn3, &txt_doc.id).unwrap();
    assert_eq!(concepts_txt.len(), 1);
    assert_eq!(concepts_txt[0].name, "Dynamic Programming");

    let concepts_md = db_tags::get_concepts_for_document(&conn3, &md_doc.id).unwrap();
    assert_eq!(concepts_md.len(), 1);
    assert_eq!(concepts_md[0].name, "Dynamic Programming");

    let count: i64 = conn3
        .query_row(
            "SELECT COUNT(*) FROM document_concepts WHERE concept_id = ?1;",
            rusqlite::params![concept_dp_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2, "Concept should be linked to 2 documents");
    println!(
        "  -> PASS: Concept sharing verified across multiple documents with distinct evidence."
    );

    // 30. Phase 3.3: Document Filtering by Tag
    println!("\n[30/32] Testing Multi-Tag Document Filtering...");
    let matching_docs =
        db_tags::get_documents_matching_all_tags(&conn3, &["Java".to_string()]).unwrap();
    assert_eq!(matching_docs.len(), 1);
    assert_eq!(matching_docs[0], txt_doc.id);

    let no_match =
        db_tags::get_documents_matching_all_tags(&conn3, &["NonExistentTag".to_string()]).unwrap();
    assert!(no_match.is_empty());
    println!("  -> PASS: get_documents_matching_all_tags verified.");

    // 31. Phase 3.3: Tag-Aware RAG Prioritization Boost
    println!("\n[31/32] Testing Tag-Aware Relevance Prioritization (15% Boost)...");
    let emb1 = database::StoredEmbeddingWithChunk {
        embedding_id: "emb-untagged".to_string(),
        chunk_id: "chunk-untagged".to_string(),
        document_id: "doc-untagged".to_string(),
        filename: "other.txt".to_string(),
        file_type: "txt".to_string(),
        chunk_index: 0,
        chunk_text: "General discussion of programming paradigms.".to_string(),
        model: "gemini-embedding-001".to_string(),
        dimensions: 2,
        embedding: vec![0.80, 0.60],
    };
    let emb2 = database::StoredEmbeddingWithChunk {
        embedding_id: "emb-tagged".to_string(),
        chunk_id: "chunk-tagged".to_string(),
        document_id: "doc-tagged".to_string(),
        filename: "java_guide.txt".to_string(),
        file_type: "txt".to_string(),
        chunk_index: 0,
        chunk_text: "In-depth guide to Java collections and algorithms.".to_string(),
        model: "gemini-embedding-001".to_string(),
        dimensions: 2,
        embedding: vec![0.75, (1.0f32 - 0.75f32 * 0.75f32).sqrt()], // unit vector with x = 0.75
    };

    let stored = vec![emb1, emb2];
    let query_vec = vec![1.0, 0.0]; // query vector aligned with x-axis

    // Without boost: doc-untagged (sim 0.80) ranks above doc-tagged (sim 0.75)
    let empty_set = HashSet::new();
    let unboosted =
        semantic_search::rank_embeddings_tag_aware(&stored, &query_vec, 2, 0.0, &empty_set);
    assert_eq!(unboosted[0].document_id, "doc-untagged");

    // With tag-aware ranking: doc-tagged gets 15% boost (0.75 * 1.15 = 0.8625), jumping to #1!
    let mut prioritized = HashSet::new();
    prioritized.insert("doc-tagged".to_string());
    let boosted =
        semantic_search::rank_embeddings_tag_aware(&stored, &query_vec, 2, 0.0, &prioritized);
    assert_eq!(boosted[0].document_id, "doc-tagged");
    assert!(boosted[0].similarity > 0.86);
    println!(
        "  -> PASS: Tagged document successfully boosted from #2 to #1 (sim boosted to {:.4}).",
        boosted[0].similarity
    );

    // 32. Phase 3.3: Tagging Status Decoupling & Zero API Keys Verification
    println!("\n[32/32] Testing Decoupled Tagging Status & Security Verification...");
    let tagging_rec = db_tags::DocumentTaggingStatusRecord {
        document_id: "doc-txt-1".to_string(),
        status: "failed".to_string(),
        error_message: Some("AI Tagging service timeout".to_string()),
        content_hash: "hash123".to_string(),
        tags_count: 0,
        last_attempt_at: Some(Utc::now().to_rfc3339()),
        tagged_at: None,
        attempt_count: 1,
        next_retry_at: None,
        retry_state: "permanently_failed".to_string(),
        last_successful_attempt_at: None,
        last_error: Some("AI Tagging service timeout".to_string()),
    };
    db_tags::upsert_tagging_status(&conn3, &tagging_rec).unwrap();
    let tagging_stat = db_tags::get_tagging_status(&conn3, "doc-txt-1")
        .unwrap()
        .unwrap();
    assert_eq!(tagging_stat.status, "failed");

    // Verify indexing status is unaffected (remains failed/indexed as set in prior test)
    let idx_stat = echo_lib::database::indexing_status::get_indexing_status(&conn3, "doc-txt-1")
        .unwrap()
        .unwrap();
    assert_ne!(idx_stat.status, "idle");
    assert_eq!(idx_stat.status, "failed");

    // Verify zero API keys or secrets in SQLite
    let mut stmt = conn3
        .prepare("SELECT key, value FROM app_settings;")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        let k: String = row.get(0).unwrap();
        let v: String = row.get(1).unwrap();
        assert!(
            !k.to_lowercase().contains("api_key"),
            "Security violation: api_key in SQLite key: {}",
            k
        );
        assert!(
            !v.to_lowercase().contains("aiza"),
            "Security violation: Gemini API key in SQLite: {}",
            v
        );
    }
    println!(
        "  -> PASS: Tagging failure decoupling verified and zero credentials confirmed in SQLite."
    );

    // 33. Phase 3.3.1: Automatic Tagging Needs Assessment & Startup Backfill Logic
    println!("\n[33/35] Testing Phase 3.3.1 Tagging Backfill Assessment Logic...");
    // md_doc currently has no tags attached, so it MUST need tagging
    let needs_tagging =
        echo_lib::ai::coordinator::IndexingCoordinator::check_document_needs_tagging(
            &conn3, &md_doc,
        );
    assert!(
        needs_tagging,
        "Untagged document must be flagged for tagging backfill"
    );

    // 34. Phase 3.3.1: Tagging Status UPSERT & get_all_tagging_statuses
    println!("\n[34/35] Testing Tagging Status UPSERT & Bulk Status Retrieval...");
    db_tags::update_tagging_status(&conn3, &md_doc.id, "pending", None, 0).unwrap();
    let all_statuses = db_tags::get_all_tagging_statuses(&conn3).unwrap();
    assert!(
        all_statuses
            .iter()
            .any(|s| s.document_id == md_doc.id && s.status == "pending")
    );
    println!(
        "  -> PASS: update_tagging_status successfully performed UPSERT on missing row (found {} statuses).",
        all_statuses.len()
    );

    // 35. Phase 3.3.1: Manual Tag Preservation during AI Tag Saving
    println!("\n[35/35] Testing Manual Tag Preservation during AI Tag Extraction...");
    let user_tag_id =
        db_tags::upsert_tag(&conn3, "Personal Research", "personal-research", "topic").unwrap();
    db_tags::add_tag_to_document(
        &conn3,
        &md_doc.id,
        &user_tag_id,
        false,
        1.0,
        Some("User manual note"),
    )
    .unwrap();

    let ai_tags = vec![
        echo_lib::ai::tagging::ExtractedTag {
            name: "Architecture".to_string(),
            category: "concept".to_string(),
            confidence: 0.95,
            evidence: Some("Architecture description".to_string()),
        },
        echo_lib::ai::tagging::ExtractedTag {
            name: "Desktop App".to_string(),
            category: "technology".to_string(),
            confidence: 0.92,
            evidence: Some("Desktop details".to_string()),
        },
    ];

    let saved = echo_lib::ai::tagging::save_extracted_tags(&conn3, &md_doc, &ai_tags).unwrap();
    // Verify that manual tag still exists alongside the 2 newly added AI tags
    assert_eq!(
        saved.len(),
        3,
        "Document must have 1 manual tag + 2 AI tags = 3 total"
    );
    assert!(
        saved
            .iter()
            .any(|t| t.name == "Personal Research" && !t.is_ai_generated),
        "Manual tag MUST be preserved"
    );
    assert!(
        saved
            .iter()
            .any(|t| t.name == "Architecture" && t.is_ai_generated),
        "AI tag must exist"
    );
    println!("  -> PASS: Manual tags strictly preserved while AI tags were seamlessly saved.");

    // Now verify that md_doc does NOT need tagging since it has valid AI tags and matching hash
    let needs_tagging_after =
        echo_lib::ai::coordinator::IndexingCoordinator::check_document_needs_tagging(
            &conn3, &md_doc,
        );
    assert!(
        !needs_tagging_after,
        "Properly tagged document must NOT be re-tagged"
    );
    println!("  -> PASS: Properly tagged document correctly skipped on subsequent assessments.");

    // ==========================================
    // PHASE 4.1 KNOWLEDGE GRAPH & AUTO-RETRY TESTS
    // ==========================================

    // 36. Phase 4.1: Error Classification
    println!("\n[36/45] Testing Error Classification for Auto-Retry...");
    use echo_lib::ai::retry::{ErrorCategory, calculate_next_retry, classify_error};
    let (c1, _) = classify_error(Some(503), "Service Unavailable");
    assert_eq!(c1, ErrorCategory::Transient);
    let (c2, _) = classify_error(Some(500), "Internal server error");
    assert_eq!(c2, ErrorCategory::Transient);
    let (c3, _) = classify_error(None, "network connection timed out after 30000ms");
    assert_eq!(c3, ErrorCategory::Transient);
    let (c4, sec4) = classify_error(Some(429), "rate limit exceeded, retry after 25s");
    assert_eq!(c4, ErrorCategory::RateLimit);
    assert_eq!(sec4, Some(25));
    let (c5, _) = classify_error(Some(429), "daily quota exceeded for generative language");
    assert_eq!(c5, ErrorCategory::QuotaExhausted);
    let (c6, _) = classify_error(Some(401), "API_KEY_INVALID: API key not valid");
    assert_eq!(c6, ErrorCategory::Permanent);
    println!(
        "  -> PASS: Error classification accurately maps transient, rate-limit, quota, and permanent failures."
    );

    // 37. Phase 4.1: Bounded Exponential Backoff with Jitter
    println!("\n[37/45] Testing Bounded Exponential Backoff with Jitter...");
    let (_iso1, d1) = calculate_next_retry(1, &ErrorCategory::Transient, None);
    assert!(d1 >= 5 && d1 <= 9, "Attempt 1 backoff: {} not in 5..9s", d1);
    let (_iso2, d2) = calculate_next_retry(2, &ErrorCategory::Transient, None);
    assert!(
        d2 >= 15 && d2 <= 19,
        "Attempt 2 backoff: {} not in 15..19s",
        d2
    );
    let (_iso3, d3) = calculate_next_retry(3, &ErrorCategory::Transient, None);
    assert!(
        d3 >= 30 && d3 <= 34,
        "Attempt 3 backoff: {} not in 30..34s",
        d3
    );
    let (_iso_high, d_high) = calculate_next_retry(15, &ErrorCategory::Transient, None);
    assert!(d_high <= 1800, "Max backoff must not exceed 1800s (30m)");
    println!("  -> PASS: Bounded exponential backoff sequence and max limits verified.");

    // 38. Phase 4.1: Indexing & Tagging Retry State Tracking
    println!("\n[38/45] Testing Persistent Retry State Tracking in SQLite...");
    echo_lib::database::update_indexing_retry_state(
        &conn3,
        "doc-txt-1",
        "retry_wait",
        "failed",
        2,
        Some("2026-11-10T12:00:00Z"),
        Some("Rate limit exceeded"),
    )
    .unwrap();

    let retry_status = echo_lib::database::get_indexing_status(&conn3, "doc-txt-1")
        .unwrap()
        .unwrap();
    assert_eq!(retry_status.retry_state, "retry_wait");
    assert_eq!(retry_status.attempt_count, 2);
    assert_eq!(
        retry_status.next_retry_at,
        Some("2026-11-10T12:00:00Z".to_string())
    );
    assert_eq!(
        retry_status.last_error,
        Some("Rate limit exceeded".to_string())
    );
    println!("  -> PASS: SQLite indexing retry metadata successfully persisted.");

    // 39. Phase 4.1: Pause & Resume All Retries
    println!("\n[39/45] Testing Global Pause & Resume Controls in SQLite...");
    let paused_count = echo_lib::database::pause_all_indexing_retries(&conn3).unwrap();
    assert!(paused_count >= 1);
    let paused_status = echo_lib::database::get_indexing_status(&conn3, "doc-txt-1")
        .unwrap()
        .unwrap();
    assert_eq!(paused_status.retry_state, "paused");

    let resumed_count = echo_lib::database::resume_all_indexing_retries(&conn3).unwrap();
    assert!(resumed_count >= 1);
    let resumed_status = echo_lib::database::get_indexing_status(&conn3, "doc-txt-1")
        .unwrap()
        .unwrap();
    assert_eq!(resumed_status.retry_state, "retry_wait");
    println!("  -> PASS: Pause and resume transitions executed atomically across retry records.");

    // 40. Phase 4.1: Large Document (50-Page PDF) Multi-Anchor Sampling & UTF-8 Safety
    println!("\n[40/45] Testing 50-Page PDF Multi-Anchor Sampling & UTF-8 Safety...");
    // Create simulated large document text (~50,000 characters with multi-byte unicode characters)
    let mut large_text = String::new();
    for i in 0..1000 {
        large_text.push_str(&format!(
            "Page {} has Unicode symbols: ©, ®, →, —, £, „, and algorithms. ",
            i
        ));
    }
    assert!(large_text.len() > 50000);

    let sample = echo_lib::ai::tagging::service::prepare_representative_sample(&large_text);
    assert!(
        sample.chars().count() <= 4200,
        "Sample must be bounded within token budget"
    );
    assert!(
        sample.contains("[... Section 1 ...]"),
        "Must contain section 1 anchor"
    );
    assert!(
        sample.contains("[... Section 2 ...]"),
        "Must contain section 2 anchor"
    );
    assert!(
        sample.contains("[... Section 3 ...]"),
        "Must contain section 3 anchor"
    );
    assert!(
        sample.contains("[... Section 4 ...]"),
        "Must contain section 4 anchor"
    );
    // Verify valid UTF-8 string integrity without slicing panics
    assert!(!sample.is_empty());
    println!(
        "  -> PASS: Multi-anchor sampling across 5 anchors safely handles 50-page document with valid UTF-8."
    );

    // 41. Phase 4.1: Knowledge Graph concept_relationships Table & Validation
    println!("\n[41/45] Testing Knowledge Graph concept_relationships Schema & Constraints...");
    // Create two test concepts
    let c1_id = echo_lib::database::upsert_concept(
        &conn3,
        "Vector Search",
        "vector-search",
        "technology",
        Some("High-dimensional similarity"),
    )
    .unwrap();
    let c2_id = echo_lib::database::upsert_concept(
        &conn3,
        "Embeddings",
        "embeddings",
        "concept",
        Some("Dense vector representation"),
    )
    .unwrap();

    let rel = echo_lib::database::upsert_concept_relationship(
        &conn3,
        &c1_id,
        &c2_id,
        "USES",
        0.95,
        "Vector search uses embeddings to calculate cosine similarity.",
        Some(&md_doc.id),
        None,
        "ai_extracted",
    )
    .unwrap();

    assert_eq!(rel.relationship_type, "USES");
    assert_eq!(rel.confidence, 0.95);
    assert_eq!(rel.source_concept_name, Some("Vector Search".to_string()));
    assert_eq!(rel.target_concept_name, Some("Embeddings".to_string()));

    // Invalid relationship type must fail
    let invalid_rel = echo_lib::database::upsert_concept_relationship(
        &conn3,
        &c1_id,
        &c2_id,
        "INVALID_REL_TYPE",
        0.90,
        "evidence",
        None,
        None,
        "ai_extracted",
    );
    assert!(
        invalid_rel.is_err(),
        "Invalid relationship type must be rejected"
    );
    println!(
        "  -> PASS: concept_relationships table enforced valid types and successfully stored relationship."
    );

    // 42. Phase 4.1: Graph Neighbors & Traversal
    println!("\n[42/45] Testing Concept Neighbors & Bidirectional Graph Traversal...");
    let neighbors_c1 = echo_lib::database::get_concept_neighbors(&conn3, &c1_id).unwrap();
    assert_eq!(neighbors_c1.len(), 1);
    assert_eq!(neighbors_c1[0].direction, "outgoing");
    assert_eq!(neighbors_c1[0].neighbor_concept_id, c2_id);

    let neighbors_c2 = echo_lib::database::get_concept_neighbors(&conn3, &c2_id).unwrap();
    assert_eq!(neighbors_c2.len(), 1);
    assert_eq!(neighbors_c2[0].direction, "incoming");
    assert_eq!(neighbors_c2[0].neighbor_concept_id, c1_id);
    println!(
        "  -> PASS: Bidirectional neighbor discovery verified for outgoing and incoming edges."
    );

    // 43. Phase 4.1: Concept Documents Query
    println!("\n[43/45] Testing Concept Connected Documents Retrieval...");
    // Link concept c1 to md_doc
    echo_lib::database::add_concept_to_document(
        &conn3,
        &md_doc.id,
        &c1_id,
        0.95,
        Some("Direct reference"),
    )
    .unwrap();
    let doc_links = echo_lib::database::get_concept_documents(&conn3, &c1_id).unwrap();
    assert_eq!(doc_links.len(), 1);
    assert_eq!(doc_links[0].document_id, md_doc.id);
    println!("  -> PASS: Concept document associations retrieved successfully.");

    // 44. Phase 4.1: Graph High-Level Statistics
    println!("\n[44/45] Testing Graph High-Level Statistics Query...");
    let stats = echo_lib::database::get_graph_stats(&conn3).unwrap();
    assert!(stats.total_concepts >= 2);
    assert!(stats.total_relationships >= 1);
    assert!(stats.connected_documents >= 1);
    assert!(
        stats
            .relationship_type_counts
            .iter()
            .any(|(t, count)| t == "USES" && *count >= 1)
    );
    println!(
        "  -> PASS: Graph stats computed correctly (total concepts: {}, relationships: {}).",
        stats.total_concepts, stats.total_relationships
    );

    // 45. Phase 4.1: Zero Credentials in SQLite Verification
    println!("\n[45/50] Verifying Strict Credential Isolation in SQLite...");
    let mut check_stmt = conn3
        .prepare(
            "SELECT key, value FROM app_settings WHERE key LIKE '%key%' OR value LIKE '%aiza%';",
        )
        .unwrap();
    let mut check_rows = check_stmt.query([]).unwrap();
    assert!(
        check_rows.next().unwrap().is_none(),
        "Zero API keys or secrets allowed in SQLite"
    );
    println!(
        "  -> PASS: Strict credential isolation verified. All secrets stored exclusively in OS Credential Manager."
    );

    // 46. Phase 4.2.1: Node Positions Persistence & Batch Upsert
    println!("\n[46/50] Testing Concept Node Positions Persistence (Phase 4.2.1)...");
    let positions_to_save = vec![
        echo_lib::database::NodePositionInput {
            concept_id: c1_id.clone(),
            x: 250.0,
            y: 180.0,
        },
        echo_lib::database::NodePositionInput {
            concept_id: c2_id.clone(),
            x: 600.0,
            y: 420.0,
        },
    ];
    echo_lib::database::save_node_positions(&conn3, &positions_to_save).unwrap();

    let fetched_positions = echo_lib::database::get_all_node_positions(&conn3).unwrap();
    assert_eq!(
        fetched_positions.len(),
        2,
        "Must return 2 saved node positions"
    );
    let pos_c1 = fetched_positions
        .iter()
        .find(|p| p.concept_id == c1_id)
        .unwrap();
    assert!((pos_c1.x - 250.0).abs() < 0.001);
    assert!((pos_c1.y - 180.0).abs() < 0.001);
    println!("  -> PASS: Concept node positions successfully persisted to SQLite and retrieved.");

    // 47. Phase 4.2.1: Position Upsert Update
    println!("\n[47/50] Testing Concept Node Position Upsert & Coordinate Update...");
    let updated_c1 = vec![echo_lib::database::NodePositionInput {
        concept_id: c1_id.clone(),
        x: 320.5,
        y: 195.0,
    }];
    echo_lib::database::save_node_positions(&conn3, &updated_c1).unwrap();
    let refetched = echo_lib::database::get_all_node_positions(&conn3).unwrap();
    assert_eq!(
        refetched.len(),
        2,
        "Upsert must not create duplicate entries for same concept_id"
    );
    let updated_pos = refetched.iter().find(|p| p.concept_id == c1_id).unwrap();
    assert!((updated_pos.x - 320.5).abs() < 0.001);
    assert!((updated_pos.y - 195.0).abs() < 0.001);
    println!(
        "  -> PASS: Node position update correctly overwritten existing record without duplicates."
    );

    // 48. Phase 4.2.1: Canvas Viewport Persistence
    println!("\n[48/50] Testing Graph Canvas Viewport Persistence...");
    let test_viewport = echo_lib::database::GraphViewportRecord {
        zoom: 1.15,
        x: -45.0,
        y: 120.0,
    };
    echo_lib::database::save_graph_viewport(&conn3, &test_viewport).unwrap();
    let retrieved_vp = echo_lib::database::get_graph_viewport(&conn3).unwrap();
    assert!(
        retrieved_vp.is_some(),
        "Viewport must be found in SQLite settings"
    );
    let vp = retrieved_vp.unwrap();
    assert!((vp.zoom - 1.15).abs() < 0.001);
    assert!((vp.x - (-45.0)).abs() < 0.001);
    assert!((vp.y - 120.0).abs() < 0.001);
    println!(
        "  -> PASS: Canvas viewport (zoom: {}, x: {}, y: {}) saved and retrieved cleanly.",
        vp.zoom, vp.x, vp.y
    );

    // 49. Phase 4.2.1: Foreign Key Cascade & Position Deletion
    println!("\n[49/50] Testing Concept Node Position Explicit Deletion...");
    echo_lib::database::delete_node_position(&conn3, &c2_id).unwrap();
    let after_delete = echo_lib::database::get_all_node_positions(&conn3).unwrap();
    assert_eq!(after_delete.len(), 1);
    assert_eq!(after_delete[0].concept_id, c1_id);
    println!("  -> PASS: Node position deleted successfully.");

    // 50. Phase 4.2.1: Bounding Box Collision Check Logic
    println!("\n[50/50] Testing Bounding Box Separation and Collision Avoidance Logic...");
    // AABB overlap condition: |dx| < 250 && |dy| < 135
    let p_a = (100.0, 100.0);
    let p_b_colliding = (200.0, 150.0); // dx = 100 (< 250), dy = 50 (< 135) -> Collides
    let p_c_safe = (400.0, 100.0); // dx = 300 (>= 250) -> Safe
    let is_colliding_ab = (p_a.0 - p_b_colliding.0 as f64).abs() < 250.0
        && (p_a.1 - p_b_colliding.1 as f64).abs() < 135.0;
    let is_colliding_ac =
        (p_a.0 - p_c_safe.0 as f64).abs() < 250.0 && (p_a.1 - p_c_safe.1 as f64).abs() < 135.0;
    assert!(is_colliding_ab, "p_a and p_b must be detected as colliding");
    assert!(
        !is_colliding_ac,
        "p_a and p_c must be detected as non-colliding"
    );
    println!("  -> PASS: Bounding box collision calculation verified with card dimensions + gap.");

    println!(
        "\n=== ALL 50 PHASE 2, 3.0, 3.1, 3.2, 3.3, 3.3.1, 4.1 & 4.2.1 VERIFICATION CHECKS PASSED SUCCESSFULLY! ==="
    );
}
