use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    let current_version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| format!("Failed to read schema version: {}", e))?;

    if current_version < 1 {
        apply_v1_migration(conn).map_err(|e| format!("Failed to apply migration v1: {}", e))?;
    }

    if current_version < 2 {
        apply_v2_migration(conn).map_err(|e| format!("Failed to apply migration v2: {}", e))?;
    }

    if current_version < 3 {
        apply_v3_migration(conn).map_err(|e| format!("Failed to apply migration v3: {}", e))?;
    }

    if current_version < 4 {
        apply_v4_migration(conn).map_err(|e| format!("Failed to apply migration v4: {}", e))?;
    }

    if current_version < 5 {
        apply_v5_migration(conn).map_err(|e| format!("Failed to apply migration v5: {}", e))?;
    }

    if current_version < 6 {
        apply_v6_migration(conn).map_err(|e| format!("Failed to apply migration v6: {}", e))?;
    }

    if current_version < 7 {
        apply_v7_migration(conn).map_err(|e| format!("Failed to apply migration v7: {}", e))?;
    }

    if current_version < 8 {
        apply_v8_migration(conn).map_err(|e| format!("Failed to apply migration v8: {}", e))?;
    }

    if current_version < 9 {
        apply_v9_migration(conn).map_err(|e| format!("Failed to apply migration v9: {}", e))?;
    }

    if current_version < 10 {
        apply_v10_migration(conn).map_err(|e| format!("Failed to apply migration v10: {}", e))?;
    }

    if current_version < 11 {
        apply_v11_migration(conn).map_err(|e| format!("Failed to apply migration v11: {}", e))?;
    }

    if current_version < 12 {
        apply_v12_migration(conn).map_err(|e| format!("Failed to apply migration v12: {}", e))?;
    }

    Ok(())
}

