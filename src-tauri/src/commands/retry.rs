use crate::ai::coordinator::{run_background_indexing_worker, run_background_tagging_worker};
use crate::ai::retry::RetryOverview;
use crate::database::{self, AppStorage};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_retry_overview(storage: State<'_, AppStorage>) -> Result<RetryOverview, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    crate::ai::coordinator::get_retry_overview(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_all_retries(app: AppHandle, storage: State<'_, AppStorage>) -> Result<usize, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "retries_paused", "true").map_err(|e| e.to_string())?;
    let c1 = database::pause_all_indexing_retries(&conn).unwrap_or(0);
    let c2 = database::pause_all_tagging_retries(&conn).unwrap_or(0);

    let _ = app.emit("indexing:status_changed", ());
    let _ = app.emit("tagging:status_changed", ());
    Ok(c1 + c2)
}

#[tauri::command]
pub fn resume_all_retries(app: AppHandle, storage: State<'_, AppStorage>) -> Result<usize, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "retries_paused", "false").map_err(|e| e.to_string())?;
    let c1 = database::resume_all_indexing_retries(&conn).unwrap_or(0);
    let c2 = database::resume_all_tagging_retries(&conn).unwrap_or(0);

    let _ = app.emit("indexing:status_changed", ());
    let _ = app.emit("tagging:status_changed", ());

    // Kick off workers
    let app_idx = app.clone();
    tauri::async_runtime::spawn(async move {
        run_background_indexing_worker(app_idx, Vec::new()).await;
    });
    let app_tag = app.clone();
    tauri::async_runtime::spawn(async move {
        run_background_tagging_worker(app_tag).await;
    });

    Ok(c1 + c2)
}

#[tauri::command]
pub fn retry_all_failed(app: AppHandle, storage: State<'_, AppStorage>) -> Result<usize, String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;
    database::set_app_setting(&conn, "retries_paused", "false").map_err(|e| e.to_string())?;

    // Reset failed / retry_wait / cancelled / paused indexing to pending
    let c1 = conn
        .execute(
            "UPDATE document_indexing_status 
             SET retry_state = 'pending', status = 'pending', attempt_count = 0, next_retry_at = NULL 
             WHERE retry_state IN ('retry_wait', 'paused', 'permanently_failed', 'cancelled');",
            [],
        )
        .unwrap_or(0);

    // Reset failed / retry_wait / cancelled / paused tagging to pending
    let c2 = conn
        .execute(
            "UPDATE document_tagging_status 
             SET retry_state = 'pending', status = 'pending', attempt_count = 0, next_retry_at = NULL 
             WHERE retry_state IN ('retry_wait', 'paused', 'permanently_failed', 'cancelled');",
            [],
        )
        .unwrap_or(0);

    let _ = app.emit("indexing:status_changed", ());
    let _ = app.emit("tagging:status_changed", ());

    let app_idx = app.clone();
    tauri::async_runtime::spawn(async move {
        run_background_indexing_worker(app_idx, Vec::new()).await;
    });
    let app_tag = app.clone();
    tauri::async_runtime::spawn(async move {
        run_background_tagging_worker(app_tag).await;
    });

    Ok(c1 + c2)
}

#[tauri::command]
pub fn retry_document_operation(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    document_id: String,
    operation: String,
) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    let op = operation.to_lowercase();
    let do_indexing = op == "indexing" || op == "both";
    let do_tagging = op == "tagging" || op == "both";

    if do_indexing {
        database::reset_indexing_retry(&conn, &document_id).map_err(|e| e.to_string())?;
        let _ = app.emit("indexing:status_changed", ());
        let app_idx = app.clone();
        tauri::async_runtime::spawn(async move {
            run_background_indexing_worker(app_idx, Vec::new()).await;
        });
    }

    if do_tagging {
        database::reset_tagging_retry(&conn, &document_id).map_err(|e| e.to_string())?;
        let _ = app.emit("tagging:status_changed", ());
        let app_tag = app.clone();
        tauri::async_runtime::spawn(async move {
            run_background_tagging_worker(app_tag).await;
        });
    }

    Ok(())
}

#[tauri::command]
pub fn cancel_document_retry(
    app: AppHandle,
    storage: State<'_, AppStorage>,
    document_id: String,
    operation: String,
) -> Result<(), String> {
    let conn = storage.conn.lock().map_err(|e| e.to_string())?;

    let op = operation.to_lowercase();
    let do_indexing = op == "indexing" || op == "both";
    let do_tagging = op == "tagging" || op == "both";

    if do_indexing {
        database::cancel_indexing_retry(&conn, &document_id).map_err(|e| e.to_string())?;
        let _ = app.emit("indexing:status_changed", ());
    }

    if do_tagging {
        database::cancel_tagging_retry(&conn, &document_id).map_err(|e| e.to_string())?;
        let _ = app.emit("tagging:status_changed", ());
    }

    Ok(())
}
