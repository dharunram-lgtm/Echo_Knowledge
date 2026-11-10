pub mod ai;
pub mod commands;
pub mod database;
pub mod security;

use database::AppStorage;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Automatically initialize local database and document storage directories
            let storage = AppStorage::initialize(app.handle())
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            app.manage(storage);
            app.manage(commands::ai::AIState::new());
            let coordinator = ai::IndexingCoordinator::new();
            app.manage(coordinator);

            // Automatically start background indexing coordinator on startup
            ai::coordinator::start_startup_indexing(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_app_info,
            commands::documents::initialize_storage,
            commands::documents::import_documents,
            commands::documents::get_documents,
            commands::documents::get_document,
            commands::documents::search_documents,
            commands::documents::delete_document,
            commands::documents::get_storage_stats,
            // Phase 3 AI & RAG commands
            commands::ai::get_ai_status,
            commands::ai::discover_models,
            commands::ai::check_embedding_compatibility,
            commands::ai::save_api_key,
            commands::ai::remove_api_key,
            commands::ai::test_ai_connection,
            commands::ai::get_model_config,
            commands::ai::update_model_config,
            commands::ai::index_document,
            commands::ai::index_all_documents,
            commands::ai::get_indexing_status,
            commands::ai::get_auto_index_setting,
            commands::ai::set_auto_index_setting,
            commands::ai::retry_failed_indexing,
            commands::ai::semantic_search,
            commands::ai::ask_echo,
            commands::ai::list_conversations,
            commands::ai::create_conversation,
            commands::ai::get_conversation_messages,
            commands::ai::delete_conversation,
            commands::ai::clear_conversation,
            // Phase 3.3 Intelligent Knowledge Tagging & Concepts
            commands::tags::get_document_tags,
            commands::tags::get_all_tags,
            commands::tags::get_all_concepts,
            commands::tags::add_manual_tag,
            commands::tags::remove_document_tag,
            commands::tags::retag_document,
            commands::tags::get_documents_by_tags,
            commands::tags::rename_tag,
            commands::tags::get_all_tagging_statuses,
            commands::tags::retry_failed_tagging,
            commands::tags::retry_tagging_for_document,
            // Phase 4.1 Knowledge Graph & Persistent Auto-Retry
            commands::graph::get_all_relationships,
            commands::graph::get_concept_neighbors,
            commands::graph::get_concept_documents,
            commands::graph::get_document_relationships,
            commands::graph::delete_concept_relationship,
            commands::graph::get_graph_stats,
            commands::graph::save_node_positions,
            commands::graph::get_node_positions,
            commands::graph::save_graph_viewport,
            commands::graph::get_graph_viewport,
            commands::graph::discover_document_relationships,
            commands::graph::discover_all_relationships,
            // Phase 4.3 Knowledge Intelligence Engine
            commands::graph::get_knowledge_clusters,
            commands::graph::recompute_clusters,
            commands::graph::get_suggested_relationships,
            commands::graph::discover_hidden_connections,
            commands::graph::accept_suggested_relationship,
            commands::graph::reject_suggested_relationship,
            commands::graph::explore_concept,
            commands::graph::get_knowledge_evolution_timeline,
            commands::graph::detect_potential_conflicts,
            commands::graph::get_knowledge_insights,
            commands::retry::get_retry_overview,
            commands::retry::pause_all_retries,
            commands::retry::resume_all_retries,
            commands::retry::retry_all_failed,
            commands::retry::retry_document_operation,
            commands::retry::cancel_document_retry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
