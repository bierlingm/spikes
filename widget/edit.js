// Spikes Edit Mode — Inline copy editing with auto-save
// https://spikes.sh
//
// Adds contenteditable to text elements, auto-saves to localStorage,
// and exposes edits as structured JSON for AI agents.
//
// Activated via data-edit="true" on the spikes script tag, or
// programmatically via window.Spikes.toggleEditMode().
(function() {
    'use strict';

    // Wait for Spikes to be available
    function waitForSpikes(cb) {
        if (window.Spikes && window.Spikes.config) return cb();
        var interval = setInterval(function() {
            if (window.Spikes && window.Spikes.config) {
                clearInterval(interval);
                cb();
            }
        }, 50);
    }

    waitForSpikes(function() {
        var config = window.Spikes.config;
        var STORAGE_KEY = 'spikes-edits:' + config.project;

        // State
        var editMode = false;
        var originals = {};    // selector -> original innerHTML
        var editables = [];    // elements we made editable
        var editCount = 0;
        var editPill = null;
        var editBtn = null;

        // Tags that should be editable
        var EDITABLE_TAGS = {
            H1:1, H2:1, H3:1, H4:1, H5:1, H6:1,
            P:1, SPAN:1, A:1, LI:1, TD:1, TH:1,
            LABEL:1, STRONG:1, EM:1, SMALL:1,
            FIGCAPTION:1, BLOCKQUOTE:1, DT:1, DD:1, SUMMARY:1
        };

        function isWidgetElement(el) {
            if (!el) return false;
            return !!(el.closest && (
                el.closest('#spikes-container') ||
                el.closest('#spikes-modal') ||
                el.closest('#spikes-popover') ||
                el.closest('#spikes-review-bar')
            ));
        }

        function shouldEdit(el) {
            if (!el || !el.tagName) return false;
            if (el.tagName === 'SCRIPT' || el.tagName === 'STYLE') return false;
            if (isWidgetElement(el)) return false;
            if (!el.textContent || !el.textContent.trim()) return false;

            // Skip elements whose children are themselves editable tags
            var children = el.children;
            for (var i = 0; i < children.length; i++) {
                if (EDITABLE_TAGS[children[i].tagName]) return false;
            }

            return !!EDITABLE_TAGS[el.tagName];
        }

        // Build a stable CSS selector for an element
        function getSelector(el) {
            var parts = [];
            var current = el;
            while (current && current !== document.body) {
                var sel = current.tagName.toLowerCase();
                if (current.id && current.id.indexOf('spikes') === -1) {
                    sel += '#' + current.id;
                    parts.unshift(sel);
                    break;
                }
                if (current.className && typeof current.className === 'string') {
                    var classes = current.className.split(/\s+/);
                    var filtered = [];
                    for (var i = 0; i < classes.length; i++) {
                        if (classes[i] && classes[i].indexOf('spikes') === -1) {
                            filtered.push(classes[i]);
                            if (filtered.length === 2) break;
                        }
                    }
                    if (filtered.length) sel += '.' + filtered.join('.');
                }
                var parent = current.parentElement;
                if (parent) {
                    var siblings = [];
                    for (var j = 0; j < parent.children.length; j++) {
                        if (parent.children[j].tagName === current.tagName) {
                            siblings.push(parent.children[j]);
                        }
                    }
                    if (siblings.length > 1) {
                        for (var k = 0; k < siblings.length; k++) {
                            if (siblings[k] === current) {
                                sel += ':nth-of-type(' + (k + 1) + ')';
                                break;
                            }
                        }
                    }
                }
                parts.unshift(sel);
                current = current.parentElement;
            }
            return parts.join(' > ');
        }

        function enterEditMode() {
            if (editMode) return;
            editMode = true;

            // Exit spike mode if active
            if (window.Spikes._exitSpikeMode) {
                window.Spikes._exitSpikeMode();
            }

            // Walk DOM and make text elements editable
            var all = document.querySelectorAll('*');
            for (var i = 0; i < all.length; i++) {
                var el = all[i];
                if (shouldEdit(el)) {
                    el.setAttribute('contenteditable', 'true');
                    el.setAttribute('spellcheck', 'true');
                    var selector = getSelector(el);
                    if (!originals[selector]) {
                        originals[selector] = el.innerHTML;
                    }
                    el.addEventListener('input', handleInput);

                    // Prevent link navigation while editing
                    if (el.tagName === 'A') {
                        el.addEventListener('click', preventNavigation);
                    }

                    editables.push(el);
                }
            }

            // Restore any saved edits
            restoreEdits();

            // Update UI
            updateEditButton();
            injectEditStyles();
        }

        function exitEditMode() {
            if (!editMode) return;
            editMode = false;

            // Remove contenteditable from all elements
            for (var i = 0; i < editables.length; i++) {
                var el = editables[i];
                el.removeAttribute('contenteditable');
                el.removeAttribute('spellcheck');
                el.removeEventListener('input', handleInput);
                if (el.tagName === 'A') {
                    el.removeEventListener('click', preventNavigation);
                }
            }

            // Update UI
            updateEditButton();
            removeEditStyles();
        }

        function preventNavigation(e) {
            e.preventDefault();
        }

        function handleInput(e) {
            var el = e.target;
            var selector = getSelector(el);
            var original = originals[selector];

            if (el.innerHTML !== original) {
                el.setAttribute('data-spikes-edited', 'true');
            } else {
                el.removeAttribute('data-spikes-edited');
            }

            updateEditCount();
            persistEdits();
        }

        function updateEditCount() {
            var edited = document.querySelectorAll('[data-spikes-edited]');
            editCount = edited.length;
            updateEditPill();
            updateToolbarStatus();
        }

        function getChanges() {
            var changes = [];
            var edited = document.querySelectorAll('[data-spikes-edited]');
            for (var i = 0; i < edited.length; i++) {
                var el = edited[i];
                var selector = getSelector(el);
                var original = originals[selector] || '';
                changes.push({
                    selector: selector,
                    original: original.replace(/<[^>]+>/g, '').trim(),
                    updated: el.textContent.trim(),
                    html_original: original,
                    html_updated: el.innerHTML,
                    tag: el.tagName.toLowerCase()
                });
            }
            return changes;
        }

        // --- Persistence ---

        function persistEdits() {
            var edits = {};
            var edited = document.querySelectorAll('[data-spikes-edited]');
            for (var i = 0; i < edited.length; i++) {
                var sel = getSelector(edited[i]);
                edits[sel] = edited[i].innerHTML;
            }
            try {
                localStorage.setItem(STORAGE_KEY, JSON.stringify(edits));
            } catch(e) {
                // localStorage full or unavailable
            }
        }

        function restoreEdits() {
            var stored;
            try {
                stored = localStorage.getItem(STORAGE_KEY);
            } catch(e) {
                return;
            }
            if (!stored) return;

            var edits;
            try {
                edits = JSON.parse(stored);
            } catch(e) {
                return;
            }

            var keys = Object.keys(edits);
            if (!keys.length) return;

            var restored = 0;
            for (var i = 0; i < editables.length; i++) {
                var el = editables[i];
                var sel = getSelector(el);
                if (edits[sel] && edits[sel] !== originals[sel]) {
                    el.innerHTML = edits[sel];
                    el.setAttribute('data-spikes-edited', 'true');
                    restored++;
                }
            }

            if (restored > 0) {
                updateEditCount();
            }
        }

        function clearEdits() {
            var edited = document.querySelectorAll('[data-spikes-edited]');
            for (var i = 0; i < edited.length; i++) {
                var sel = getSelector(edited[i]);
                var original = originals[sel];
                if (original !== undefined) {
                    edited[i].innerHTML = original;
                }
                edited[i].removeAttribute('data-spikes-edited');
            }
            try {
                localStorage.removeItem(STORAGE_KEY);
            } catch(e) {}
            editCount = 0;
            updateEditPill();
        }

        // --- UI ---

        var styleEl = null;

        function injectEditStyles() {
            if (styleEl) return;
            styleEl = document.createElement('style');
            styleEl.id = 'spikes-edit-styles';
            styleEl.textContent = [
                '[contenteditable="true"] {',
                '  outline: none;',
                '  transition: box-shadow 0.15s;',
                '}',
                '[contenteditable="true"]:hover {',
                '  box-shadow: inset 0 0 0 1px rgba(212, 160, 74, 0.3);',
                '  cursor: text;',
                '}',
                '[contenteditable="true"]:focus {',
                '  box-shadow: inset 0 0 0 2px rgba(212, 160, 74, 0.6);',
                '}',
                '[data-spikes-edited] {',
                '  box-shadow: inset 0 0 0 2px rgba(100, 200, 100, 0.5) !important;',
                '  position: relative;',
                '}',
                '[data-spikes-edited]::after {',
                '  content: "edited";',
                '  position: absolute;',
                '  top: -8px;',
                '  right: -4px;',
                '  font-size: 9px;',
                '  color: #0a0a0a;',
                '  background: #64c864;',
                '  padding: 1px 5px;',
                '  border-radius: 3px;',
                '  font-family: ui-monospace, "SF Mono", Monaco, monospace;',
                '  pointer-events: none;',
                '}',
                '#spikes-edit-toolbar {',
                '  position: fixed;',
                '  bottom: 24px;',
                '  left: 50%;',
                '  transform: translateX(-50%);',
                '  z-index: 2147483645;',
                '  display: flex;',
                '  gap: 8px;',
                '  padding: 10px 16px;',
                '  background: rgba(20, 20, 20, 0.95);',
                '  backdrop-filter: blur(12px);',
                '  -webkit-backdrop-filter: blur(12px);',
                '  border: 1px solid #333;',
                '  border-radius: 12px;',
                '  box-shadow: 0 8px 32px rgba(0,0,0,0.5);',
                '  font-family: ui-monospace, "SF Mono", Monaco, monospace;',
                '  font-size: 13px;',
                '}',
                '#spikes-edit-toolbar button {',
                '  padding: 8px 16px;',
                '  border: 1px solid #444;',
                '  border-radius: 6px;',
                '  background: #1a1a1a;',
                '  color: #ccc;',
                '  font-family: inherit;',
                '  font-size: 12px;',
                '  cursor: pointer;',
                '  transition: all 0.15s;',
                '  white-space: nowrap;',
                '}',
                '#spikes-edit-toolbar button:hover {',
                '  background: #2a2a2a;',
                '  border-color: #d4a04a;',
                '  color: #d4a04a;',
                '}',
                '#spikes-edit-toolbar .spikes-edit-status {',
                '  color: #666;',
                '  font-size: 11px;',
                '  display: flex;',
                '  align-items: center;',
                '  padding: 0 8px;',
                '}',
                '#spikes-edit-toolbar .spikes-edit-divider {',
                '  width: 1px;',
                '  background: #333;',
                '  margin: 0 4px;',
                '}',
                '#spikes-diff-modal {',
                '  display: none;',
                '  position: fixed;',
                '  inset: 0;',
                '  z-index: 2147483647;',
                '  background: rgba(0,0,0,0.8);',
                '  backdrop-filter: blur(4px);',
                '}',
                '#spikes-diff-modal.visible {',
                '  display: flex;',
                '  align-items: center;',
                '  justify-content: center;',
                '}',
                '#spikes-diff-content {',
                '  background: #111;',
                '  border: 1px solid #333;',
                '  border-radius: 12px;',
                '  width: 90vw;',
                '  max-width: 900px;',
                '  max-height: 80vh;',
                '  overflow: auto;',
                '  padding: 24px;',
                '  font-family: ui-monospace, "SF Mono", Monaco, monospace;',
                '  font-size: 13px;',
                '  color: #ccc;',
                '}',
                '#spikes-diff-content h3 { color: #d4a04a; margin-bottom: 16px; }',
                '#spikes-diff-content .diff-entry { margin-bottom: 16px; padding: 12px; background: #1a1a1a; border-radius: 8px; }',
                '#spikes-diff-content .diff-selector { color: #666; font-size: 11px; margin-bottom: 6px; }',
                '#spikes-diff-content .diff-old { color: #e06060; }',
                '#spikes-diff-content .diff-new { color: #60c060; }',
                '#spikes-diff-content .diff-old::before { content: "- "; }',
                '#spikes-diff-content .diff-new::before { content: "+ "; }',
                '#spikes-diff-modal .close-modal {',
                '  position: absolute;',
                '  top: 20px;',
                '  right: 20px;',
                '  background: none;',
                '  border: none;',
                '  color: #666;',
                '  font-size: 24px;',
                '  cursor: pointer;',
                '}'
            ].join('\n');
            document.head.appendChild(styleEl);
            createToolbar();
        }

        function removeEditStyles() {
            if (styleEl) {
                styleEl.remove();
                styleEl = null;
            }
            removeToolbar();
        }

        // --- Toolbar + Diff Modal ---

        var toolbar = null;
        var diffModal = null;
        var statusEl = null;

        function createToolbar() {
            if (toolbar) return;

            toolbar = document.createElement('div');
            toolbar.id = 'spikes-edit-toolbar';

            statusEl = document.createElement('span');
            statusEl.className = 'spikes-edit-status';
            statusEl.textContent = '0 changes';

            var div1 = document.createElement('div');
            div1.className = 'spikes-edit-divider';

            var diffBtn = document.createElement('button');
            diffBtn.textContent = 'Diff';
            diffBtn.title = 'Show what changed';
            diffBtn.onclick = showDiff;

            var exportBtn = document.createElement('button');
            exportBtn.textContent = 'Export JSON';
            exportBtn.title = 'Export edits as JSON for AI agents';
            exportBtn.onclick = exportJSON;

            var resetBtn = document.createElement('button');
            resetBtn.textContent = 'Reset';
            resetBtn.title = 'Revert all edits';
            resetBtn.onclick = function() {
                if (confirm('Revert all edits?')) clearEdits();
            };

            toolbar.appendChild(statusEl);
            toolbar.appendChild(div1);
            toolbar.appendChild(diffBtn);
            toolbar.appendChild(exportBtn);
            toolbar.appendChild(resetBtn);

            // Diff modal
            diffModal = document.createElement('div');
            diffModal.id = 'spikes-diff-modal';
            var closeBtn = document.createElement('button');
            closeBtn.className = 'close-modal';
            closeBtn.innerHTML = '&times;';
            closeBtn.onclick = closeDiff;
            var diffContent = document.createElement('div');
            diffContent.id = 'spikes-diff-content';
            diffModal.appendChild(closeBtn);
            diffModal.appendChild(diffContent);

            document.body.appendChild(toolbar);
            document.body.appendChild(diffModal);
        }

        function removeToolbar() {
            if (toolbar) { toolbar.remove(); toolbar = null; }
            if (diffModal) { diffModal.remove(); diffModal = null; }
            statusEl = null;
        }

        function updateToolbarStatus() {
            if (statusEl) {
                statusEl.textContent = editCount + ' change' + (editCount !== 1 ? 's' : '');
            }
        }

        function escHtml(s) {
            return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
        }

        function showDiff() {
            var changes = getChanges();
            var content = document.getElementById('spikes-diff-content');
            if (!content) return;

            if (changes.length === 0) {
                content.innerHTML = '<h3>No Changes</h3><p style="color:#666">Click any text on the page to edit it inline.</p>';
            } else {
                var html = '<h3>' + changes.length + ' Change' + (changes.length !== 1 ? 's' : '') + '</h3>';
                for (var i = 0; i < changes.length; i++) {
                    var c = changes[i];
                    html += '<div class="diff-entry">';
                    html += '<div class="diff-selector">' + escHtml(c.selector) + ' &lt;' + c.tag + '&gt;</div>';
                    html += '<div class="diff-old">' + escHtml(c.original) + '</div>';
                    html += '<div class="diff-new">' + escHtml(c.updated) + '</div>';
                    html += '</div>';
                }
                content.innerHTML = html;
            }
            diffModal.classList.add('visible');
        }

        function closeDiff() {
            if (diffModal) diffModal.classList.remove('visible');
        }

        function exportJSON() {
            var changes = getChanges();
            var spikesFeedback = [];
            try {
                if (window.Spikes && window.Spikes.getSpikes) {
                    spikesFeedback = window.Spikes.getSpikes();
                }
            } catch(e) {}

            var data = {
                timestamp: new Date().toISOString(),
                copy_changes: changes,
                spikes_feedback: spikesFeedback
            };

            if (changes.length === 0 && spikesFeedback.length === 0) {
                alert('No changes or feedback to export.');
                return;
            }

            if (navigator.clipboard) {
                navigator.clipboard.writeText(JSON.stringify(data, null, 2));
                alert('Copied ' + (changes.length + spikesFeedback.length) + ' items to clipboard.\nPaste into your AI agent.');
            }
        }

        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {
            if (e.metaKey && e.key === 's' && editMode) {
                e.preventDefault();
                exportJSON();
            }
            if (e.key === 'Escape') {
                closeDiff();
            }
        });

        function createEditButton() {
            var container = document.getElementById('spikes-container');
            if (!container) return;

            editBtn = document.createElement('button');
            editBtn.id = 'spikes-edit-btn';
            editBtn.innerHTML = '✎';
            editBtn.setAttribute('aria-label', 'Toggle Edit Mode');
            editBtn.setAttribute('title', 'Toggle edit mode — click text to edit inline');

            var theme = (window.Spikes.config && window.Spikes.config.theme === 'light')
                ? { bgCard: '#f8f9fa', text: '#1a1a1a', border: '#e0e0e0' }
                : { bgCard: '#141417', text: '#fafafa', border: '#27272a' };

            editBtn.style.cssText = [
                'width:36px',
                'height:36px',
                'background:' + theme.bgCard,
                'color:' + theme.text,
                'border:1px solid ' + theme.border,
                'border-radius:6px',
                'font-size:16px',
                'cursor:pointer',
                'transition:all 0.15s',
                'display:flex',
                'align-items:center',
                'justify-content:center',
                'font-family:ui-monospace,SF Mono,Monaco,monospace'
            ].join(';');

            editBtn.onmouseenter = function() {
                editBtn.style.borderColor = config.color;
                editBtn.style.color = config.color;
            };
            editBtn.onmouseleave = function() {
                if (!editMode) {
                    editBtn.style.borderColor = theme.border;
                    editBtn.style.color = theme.text;
                }
            };
            editBtn.onclick = function(e) {
                e.stopPropagation();
                toggleEditMode();
            };

            // Edit count pill
            editPill = document.createElement('span');
            editPill.id = 'spikes-edit-pill';
            editPill.style.cssText = [
                'display:none',
                'background:#22c55e',
                'color:#0a0a0a',
                'font-size:10px',
                'font-weight:600',
                'padding:1px 6px',
                'border-radius:10px',
                'font-family:ui-monospace,SF Mono,Monaco,monospace',
                'white-space:nowrap'
            ].join(';');

            // Insert before reviewer indicator
            var reviewer = document.getElementById('spikes-reviewer');
            if (reviewer) {
                container.insertBefore(editPill, reviewer);
                container.insertBefore(editBtn, editPill);
            } else {
                container.appendChild(editBtn);
                container.appendChild(editPill);
            }
        }

        function updateEditButton() {
            if (!editBtn) return;
            var theme = (config.theme === 'light')
                ? { bgCard: '#f8f9fa', text: '#1a1a1a', border: '#e0e0e0' }
                : { bgCard: '#141417', text: '#fafafa', border: '#27272a' };

            if (editMode) {
                editBtn.style.background = config.color;
                editBtn.style.color = 'white';
                editBtn.style.borderColor = config.color;
                editBtn.setAttribute('title', 'Exit edit mode');
            } else {
                editBtn.style.background = theme.bgCard;
                editBtn.style.color = theme.text;
                editBtn.style.borderColor = theme.border;
                editBtn.setAttribute('title', 'Toggle edit mode — click text to edit inline');
            }
        }

        function updateEditPill() {
            if (!editPill) return;
            if (editCount > 0) {
                editPill.style.display = 'block';
                editPill.textContent = editCount + ' edit' + (editCount !== 1 ? 's' : '');
            } else {
                editPill.style.display = 'none';
            }
        }

        function toggleEditMode() {
            if (editMode) {
                exitEditMode();
            } else {
                enterEditMode();
            }
        }

        // --- Init ---

        function init() {
            createEditButton();

            // Check for persisted edits and show pill even before entering edit mode
            try {
                var stored = localStorage.getItem(STORAGE_KEY);
                if (stored) {
                    var edits = JSON.parse(stored);
                    var count = Object.keys(edits).length;
                    if (count > 0) {
                        editCount = count;
                        updateEditPill();
                    }
                }
            } catch(e) {}

            // Auto-enter edit mode if configured
            if (config.edit === 'true' || config.edit === 'auto') {
                enterEditMode();
            }
        }

        init();

        // --- Public API (extend window.Spikes) ---

        window.Spikes.isEditMode = function() { return editMode; };
        window.Spikes.toggleEditMode = toggleEditMode;
        window.Spikes.enterEditMode = enterEditMode;
        window.Spikes.exitEditMode = exitEditMode;
        window.Spikes.getEdits = getChanges;
        window.Spikes.clearEdits = clearEdits;

        // Internal hook for spike mode coordination
        window.Spikes._exitEditMode = exitEditMode;
    });
})();
