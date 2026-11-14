// ECHO Web Clipper - Popup Script

document.addEventListener('DOMContentLoaded', async () => {
  // Elements
  const statusIndicator = document.getElementById('statusIndicator');
  const statusText = document.getElementById('statusText');
  const settingsBtn = document.getElementById('settingsBtn');
  const pageFavicon = document.getElementById('pageFavicon');
  const pageDomain = document.getElementById('pageDomain');
  const pageTitle = document.getElementById('pageTitle');
  const btnArticle = document.getElementById('btnArticle');
  const btnSelection = document.getElementById('btnSelection');
  const btnBookmark = document.getElementById('btnBookmark');
  const duplicateView = document.getElementById('duplicateView');
  const duplicateDesc = document.getElementById('duplicateDesc');
  const btnDupUpdate = document.getElementById('btnDupUpdate');
  const btnDupCopy = document.getElementById('btnDupCopy');
  const btnDupSkip = document.getElementById('btnDupSkip');
  const feedbackView = document.getElementById('feedbackView');
  const feedbackText = document.getElementById('feedbackText');
  const queueCount = document.getElementById('queueCount');
  const failedBadge = document.getElementById('failedBadge');
  const failedCount = document.getElementById('failedCount');
  const btnViewQueue = document.getElementById('btnViewQueue');
  const btnSyncQueue = document.getElementById('btnSyncQueue');

  // Queue Modal Elements
  const queueModal = document.getElementById('queueModal');
  const closeQueueBtn = document.getElementById('closeQueueBtn');
  const queueListContainer = document.getElementById('queueListContainer');
  const btnRetryAllQueue = document.getElementById('btnRetryAllQueue');
  const btnClearAllQueue = document.getElementById('btnClearAllQueue');

  // Settings Elements
  const settingsView = document.getElementById('settingsView');
  const closeSettingsBtn = document.getElementById('closeSettingsBtn');
  const settingPort = document.getElementById('settingPort');
  const settingToken = document.getElementById('settingToken');
  const settingExtId = document.getElementById('settingExtId');
  const copyExtIdBtn = document.getElementById('copyExtIdBtn');
  const saveSettingsBtn = document.getElementById('saveSettingsBtn');

  let currentTab = null;
  let activePayload = null;

  // Initialize view
  await initActiveTab();
  await refreshStatus();

  // Settings Open/Close
  settingsBtn.addEventListener('click', async () => {
    // Show settings view immediately so UI navigates even if worker is still starting
    settingsView.classList.remove('hidden');
    settingExtId.value = chrome.runtime.id;

    // Fetch config asynchronously with graceful fallback
    const configResp = await sendMessageToWorker({ type: 'GET_CONFIG' });
    if (configResp && configResp.success) {
      settingPort.value = configResp.port || 23119;
      settingToken.value = configResp.token || '';
      settingExtId.value = configResp.extensionId || chrome.runtime.id;
    } else {
      // Local fallback from storage directly if worker did not reply in time
      const data = await chrome.storage.local.get(['echo_port', 'echo_token']).catch(() => ({}));
      settingPort.value = data.echo_port || 23119;
      settingToken.value = data.echo_token || '';
    }
  });

  closeSettingsBtn.addEventListener('click', () => {
    settingsView.classList.add('hidden');
  });

  copyExtIdBtn.addEventListener('click', () => {
    navigator.clipboard.writeText(settingExtId.value);
    copyExtIdBtn.textContent = 'Copied!';
    setTimeout(() => {
      copyExtIdBtn.textContent = 'Copy';
    }, 1500);
  });

  saveSettingsBtn.addEventListener('click', async () => {
    const port = parseInt(settingPort.value, 10) || 23119;
    const token = settingToken.value.trim();

    await sendMessageToWorker({
      type: 'SAVE_CONFIG',
      port,
      token
    });

    settingsView.classList.add('hidden');
    showFeedback('info', 'Settings saved. Rechecking connection...');
    await refreshStatus();
  });

  // Queue Modal Open/Close & Actions
  btnViewQueue.addEventListener('click', async () => {
    await renderQueueModal();
    queueModal.classList.remove('hidden');
  });

  closeQueueBtn.addEventListener('click', () => {
    queueModal.classList.add('hidden');
  });

  btnRetryAllQueue.addEventListener('click', async () => {
    btnRetryAllQueue.disabled = true;
    showFeedback('info', 'Retrying queued items with ECHO...');
    const resp = await sendMessageToWorker({ type: 'RETRY_QUEUE' });
    btnRetryAllQueue.disabled = false;
    await renderQueueModal();
    await refreshStatus();
    if (resp && resp.success && resp.result) {
      showFeedback('success', `Processed ${resp.result.processed || 0} queued item(s).`);
    }
  });

  btnClearAllQueue.addEventListener('click', async () => {
    if (confirm('Are you sure you want to clear all offline items?')) {
      await sendMessageToWorker({ type: 'CLEAR_QUEUE' });
      await renderQueueModal();
      await refreshStatus();
      showFeedback('info', 'Offline queue cleared.');
    }
  });

  // Action Buttons
  btnArticle.addEventListener('click', () => triggerCapture('EXTRACT_ARTICLE'));
  btnSelection.addEventListener('click', () => triggerCapture('EXTRACT_SELECTION'));
  btnBookmark.addEventListener('click', () => triggerCapture('EXTRACT_BOOKMARK'));

  // Duplicate Choices
  btnDupUpdate.addEventListener('click', () => resolveDuplicate('update'));
  btnDupCopy.addEventListener('click', () => resolveDuplicate('save_copy'));
  btnDupSkip.addEventListener('click', () => resolveDuplicate('skip'));

  // Queue Sync from strip
  btnSyncQueue.addEventListener('click', async () => {
    btnSyncQueue.disabled = true;
    showFeedback('info', 'Synchronizing offline queue with ECHO...');
    const resp = await sendMessageToWorker({ type: 'RETRY_QUEUE' });
    btnSyncQueue.disabled = false;
    await refreshStatus();
    if (resp && resp.success && resp.result) {
      showFeedback('success', `Processed ${resp.result.processed || 0} queued item(s).`);
    }
  });

  // Helpers
  async function initActiveTab() {
    try {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      currentTab = tab;
      if (tab) {
        pageTitle.textContent = tab.title || 'Untitled Web Page';
        pageTitle.title = tab.title || '';
        try {
          const urlObj = new URL(tab.url);
          pageDomain.textContent = urlObj.hostname;
          if (tab.favIconUrl) {
            pageFavicon.src = tab.favIconUrl;
          }
        } catch (_) {
          pageDomain.textContent = 'Current Page';
        }
      }
    } catch (e) {
      console.error('Failed to get active tab:', e);
    }
  }

  async function refreshStatus() {
    statusIndicator.className = 'status-badge status-offline';
    statusText.textContent = 'Connecting...';

    const resp = await sendMessageToWorker({ type: 'CHECK_STATUS' });
    if (!resp || !resp.success) {
      statusIndicator.className = 'status-badge status-offline';
      statusText.textContent = 'Offline';
      statusIndicator.title = resp?.error || 'ECHO background worker unavailable';
      return;
    }

    queueCount.textContent = resp.queueCount || 0;
    if (resp.failedCount > 0) {
      failedCount.textContent = resp.failedCount;
      failedBadge.classList.remove('hidden');
    } else {
      failedBadge.classList.add('hidden');
    }

    // Accurate State Machine Mapping:
    // 1. connected -> "Connected" / "Online (:port)"
    // 2. unpaired -> "Unpaired"
    // 3. invalid_token -> "Invalid Token"
    // 4. incomplete_config -> "Unpaired"
    // 5. offline -> "Offline"
    switch (resp.status) {
      case 'connected':
        statusIndicator.className = 'status-badge status-online';
        statusText.textContent = `Online (:${resp.port})`;
        statusIndicator.title = `Connected and authorized on port ${resp.port}`;
        break;

      case 'unpaired':
        statusIndicator.className = 'status-badge status-warn';
        statusText.textContent = 'Unpaired';
        statusIndicator.title = resp.error || 'Extension not authorized. Add Extension ID in ECHO Settings.';
        break;

      case 'invalid_token':
        statusIndicator.className = 'status-badge status-danger';
        statusText.textContent = 'Invalid Token';
        statusIndicator.title = resp.error || 'Pairing token invalid or expired. Check Clipper Settings.';
        break;

      case 'incomplete_config':
        statusIndicator.className = 'status-badge status-warn';
        statusText.textContent = 'Unpaired';
        statusIndicator.title = resp.error || 'Pairing configuration incomplete. Add token in Clipper Settings.';
        break;

      case 'offline':
      default:
        statusIndicator.className = 'status-badge status-offline';
        statusText.textContent = 'Offline';
        statusIndicator.title = resp.error || 'ECHO desktop application is offline or unreachable.';
        break;
    }
  }

  async function renderQueueModal() {
    queueListContainer.innerHTML = '';
    const resp = await sendMessageToWorker({ type: 'GET_QUEUE' });
    if (!resp || !resp.success || !Array.isArray(resp.queue) || resp.queue.length === 0) {
      queueListContainer.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 20px;">Queue is empty.</div>';
      return;
    }

    resp.queue.forEach(item => {
      const card = document.createElement('div');
      card.className = `queue-card-item ${item.status === 'failed' ? 'item-failed' : ''}`;

      const title = (item.payload && item.payload.title) ? item.payload.title : 'Untitled Clip';
      const statusLabel = item.status === 'failed' ? `<span style="color:#F87171; font-size:10px;">Failed (${item.attempts} retries)</span>` : `<span style="color:#38BDF8; font-size:10px;">Pending</span>`;

      card.innerHTML = `
        <div class="queue-item-header">
          <span class="queue-item-title" title="${title}">${title}</span>
          ${statusLabel}
        </div>
        ${item.last_error ? `<div class="queue-item-error">${item.last_error}</div>` : ''}
        <div class="queue-item-footer">
          <span style="font-size: 10px; color: #64748B;">Key: ${item.idempotency_key ? item.idempotency_key.substring(0, 8) : '...'}</span>
          <button class="btn-ghost btn-sm remove-item-btn" data-id="${item.id}" style="color: #F87171;">Remove</button>
        </div>
      `;

      card.querySelector('.remove-item-btn').addEventListener('click', async (e) => {
        const idToRemove = e.target.getAttribute('data-id');
        await sendMessageToWorker({ type: 'REMOVE_QUEUE_ITEM', id: idToRemove });
        await renderQueueModal();
        await refreshStatus();
      });

      queueListContainer.appendChild(card);
    });
  }

  async function triggerCapture(action) {
    if (!currentTab || !currentTab.id) {
      showFeedback('error', 'No active tab available for capture.');
      return;
    }

    duplicateView.classList.add('hidden');
    hideFeedback();
    setActionsDisabled(true);

    try {
      const extractionResponse = await chrome.tabs.sendMessage(currentTab.id, { action }).catch(async (err) => {
        await chrome.scripting.executeScript({
          target: { tabId: currentTab.id },
          files: ['lib/Readability.js', 'lib/turndown.js', 'content/extractor.js']
        });
        return await chrome.tabs.sendMessage(currentTab.id, { action });
      });

      if (!extractionResponse || !extractionResponse.success || !extractionResponse.data) {
        throw new Error(extractionResponse ? extractionResponse.error : 'Content extraction failed');
      }

      activePayload = extractionResponse.data;

      showFeedback('info', 'Saving to ECHO Knowledge...');
      const clipResp = await sendMessageToWorker({
        type: 'CLIP_NOW',
        payload: activePayload,
        duplicate_strategy: null
      });

      handleClipResponse(clipResp);
    } catch (err) {
      console.error('Capture failed:', err);
      showFeedback('error', err.message || 'Capture failed');
    } finally {
      setActionsDisabled(false);
      await refreshStatus();
    }
  }

  async function resolveDuplicate(strategy) {
    if (!activePayload) {
      showFeedback('error', 'No active capture to resolve.');
      return;
    }

    duplicateView.classList.add('hidden');
    setActionsDisabled(true);
    showFeedback('info', `Resolving duplicate (${strategy})...`);

    try {
      const clipResp = await sendMessageToWorker({
        type: 'CLIP_NOW',
        payload: activePayload,
        duplicate_strategy: strategy
      });
      handleClipResponse(clipResp);
    } catch (err) {
      showFeedback('error', err.message || 'Duplicate resolution failed');
    } finally {
      setActionsDisabled(false);
      await refreshStatus();
    }
  }

  function handleClipResponse(res) {
    if (!res) {
      showFeedback('error', 'No response received from background service worker.');
      return;
    }

    if (res.conflict) {
      duplicateView.classList.remove('hidden');
      duplicateDesc.textContent = res.message || 'This note already exists in ECHO. Choose an action:';
      hideFeedback();
      return;
    }

    if (res.queued) {
      showFeedback('info', 'ECHO is offline. Capture saved to offline queue!');
      return;
    }

    if (res.success) {
      const verb = res.action === 'updated' ? 'Updated' : (res.action === 'skipped' ? 'Skipped' : 'Saved');
      showFeedback('success', `${verb} successfully to ECHO!`);
    } else {
      showFeedback('error', res.error || 'Failed to clip content.');
    }
  }

  function setActionsDisabled(disabled) {
    btnArticle.disabled = disabled;
    btnSelection.disabled = disabled;
    btnBookmark.disabled = disabled;
  }

  function showFeedback(type, message) {
    feedbackView.className = `feedback feedback-${type}`;
    feedbackText.textContent = message;
    feedbackView.classList.remove('hidden');
  }

  function hideFeedback() {
    feedbackView.classList.add('hidden');
  }

  function sendMessageToWorker(msg) {
    return new Promise((resolve) => {
      const timeoutId = setTimeout(() => {
        resolve({ success: false, error: 'Worker response timed out' });
      }, 3000);

      try {
        chrome.runtime.sendMessage(msg, (response) => {
          clearTimeout(timeoutId);
          if (chrome.runtime.lastError) {
            resolve({ success: false, error: chrome.runtime.lastError.message });
          } else {
            resolve(response);
          }
        });
      } catch (err) {
        clearTimeout(timeoutId);
        resolve({ success: false, error: err.message });
      }
    });
  }
});