fn apply_v1_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY NOT NULL,
            filename TEXT NOT NULL,
            original_path TEXT NOT NULL,
            stored_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            extracted_text TEXT NOT NULL,
            content_hash TEXT NOT NULL UNIQUE,
            imported_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_documents_content_hash ON documents(content_hash);
        CREATE INDEX IF NOT EXISTS idx_documents_filename ON documents(filename);
        CREATE INDEX IF NOT EXISTS idx_documents_file_type ON documents(file_type);
        CREATE INDEX IF NOT EXISTS idx_documents_imported_at ON documents(imported_at);

        PRAGMA user_version = 1;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v2_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS document_chunks (
            id TEXT PRIMARY KEY NOT NULL,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            chunk_index INTEGER NOT NULL,
            chunk_text TEXT NOT NULL,
            char_count INTEGER NOT NULL,
            word_count INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON document_chunks(document_id);
        CREATE INDEX IF NOT EXISTS idx_chunks_doc_index ON document_chunks(document_id, chunk_index);

        CREATE TABLE IF NOT EXISTS document_embeddings (
            id TEXT PRIMARY KEY NOT NULL,
            chunk_id TEXT NOT NULL UNIQUE REFERENCES document_chunks(id) ON DELETE CASCADE,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            model TEXT NOT NULL,
            dimensions INTEGER NOT NULL,
            embedding_blob BLOB NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_embeddings_chunk_id ON document_embeddings(chunk_id);
        CREATE INDEX IF NOT EXISTS idx_embeddings_doc_id ON document_embeddings(document_id);
        CREATE INDEX IF NOT EXISTS idx_embeddings_model ON document_embeddings(model);

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at);

        CREATE TABLE IF NOT EXISTS conversation_messages (
            id TEXT PRIMARY KEY NOT NULL,
            conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
            content TEXT NOT NULL,
            citations_json TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON conversation_messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created ON conversation_messages(created_at);

        PRAGMA user_version = 2;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v3_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        PRAGMA user_version = 3;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v4_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS document_indexing_status (
            document_id TEXT PRIMARY KEY NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK(status IN ('indexed', 'pending', 'indexing', 'failed', 'awaiting_key')),
            error_message TEXT,
            content_hash TEXT NOT NULL,
            model TEXT,
            dimensions INTEGER,
            chunks_count INTEGER NOT NULL DEFAULT 0,
            embeddings_count INTEGER NOT NULL DEFAULT 0,
            last_attempt_at TEXT,
            indexed_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_indexing_status ON document_indexing_status(status);

        PRAGMA user_version = 4;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v5_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        -- Tags table: normalized unique tags
        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL UNIQUE,
            normalized_name TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL DEFAULT 'concept' CHECK(category IN ('technology', 'concept', 'topic')),
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tags_normalized_name ON tags(normalized_name);
        CREATE INDEX IF NOT EXISTS idx_tags_category ON tags(category);

        -- Document tags join table
        CREATE TABLE IF NOT EXISTS document_tags (
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            is_ai_generated INTEGER NOT NULL DEFAULT 1,
            confidence REAL NOT NULL DEFAULT 1.0,
            evidence TEXT,
            created_at TEXT NOT NULL,
            PRIMARY KEY (document_id, tag_id)
        );

        CREATE INDEX IF NOT EXISTS idx_doc_tags_doc_id ON document_tags(document_id);
        CREATE INDEX IF NOT EXISTS idx_doc_tags_tag_id ON document_tags(tag_id);

        -- Concepts table for Phase 4 Knowledge Graph reusability
        CREATE TABLE IF NOT EXISTS concepts (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            canonical_id TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL DEFAULT 'concept' CHECK(category IN ('technology', 'concept', 'topic')),
            description TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_concepts_canonical_id ON concepts(canonical_id);

        -- Document concepts association table
        CREATE TABLE IF NOT EXISTS document_concepts (
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            confidence REAL NOT NULL DEFAULT 1.0,
            evidence TEXT,
            created_at TEXT NOT NULL,
            PRIMARY KEY (document_id, concept_id)
        );

        CREATE INDEX IF NOT EXISTS idx_doc_concepts_doc_id ON document_concepts(document_id);
        CREATE INDEX IF NOT EXISTS idx_doc_concepts_concept_id ON document_concepts(concept_id);

        -- Document tagging status table
        CREATE TABLE IF NOT EXISTS document_tagging_status (
            document_id TEXT PRIMARY KEY NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK(status IN ('untagged', 'pending', 'tagging', 'tagged', 'failed', 'awaiting_key')),
            error_message TEXT,
            content_hash TEXT NOT NULL,
            tags_count INTEGER NOT NULL DEFAULT 0,
            last_attempt_at TEXT,
            tagged_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_document_tagging_status ON document_tagging_status(status);

        PRAGMA user_version = 5;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v6_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // 1. Add retry metadata to document_indexing_status
    let _ = tx.execute(
        "ALTER TABLE document_indexing_status ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 0;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_indexing_status ADD COLUMN next_retry_at TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_indexing_status ADD COLUMN retry_state TEXT NOT NULL DEFAULT 'pending';",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_indexing_status ADD COLUMN last_successful_attempt_at TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_indexing_status ADD COLUMN last_error TEXT;",
        [],
    );

    let _ = tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_indexing_retry ON document_indexing_status(retry_state, next_retry_at);",
        [],
    );

    let _ = tx.execute(
        "UPDATE document_indexing_status SET retry_state = 'succeeded', last_successful_attempt_at = indexed_at WHERE status = 'indexed';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_indexing_status SET retry_state = 'awaiting_key' WHERE status = 'awaiting_key';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_indexing_status SET retry_state = 'permanently_failed', last_error = error_message WHERE status = 'failed';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_indexing_status SET retry_state = 'pending' WHERE status = 'pending';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_indexing_status SET retry_state = 'processing' WHERE status = 'indexing';",
        [],
    );

    // 2. Add retry metadata to document_tagging_status
    let _ = tx.execute(
        "ALTER TABLE document_tagging_status ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 0;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_tagging_status ADD COLUMN next_retry_at TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_tagging_status ADD COLUMN retry_state TEXT NOT NULL DEFAULT 'pending';",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_tagging_status ADD COLUMN last_successful_attempt_at TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE document_tagging_status ADD COLUMN last_error TEXT;",
        [],
    );

    let _ = tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_tagging_retry ON document_tagging_status(retry_state, next_retry_at);",
        [],
    );

    let _ = tx.execute(
        "UPDATE document_tagging_status SET retry_state = 'succeeded', last_successful_attempt_at = tagged_at WHERE status = 'tagged';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_tagging_status SET retry_state = 'awaiting_key' WHERE status = 'awaiting_key';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_tagging_status SET retry_state = 'permanently_failed', last_error = error_message WHERE status = 'failed';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_tagging_status SET retry_state = 'pending' WHERE status = 'pending';",
        [],
    );
    let _ = tx.execute(
        "UPDATE document_tagging_status SET retry_state = 'processing' WHERE status = 'tagging';",
        [],
    );

    // 3. Knowledge Graph concept_relationships table
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS concept_relationships (
            id TEXT PRIMARY KEY NOT NULL,
            source_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            target_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            relationship_type TEXT NOT NULL CHECK(relationship_type IN (
                'RELATED_TO', 'IS_A', 'PART_OF', 'USES', 'IMPLEMENTS', 'REQUIRES', 'COMPARED_WITH', 'INCLUDES'
            )),
            confidence REAL NOT NULL DEFAULT 1.0,
            evidence TEXT NOT NULL,
            source_document_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
            source_chunk_id TEXT REFERENCES document_chunks(id) ON DELETE SET NULL,
            discovery_method TEXT NOT NULL DEFAULT 'ai_extracted',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(source_concept_id, target_concept_id, relationship_type)
        );

        CREATE INDEX IF NOT EXISTS idx_rel_source ON concept_relationships(source_concept_id);
        CREATE INDEX IF NOT EXISTS idx_rel_target ON concept_relationships(target_concept_id);
        CREATE INDEX IF NOT EXISTS idx_rel_type ON concept_relationships(relationship_type);
        CREATE INDEX IF NOT EXISTS idx_rel_doc ON concept_relationships(source_document_id);

        PRAGMA user_version = 6;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v7_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS concept_node_positions (
            concept_id TEXT PRIMARY KEY NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            position_x REAL NOT NULL,
            position_y REAL NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_concept_node_pos_id ON concept_node_positions(concept_id);

        PRAGMA user_version = 7;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v8_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS suggested_relationships (
            id TEXT PRIMARY KEY NOT NULL,
            source_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            target_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            relationship_type TEXT NOT NULL CHECK(relationship_type IN (
                'RELATED_TO', 'IS_A', 'PART_OF', 'USES', 'IMPLEMENTS', 'REQUIRES', 'COMPARED_WITH', 'INCLUDES'
            )),
            confidence REAL NOT NULL DEFAULT 0.75,
            evidence TEXT NOT NULL,
            explanation TEXT NOT NULL,
            source_document_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
            status TEXT NOT NULL DEFAULT 'suggested' CHECK(status IN ('suggested', 'accepted', 'rejected')),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(source_concept_id, target_concept_id, relationship_type)
        );

        CREATE INDEX IF NOT EXISTS idx_suggested_status ON suggested_relationships(status);
        CREATE INDEX IF NOT EXISTS idx_suggested_source ON suggested_relationships(source_concept_id);
        CREATE INDEX IF NOT EXISTS idx_suggested_target ON suggested_relationships(target_concept_id);

        CREATE TABLE IF NOT EXISTS knowledge_clusters (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            color TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS concept_cluster_members (
            cluster_id TEXT NOT NULL REFERENCES knowledge_clusters(id) ON DELETE CASCADE,
            concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            is_primary INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (cluster_id, concept_id)
        );

        CREATE INDEX IF NOT EXISTS idx_cluster_members_concept ON concept_cluster_members(concept_id);

        CREATE TABLE IF NOT EXISTS potential_conflicts (
            id TEXT PRIMARY KEY NOT NULL,
            concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
            statement_a TEXT NOT NULL,
            doc_a_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
            doc_a_filename TEXT NOT NULL,
            statement_b TEXT NOT NULL,
            doc_b_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
            doc_b_filename TEXT NOT NULL,
            explanation TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_conflicts_concept ON potential_conflicts(concept_id);

        PRAGMA user_version = 8;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v9_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS collections (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            color TEXT,
            icon TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_collections_name ON collections(name COLLATE NOCASE);

        CREATE TABLE IF NOT EXISTS collection_documents (
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            added_at TEXT NOT NULL,
            PRIMARY KEY (collection_id, document_id)
        );

        CREATE INDEX IF NOT EXISTS idx_collection_documents_document
            ON collection_documents(document_id);

        PRAGMA user_version = 9;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v10_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute_batch(
        "
        -- 1. Remove relationships whose endpoints or source documents no longer exist
        DELETE FROM concept_relationships
        WHERE source_concept_id NOT IN (SELECT id FROM concepts)
           OR target_concept_id NOT IN (SELECT id FROM concepts);

        DELETE FROM suggested_relationships
        WHERE source_concept_id NOT IN (SELECT id FROM concepts)
           OR target_concept_id NOT IN (SELECT id FROM concepts);

        -- Purge relationships originating from non-existent documents
        DELETE FROM concept_relationships
        WHERE source_document_id IS NOT NULL
          AND source_document_id NOT IN (SELECT id FROM documents);

        DELETE FROM suggested_relationships
        WHERE source_document_id IS NOT NULL
          AND source_document_id NOT IN (SELECT id FROM documents);

        -- 2. Clean orphaned concepts with 0 remaining document associations, preserving manually created ones
        DELETE FROM concepts
        WHERE id NOT IN (SELECT DISTINCT concept_id FROM document_concepts)
          AND (description IS NULL OR description != 'User-defined concept');

        -- 3. Secondary relationship purge in case concepts were deleted above
        DELETE FROM concept_relationships
        WHERE source_concept_id NOT IN (SELECT id FROM concepts)
           OR target_concept_id NOT IN (SELECT id FROM concepts);

        DELETE FROM suggested_relationships
        WHERE source_concept_id NOT IN (SELECT id FROM concepts)
           OR target_concept_id NOT IN (SELECT id FROM concepts);

        -- 4. Clean orphaned tags with no document associations
        DELETE FROM tags
        WHERE id NOT IN (SELECT DISTINCT tag_id FROM document_tags);

        PRAGMA user_version = 10;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v11_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // 1. Add Web Note metadata columns to documents (nullable, fully backward-compatible)
    let _ = tx.execute(
        "ALTER TABLE documents ADD COLUMN source_url TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE documents ADD COLUMN author TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE documents ADD COLUMN favicon_url TEXT;",
        [],
    );
    let _ = tx.execute(
        "ALTER TABLE documents ADD COLUMN metadata_json TEXT;",
        [],
    );

    // 2. Index source_url for duplicate URL checks
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_documents_source_url ON documents(source_url);",
        [],
    )?;

    // 3. Idempotency table for network resiliency & duplicate sync prevention (Initial v11 schema)
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS clipper_sync_idempotency (
            idempotency_key TEXT PRIMARY KEY NOT NULL,
            document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_clipper_idempotency_created
            ON clipper_sync_idempotency(created_at);

        PRAGMA user_version = 11;
        ",
    )?;

    tx.commit()?;
    Ok(())
}

fn apply_v12_migration(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // Safely add payload_hash to clipper_sync_idempotency if it doesn't exist
    let has_payload_hash: bool = tx
        .prepare("PRAGMA table_info(clipper_sync_idempotency);")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|col| col == "payload_hash");

    if !has_payload_hash {
        tx.execute(
            "ALTER TABLE clipper_sync_idempotency ADD COLUMN payload_hash TEXT NOT NULL DEFAULT '';",
            [],
        )?;
    }

    tx.execute("PRAGMA user_version = 12;", [])?;
    tx.commit()?;
    Ok(())
}
