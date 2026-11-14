// ECHO Web Clipper - Background Service Worker (Manifest V3)

const DEFAULT_PORT = 23119;
const FALLBACK_PORT = 23120;
const MAX_QUEUE_SIZE = 100;
const MAX_AUTO_RETRY_ATTEMPTS = 5;
const ALARM_QUEUE_NAME = 'echo_clipper_queue_sync';

// State in memory to prevent concurrent queue drain cycles
let isDrainingQueue = false;

// 1. Lifecycle & Alarms
chrome.runtime.onInstalled.addListener(() => {
  console.log('[ECHO Clipper] Background Service Worker installed. Runtime Extension ID:', chrome.runtime.id);
  if (chrome.alarms && typeof chrome.alarms.create === 'function') {
    try {
      chrome.alarms.create(ALARM_QUEUE_NAME, { periodInMinutes: 1 });
    } catch (err) {
      console.warn('[ECHO Clipper] Failed to create background alarm:', err);
    }
  }
});

if (chrome.alarms && chrome.alarms.onAlarm && typeof chrome.alarms.onAlarm.addListener === 'function') {
  try {
    chrome.alarms.onAlarm.addListener((alarm) => {
      if (alarm && alarm.name === ALARM_QUEUE_NAME) {
        drainQueueSafely();
      }
    });
  } catch (err) {
    console.warn('[ECHO Clipper] Failed to attach alarm listener:', err);
  }
}

// 2. Message Routing from Popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (!request || !request.type) return false;

  (async () => {
    try {
      switch (request.type) {
        case 'CHECK_STATUS': {
          const config = await getConfig();
          const hasToken = Boolean(config.token && config.token.trim().length > 0);
          const queue = await getQueue();
          const pendingCount = queue.filter(item => item.status !== 'failed').length;
          const failedCount = queue.filter(item => item.status === 'failed').length;

          if (!hasToken) {
            // Pairing configuration incomplete
            sendResponse({
              success: true,
              status: 'incomplete_config',
              statusCode: 'incomplete_config',
              port: config.preferredPort,
              extensionId: chrome.runtime.id,
              queueCount: pendingCount,
              failedCount: failedCount,
              error: 'Pairing token not configured. Open Clipper Settings to paste token.'
            });
            break;
          }

          const { activePort, health } = await discoverActivePortAndHealth(config.preferredPort, config.token);

          // Accurate Connection State Machine:
          // - connected: Server reachable, token valid, extension paired (200 OK)
          // - invalid_token: Server reachable, token invalid or expired (401 Unauthorized)
          // - unpaired: Server reachable, token valid, extension ID not authorized (403 Forbidden)
          // - offline: Server unreachable / TCP connection failed (Network Error / HTTP 0)
          let finalStatus = 'offline';
          if (health.online) {
            if (health.authenticated && health.statusCode === 'connected') {
              finalStatus = 'connected';
            } else if (health.statusCode === 'invalid_token') {
              finalStatus = 'invalid_token';
            } else if (health.statusCode === 'unpaired') {
              finalStatus = 'unpaired';
            } else {
              finalStatus = health.statusCode || 'unpaired';
            }
          }

          sendResponse({
            success: true,
            status: finalStatus,
            statusCode: health.statusCode,
            httpStatus: health.httpStatus,
            port: activePort,
            extensionId: chrome.runtime.id,
            queueCount: pendingCount,
            failedCount: failedCount,
            error: health.error || null
          });
          break;
        }

        case 'SAVE_CONFIG': {
          const { port, token } = request;
          await chrome.storage.local.set({
            echo_port: port ? parseInt(port, 10) : DEFAULT_PORT,
            echo_token: token ? token.trim() : ''
          });
          sendResponse({ success: true });
          break;
        }

        case 'GET_CONFIG': {
          const config = await getConfig();
          sendResponse({
            success: true,
            port: config.preferredPort,
            token: config.token,
            extensionId: chrome.runtime.id
          });
          break;
        }

        case 'CLIP_NOW': {
          const { payload, duplicate_strategy } = request;
          const result = await handleClipRequest(payload, duplicate_strategy);
          sendResponse(result);
          break;
        }

        case 'GET_QUEUE': {
          const queue = await getQueue();
          sendResponse({ success: true, queue });
          break;
        }

        case 'RETRY_QUEUE': {
          const drainResult = await drainQueueSafely(true); // manual retry retries failed too
          sendResponse({ success: true, result: drainResult });
          break;
        }

        case 'REMOVE_QUEUE_ITEM': {
          const { id } = request;
          const queue = await getQueue();
          const updated = queue.filter(item => item.id !== id);
          await saveQueue(updated);
          sendResponse({ success: true });
          break;
        }

        case 'CLEAR_QUEUE': {
          await chrome.storage.local.set({ echo_clipper_queue: [] });
          sendResponse({ success: true });
          break;
        }

        default:
          sendResponse({ success: false, error: 'Unknown message type: ' + request.type });
      }
    } catch (err) {
      console.error('[ECHO Clipper Worker] Message error:', err);
      sendResponse({ success: false, error: err.message || 'Operation failed' });
    }
  })();

  return true; // Keep channel open for async response
});

