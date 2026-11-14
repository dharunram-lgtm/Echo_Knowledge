// ECHO Web Clipper - Content Extractor Script
// Operates in isolated world on document_idle

(function () {
  'use strict';

  // Listen for extraction requests from Popup or Background Service Worker
  chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (!request || !request.action) {
      return false;
    }

    try {
      if (request.action === 'PING') {
        sendResponse({ status: 'PONG', ready: true });
        return true;
      }

      if (request.action === 'EXTRACT_ARTICLE') {
        const result = extractArticle();
        sendResponse({ success: true, data: result });
        return true;
      }

      if (request.action === 'EXTRACT_SELECTION') {
        const result = extractSelection();
        sendResponse({ success: true, data: result });
        return true;
      }

      if (request.action === 'EXTRACT_BOOKMARK') {
        const result = extractBookmark();
        sendResponse({ success: true, data: result });
        return true;
      }

      sendResponse({ success: false, error: 'Unknown extraction action' });
      return true;
    } catch (err) {
      console.error('[ECHO Clipper Extractor] Extraction error:', err);
      sendResponse({ success: false, error: err.message || 'Extraction failed' });
      return true;
    }
  });

  // Helper to get meta content
  function getMetaContent(selector) {
    const el = document.querySelector(selector);
    return el ? (el.getAttribute('content') || '').trim() : '';
  }

  // Helper to extract favicon URL
  function extractFaviconUrl() {
    const faviconSelectors = [
      'link[rel="icon"]',
      'link[rel="shortcut icon"]',
      'link[rel="apple-touch-icon"]',
      'link[rel="apple-touch-icon-precomposed"]'
    ];
    for (const sel of faviconSelectors) {
      const el = document.querySelector(sel);
      if (el && el.getAttribute('href')) {
        const href = el.getAttribute('href').trim();
        try {
          return new URL(href, window.location.href).href;
        } catch (_) {}
      }
    }
    try {
      return new URL('/favicon.ico', window.location.origin).href;
    } catch (_) {
      return null;
    }
  }

  // Helper to extract author
  function extractAuthor() {
    const authorMeta = [
      'meta[name="author"]',
      'meta[property="article:author"]',
      'meta[name="twitter:creator"]',
      'meta[property="og:article:author"]',
      'meta[name="dc.creator"]'
    ];
    for (const sel of authorMeta) {
      const val = getMetaContent(sel);
      if (val) return val;
    }
    const bylineEl = document.querySelector('.byline, .author, [rel="author"], [itemprop="author"]');
    if (bylineEl) {
      const text = (bylineEl.textContent || '').trim().replace(/^by\s+/i, '');
      if (text && text.length < 100) return text;
    }
    return null;
  }

  // Helper to get clean Turndown service instance
  function getTurndownService() {
    if (typeof TurndownService === 'undefined') {
      throw new Error('Turndown library is not loaded');
    }
    const td = new TurndownService({
      headingStyle: 'atx',
      hr: '---',
      bulletListMarker: '-',
      codeBlockStyle: 'fenced',
      emDelimiter: '*'
    });

    // Remove unwanted elements
    td.remove(['script', 'style', 'noscript', 'iframe', 'canvas', 'svg', 'button', 'form', 'nav', 'footer']);

    return td;
  }

  // Full Article Extraction
  function extractArticle() {
    const url = window.location.href;
    const title = (document.title || 'Untitled Web Note').trim();
    const faviconUrl = extractFaviconUrl();
    const author = extractAuthor();

    // Clone document to avoid mutating live webpage DOM
    const docClone = document.cloneNode(true);

    let article = null;
    if (typeof Readability !== 'undefined') {
      try {
        const reader = new Readability(docClone, {
          charThreshold: 80,
          keepClasses: false
        });
        article = reader.parse();
      } catch (e) {
        console.warn('[ECHO Clipper Extractor] Readability parse failed, falling back to body:', e);
      }
    }

    let markdown = '';
    let text = '';
    const td = getTurndownService();

    if (article && article.content) {
      markdown = td.turndown(article.content);
      text = (article.textContent || '').trim();
    } else {
      // Fallback: extract main or body
      const mainContent = document.querySelector('article, main, #content, .content') || document.body;
      markdown = td.turndown(mainContent.innerHTML);
      text = (mainContent.innerText || mainContent.textContent || '').trim();
    }

    const finalTitle = (article && article.title ? article.title.trim() : title) || 'Untitled Web Note';

    return {
      mode: 'article',
      url: url,
      title: finalTitle,
      author: article && article.byline ? article.byline.trim() : author,
      favicon_url: faviconUrl,
      content_markdown: markdown,
      extracted_text: text,
      metadata: {
        mode: 'article',
        domain: window.location.hostname,
        captured_at: new Date().toISOString(),
        site_name: article && article.siteName ? article.siteName : (getMetaContent('meta[property="og:site_name"]') || window.location.hostname),
        excerpt: article && article.excerpt ? article.excerpt : getMetaContent('meta[name="description"]')
      }
    };
  }

  // Selection Capture
  function extractSelection() {
    const url = window.location.href;
    const title = (document.title || 'Untitled Web Note').trim();
    const faviconUrl = extractFaviconUrl();
    const author = extractAuthor();

    const sel = window.getSelection();
    let selectedHtml = '';
    let selectedText = '';

    if (sel && sel.rangeCount > 0) {
      const container = document.createElement('div');
      for (let i = 0; i < sel.rangeCount; i++) {
        container.appendChild(sel.getRangeAt(i).cloneContents());
      }
      selectedHtml = container.innerHTML;
      selectedText = (sel.toString() || '').trim();
    }

    if (!selectedText) {
      throw new Error('No text currently selected on this page. Please select text before clipping.');
    }

    const td = getTurndownService();
    const markdownBody = td.turndown(selectedHtml);

    // Prefix selection with blockquote formatting or header
    const formattedMarkdown = `> ${markdownBody.split('\n').join('\n> ')}\n\n_Captured selection from [${title}](${url})_`;

    return {
      mode: 'selection',
      url: url,
      title: `${title} (Selection)`,
      author: author,
      favicon_url: faviconUrl,
      content_markdown: formattedMarkdown,
      extracted_text: selectedText,
      metadata: {
        mode: 'selection',
        domain: window.location.hostname,
        captured_at: new Date().toISOString(),
        site_name: getMetaContent('meta[property="og:site_name"]') || window.location.hostname
      }
    };
  }

  // Bookmark Capture
  function extractBookmark() {
    const url = window.location.href;
    const title = (document.title || 'Untitled Web Bookmark').trim();
    const faviconUrl = extractFaviconUrl();
    const author = extractAuthor();
    const description = getMetaContent('meta[name="description"]') || getMetaContent('meta[property="og:description"]');
    const siteName = getMetaContent('meta[property="og:site_name"]') || window.location.hostname;

    // Distinct bookmark Markdown representation
    let bookmarkMarkdown = `# Bookmark: [${title}](${url})\n\n`;
    bookmarkMarkdown += `- **URL:** ${url}\n`;
    bookmarkMarkdown += `- **Domain:** ${window.location.hostname}\n`;
    if (author) bookmarkMarkdown += `- **Author:** ${author}\n`;
    bookmarkMarkdown += `- **Saved At:** ${new Date().toLocaleString()}\n\n`;
    if (description) {
      bookmarkMarkdown += `### Description\n\n> ${description}\n`;
    }

    const extractedText = `Bookmark: ${title}\nURL: ${url}\nDomain: ${window.location.hostname}\n${description ? 'Description: ' + description : ''}`;

    return {
      mode: 'bookmark',
      url: url,
      title: `${title} (Bookmark)`,
      author: author,
      favicon_url: faviconUrl,
      content_markdown: bookmarkMarkdown,
      extracted_text: extractedText,
      metadata: {
        mode: 'bookmark',
        domain: window.location.hostname,
        captured_at: new Date().toISOString(),
        site_name: siteName,
        description: description || null
      }
    };
  }
})();