// 3. Storage & Configuration Helpers
async function getConfig() {
  const data = await chrome.storage.local.get(['echo_port', 'echo_token']);
  return {
    preferredPort: data.echo_port || DEFAULT_PORT,
    token: data.echo_token || ''
  };
}

async function getQueue() {
  const data = await chrome.storage.local.get('echo_clipper_queue');
  if (Array.isArray(data.echo_clipper_queue)) {
    return data.echo_clipper_queue;
  }
  return [];
}

async function saveQueue(queue) {
  try {
    await chrome.storage.local.set({ echo_clipper_queue: queue.slice(0, MAX_QUEUE_SIZE) });
  } catch (err) {
    console.error('[ECHO Clipper] Failed to persist queue to storage:', err);
    throw new Error('Local queue storage quota exceeded or failed.');
  }
}

// 4. Port Discovery & Health Checking
async function checkHealthOnPort(port, token) {
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 2000);

    const headers = {};
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    // Notice: Never set the 'Origin' header manually. The browser automatically
    // provides the true Chromium origin: chrome-extension://<extension-id>
    const res = await fetch(`http://127.0.0.1:${port}/api/v1/health`, {
      method: 'GET',
      headers,
      signal: controller.signal
    });
    clearTimeout(timeout);

    if (res.status === 200) {
      const data = await res.json().catch(() => ({}));
      return {
        online: true,
        authenticated: data.authenticated === true,
        httpStatus: 200,
        statusCode: 'connected',
        error: null
      };
    } else if (res.status === 401) {
      const data = await res.json().catch(() => ({}));
      return {
        online: true,
        authenticated: false,
        httpStatus: 401,
        statusCode: 'invalid_token',
        error: data.error || 'Invalid or expired pairing token. Please update token in Clipper Settings.'
      };
    } else if (res.status === 403) {
      const data = await res.json().catch(() => ({}));
      return {
        online: true,
        authenticated: false,
        httpStatus: 403,
        statusCode: 'unpaired',
        error: data.error || 'Extension not authorized. Please pair this browser in ECHO Settings.'
      };
    } else {
      return {
        online: false,
        authenticated: false,
        httpStatus: res.status,
        statusCode: 'offline',
        error: `HTTP ${res.status}`
      };
    }
  } catch (e) {
    return {
      online: false,
      authenticated: false,
      httpStatus: 0,
      statusCode: 'offline',
      error: e.message || 'Server unreachable'
    };
  }
}

async function discoverActivePortAndHealth(preferredPort, token) {
  // 1. Try preferred port (default: 23119)
  const healthPreferred = await checkHealthOnPort(preferredPort, token);
  if (healthPreferred.online) {
    return { activePort: preferredPort, health: healthPreferred };
  }

  // 2. If preferred was 23119 and failed, check fallback port 23120
  if (preferredPort === DEFAULT_PORT) {
    const healthFallback = await checkHealthOnPort(FALLBACK_PORT, token);
    if (healthFallback.online) {
      return { activePort: FALLBACK_PORT, health: healthFallback };
    }
  }

  return { activePort: preferredPort, health: healthPreferred };
}

// 5. API Transmission Layer (Browser-Controlled Origin)
async function sendClipToBackend(payload, duplicate_strategy, idempotencyKey, port, token) {
  const body = {
    url: payload.url,
    title: payload.title,
    content_markdown: payload.content_markdown,
    extracted_text: payload.extracted_text,
    author: payload.author || null,
    favicon_url: payload.favicon_url || null,
    metadata: payload.metadata || {},
    duplicate_strategy: duplicate_strategy || null
  };

  const headers = {
    'Content-Type': 'application/json',
    'X-Idempotency-Key': idempotencyKey,
    'X-ECHO-Client-Version': '1.2.1'
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  // Standard fetch without spoofing Origin - browser automatically sets chrome-extension://<id>
  const res = await fetch(`http://127.0.0.1:${port}/api/v1/clip`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body)
  });

  const data = await res.json().catch(() => ({ error: 'Non-JSON server response' }));

  return {
    status: res.status,
    ok: res.ok,
    data
  };
}

// 6. Clip Request Handling & Queue Routing
async function handleClipRequest(payload, duplicate_strategy) {
  const config = await getConfig();

  if (!config.token) {
    return {
      success: false,
      error: 'ECHO pairing token is missing. Please enter your pairing token in extension settings.'
    };
  }

  // Generate unique idempotency key for this capture
  const idempotencyKey = crypto.randomUUID();

  // Discover active port
  const { activePort, health } = await discoverActivePortAndHealth(config.preferredPort, config.token);

  if (health.online) {
    try {
      const res = await sendClipToBackend(payload, duplicate_strategy, idempotencyKey, activePort, config.token);

      if (res.status === 200 || res.status === 201) {
        return {
          success: true,
          action: res.data.status, // "created", "updated", "skipped"
          document_id: res.data.document_id,
          filename: res.data.filename,
          message: res.data.message
        };
      }

      if (res.status === 409) {
        // Duplicate conflict requiring user choice
        return {
          success: false,
          conflict: true,
          duplicate_detected: true,
          document_id: res.data.document_id,
          filename: res.data.filename,
          message: res.data.message
        };
      }

      if (res.status === 401 || res.status === 403) {
        // Pairing or permission error - do not queue, inform user immediately
        return {
          success: false,
          error: res.data.error || 'Authorization failed. Verify your pairing token and extension pairing in ECHO.'
        };
      }

      throw new Error(res.data.error || `Server responded with HTTP ${res.status}`);
    } catch (err) {
      console.warn('[ECHO Clipper] Request error, saving to offline queue:', err.message);
      return await enqueueCapture(payload, duplicate_strategy, idempotencyKey, err.message);
    }
  } else {
    // ECHO is offline or unreachable - safely persist to offline queue
    return await enqueueCapture(payload, duplicate_strategy, idempotencyKey, 'ECHO desktop is offline');
  }
}

async function enqueueCapture(payload, duplicate_strategy, idempotencyKey, initialError) {
  const queue = await getQueue();

  if (queue.length >= MAX_QUEUE_SIZE) {
    return {
      success: false,
      error: 'Offline queue is full (100 items). Please open ECHO to sync or manage pending clips.'
    };
  }

  const queueItem = {
    id: idempotencyKey,
    idempotency_key: idempotencyKey,
    payload,
    duplicate_strategy: duplicate_strategy || 'skip',
    attempts: 0,
    status: 'pending', // "pending", "failed"
    created_at: new Date().toISOString(),
    last_error: initialError,
    last_attempt_at: new Date().toISOString()
  };

  queue.push(queueItem);
  await saveQueue(queue);

  return {
    success: true,
    queued: true,
    message: 'ECHO desktop is currently offline. Capture saved to offline queue and will sync automatically when ECHO is opened.'
  };
}

// 7. Safe Offline Queue Drainage with Failed-Item State Preservation
async function drainQueueSafely(includeFailed = false) {
  if (isDrainingQueue) {
    return { skipped: true, reason: 'Already draining' };
  }

  isDrainingQueue = true;
  try {
    const config = await getConfig();
    if (!config.token) {
      return { skipped: true, reason: 'No pairing token' };
    }

    const { activePort, health } = await discoverActivePortAndHealth(config.preferredPort, config.token);
    if (!health.online || !health.authenticated) {
      return { skipped: true, reason: 'Backend offline or unauthenticated' };
    }

    let queue = await getQueue();
    if (queue.length === 0) {
      return { processed: 0 };
    }

    const updatedQueue = [];
    let processedCount = 0;

    for (const item of queue) {
      // Validate item integrity
      if (!item || !item.payload || !item.idempotency_key) {
        continue; // Discard corrupted entries without payload
      }

      // If item is already marked 'failed', only retry if user requested manual sync
      if (item.status === 'failed' && !includeFailed) {
        updatedQueue.push(item);
        continue;
      }

      try {
        const res = await sendClipToBackend(
          item.payload,
          item.duplicate_strategy,
          item.idempotency_key,
          activePort,
          config.token
        );

        if (res.status === 200 || res.status === 201 || res.status === 409) {
          // Success or already satisfied conflict - remove from queue
          processedCount++;
        } else {
          // Server returned error status
          item.attempts += 1;
          item.last_error = res.data.error || `HTTP ${res.status}`;
          item.last_attempt_at = new Date().toISOString();

          if (item.attempts >= MAX_AUTO_RETRY_ATTEMPTS) {
            // NEVER delete captured content: mark as failed for manual review
            item.status = 'failed';
          }
          updatedQueue.push(item);
        }
      } catch (e) {
        item.attempts += 1;
        item.last_error = e.message;
        item.last_attempt_at = new Date().toISOString();

        if (item.attempts >= MAX_AUTO_RETRY_ATTEMPTS) {
          // Transition to persistent failed state instead of deleting
          item.status = 'failed';
        }
        updatedQueue.push(item);
      }
    }

    await saveQueue(updatedQueue);
    return { processed: processedCount, remaining: updatedQueue.length };
  } finally {
    isDrainingQueue = false;
  }
}
