// Spikes — Drop-in feedback collection widget
// https://spikes.sh
(function() {
    'use strict';

    // Inline nanoid implementation (21 chars, URL-safe)
    var nanoid = (function() {
        var urlAlphabet = 'useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict';
        return function(size) {
            size = size || 21;
            var id = '';
            var bytes = crypto.getRandomValues(new Uint8Array(size));
            while (size--) {
                id += urlAlphabet[bytes[size] & 63];
            }
            return id;
        };
    })();

    // Get configuration from script tag
    var script = document.currentScript || (function() {
        var scripts = document.getElementsByTagName('script');
        return scripts[scripts.length - 1];
    })();

    var config = {
        project: script.getAttribute('data-project') || location.hostname || 'local',
        position: script.getAttribute('data-position') || 'bottom-right',
        color: script.getAttribute('data-color') || '#e74c3c',
        presetReviewer: script.getAttribute('data-reviewer') || null,
        endpoint: script.getAttribute('data-endpoint') || null
    };

    // Storage keys
    var STORAGE_KEY = 'spikes:' + config.project;
    var REVIEWER_KEY = 'spikes:reviewer';

    // State
    var selectedRating = null;
    var modal = null;
    var btn = null;
    var popover = null;
    var reviewerIndicator = null;
    
    // Spike mode state: 'idle' | 'armed' | 'capturing'
    var spikeMode = 'idle';
    var highlightedElement = null;
    var capturedElement = null;
    var capturedElementData = null;
    
    // Reviewer state
    var currentReviewer = null;
    var pendingSaveCallback = null;

    // Position mapping
    var positions = {
        'bottom-right': { bottom: '20px', right: '20px' },
        'bottom-left': { bottom: '20px', left: '20px' },
        'top-right': { top: '20px', right: '20px' },
        'top-left': { top: '20px', left: '20px' }
    };

    function getPositionStyles() {
        var pos = positions[config.position] || positions['bottom-right'];
        return Object.keys(pos).map(function(k) {
            return k + ':' + pos[k];
        }).join(';');
    }

    function loadSpikes() {
        try {
            return JSON.parse(localStorage.getItem(STORAGE_KEY)) || [];
        } catch (e) {
            return [];
        }
    }

    function saveSpike(spike) {
        var spikes = loadSpikes();
        spikes.push(spike);
        localStorage.setItem(STORAGE_KEY, JSON.stringify(spikes));

        // POST to configured endpoint (if any) or local server (if HTTP)
        var postUrl = null;
        
        if (config.endpoint) {
            // Use configured endpoint (includes token in URL)
            postUrl = config.endpoint;
        } else if (location.protocol === 'http:' || location.protocol === 'https:') {
            // Fall back to local server
            postUrl = '/spikes';
        }
        
        if (postUrl) {
            try {
                var xhr = new XMLHttpRequest();
                xhr.open('POST', postUrl, true);
                xhr.setRequestHeader('Content-Type', 'application/json');
                xhr.send(JSON.stringify(spike));
            } catch (e) {
                // Silently fail - localStorage already has the spike
            }
        }
    }
    
    // Reviewer management (N8, N9, N10)
    function loadReviewer() {
        try {
            var stored = localStorage.getItem(REVIEWER_KEY);
            if (stored) {
                return JSON.parse(stored);
            }
        } catch (e) {
            // ignore
        }
        return null;
    }
    
    function saveReviewer(reviewer) {
        localStorage.setItem(REVIEWER_KEY, JSON.stringify(reviewer));
        currentReviewer = reviewer;
        updateReviewerIndicator();
    }
    
    function createReviewer(name) {
        var reviewer = {
            id: nanoid(),
            name: name,
            createdAt: new Date().toISOString()
        };
        saveReviewer(reviewer);
        return reviewer;
    }
    
    function initReviewer() {
        currentReviewer = loadReviewer();
        
        // If preset via data-reviewer attribute and no existing reviewer, create one
        if (!currentReviewer && config.presetReviewer) {
            currentReviewer = createReviewer(config.presetReviewer);
        }
    }
    
    function getReviewerForSpike() {
        if (currentReviewer) {
            return {
                id: currentReviewer.id,
                name: currentReviewer.name
            };
        }
        return { id: 'anon', name: 'Anonymous' };
    }
    
    function updateReviewerIndicator() {
        if (!reviewerIndicator) return;
        
        if (currentReviewer) {
            reviewerIndicator.innerHTML = '<span style="color:#666;font-size:11px;">Reviewing as: </span><span style="color:#333;font-size:11px;font-weight:500;cursor:pointer;text-decoration:underline dotted;" title="Click to change">' + escapeHtml(currentReviewer.name) + '</span>';
            reviewerIndicator.style.display = 'block';
        } else {
            reviewerIndicator.style.display = 'none';
        }
    }
    
    // Name prompt UI (U9, U11)
    function showNamePrompt(targetContainer, onComplete) {
        var promptDiv = document.createElement('div');
        promptDiv.id = 'spikes-name-prompt';
        promptDiv.style.cssText = [
            'background:#fffbeb',
            'border:1px solid #fbbf24',
            'border-radius:8px',
            'padding:16px',
            'margin-bottom:16px'
        ].join(';');
        
        promptDiv.innerHTML = [
            '<div style="font-size:14px;font-weight:500;color:#92400e;margin-bottom:8px;">What should we call you?</div>',
            '<div style="display:flex;gap:8px;">',
            '  <input id="spikes-name-input" type="text" placeholder="Enter your name" style="' + nameInputStyle() + '">',
            '  <button id="spikes-name-continue" style="' + nameContinueBtnStyle() + '">Continue</button>',
            '</div>'
        ].join('');
        
        // Insert at the beginning of the target container
        if (targetContainer.firstChild) {
            targetContainer.insertBefore(promptDiv, targetContainer.firstChild);
        } else {
            targetContainer.appendChild(promptDiv);
        }
        
        var input = promptDiv.querySelector('#spikes-name-input');
        var continueBtn = promptDiv.querySelector('#spikes-name-continue');
        
        input.focus();
        
        function handleContinue() {
            var name = input.value.trim();
            if (name) {
                createReviewer(name);
                promptDiv.remove();
                if (onComplete) onComplete();
            } else {
                input.style.borderColor = '#ef4444';
                input.focus();
            }
        }
        
        continueBtn.onclick = function(e) {
            e.stopPropagation();
            handleContinue();
        };
        
        input.onkeydown = function(e) {
            if (e.key === 'Enter') {
                e.preventDefault();
                handleContinue();
            }
        };
        
        return promptDiv;
    }
    
    function showChangeNamePrompt() {
        // Create a mini modal for changing name
        var overlay = document.createElement('div');
        overlay.id = 'spikes-name-overlay';
        overlay.style.cssText = [
            'position:fixed',
            'top:0',
            'left:0',
            'right:0',
            'bottom:0',
            'background:rgba(0,0,0,0.4)',
            'display:flex',
            'justify-content:center',
            'align-items:center',
            'z-index:2147483647'
        ].join(';');
        
        var dialog = document.createElement('div');
        dialog.style.cssText = [
            'background:white',
            'padding:24px',
            'border-radius:12px',
            'max-width:320px',
            'width:90%',
            'box-shadow:0 20px 60px rgba(0,0,0,0.3)',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif'
        ].join(';');
        
        dialog.innerHTML = [
            '<div style="font-size:16px;font-weight:600;color:#1a1a1a;margin-bottom:16px;">Change Your Name</div>',
            '<input id="spikes-change-name-input" type="text" placeholder="Enter new name" value="' + escapeHtml(currentReviewer ? currentReviewer.name : '') + '" style="' + nameInputStyle() + 'width:100%;margin-bottom:16px;">',
            '<div style="display:flex;gap:8px;">',
            '  <button id="spikes-change-name-save" style="' + saveBtnStyle() + '">Save</button>',
            '  <button id="spikes-change-name-cancel" style="' + cancelBtnStyle() + '">Cancel</button>',
            '</div>'
        ].join('');
        
        overlay.appendChild(dialog);
        document.body.appendChild(overlay);
        
        var input = dialog.querySelector('#spikes-change-name-input');
        input.focus();
        input.select();
        
        function closeDialog() {
            overlay.remove();
        }
        
        dialog.querySelector('#spikes-change-name-cancel').onclick = closeDialog;
        
        overlay.onclick = function(e) {
            if (e.target === overlay) closeDialog();
        };
        
        dialog.querySelector('#spikes-change-name-save').onclick = function() {
            var name = input.value.trim();
            if (name) {
                if (currentReviewer) {
                    currentReviewer.name = name;
                    saveReviewer(currentReviewer);
                } else {
                    createReviewer(name);
                }
                closeDialog();
            } else {
                input.style.borderColor = '#ef4444';
                input.focus();
            }
        };
        
        input.onkeydown = function(e) {
            if (e.key === 'Enter') {
                dialog.querySelector('#spikes-change-name-save').click();
            } else if (e.key === 'Escape') {
                closeDialog();
            }
        };
    }
    
    function nameInputStyle() {
        return [
            'flex:1',
            'padding:10px 12px',
            'border:2px solid #e0e0e0',
            'border-radius:6px',
            'font-size:14px',
            'font-family:inherit',
            'outline:none',
            'transition:border-color 0.15s'
        ].join(';');
    }
    
    function nameContinueBtnStyle() {
        return [
            'padding:10px 16px',
            'background:#f59e0b',
            'color:white',
            'border:none',
            'border-radius:6px',
            'font-size:14px',
            'font-weight:500',
            'cursor:pointer',
            'font-family:inherit',
            'white-space:nowrap'
        ].join(';');
    }
    
    // Check if reviewer is needed before save
    function ensureReviewerAndSave(saveFn) {
        if (currentReviewer) {
            saveFn();
        } else {
            // Need to show name prompt
            pendingSaveCallback = saveFn;
        }
    }

    function createSpike(rating, comments, elementData) {
        var spike = {
            id: nanoid(),
            type: elementData ? 'element' : 'page',
            projectKey: config.project,
            page: document.title || location.pathname,
            url: location.href,
            reviewer: getReviewerForSpike(),
            rating: rating,
            comments: comments,
            timestamp: new Date().toISOString(),
            viewport: {
                width: window.innerWidth,
                height: window.innerHeight
            }
        };
        
        if (elementData) {
            spike.selector = elementData.selector;
            spike.xpath = elementData.xpath;
            spike.elementText = elementData.elementText;
            spike.boundingBox = elementData.boundingBox;
        }
        
        return spike;
    }
    
    // Selector computation algorithm
    function computeSelector(el) {
        // Priority 1: ID (if unique)
        if (el.id && document.querySelectorAll('#' + CSS.escape(el.id)).length === 1) {
            return '#' + CSS.escape(el.id);
        }
        
        // Priority 2: Unique class
        var classes = Array.prototype.slice.call(el.classList);
        for (var i = 0; i < classes.length; i++) {
            var selector = '.' + CSS.escape(classes[i]);
            if (document.querySelectorAll(selector).length === 1) {
                return selector;
            }
        }
        
        // Priority 3: Tag + class (unique combination)
        var tag = el.tagName.toLowerCase();
        for (var j = 0; j < classes.length; j++) {
            var tagClassSelector = tag + '.' + CSS.escape(classes[j]);
            if (document.querySelectorAll(tagClassSelector).length === 1) {
                return tagClassSelector;
            }
        }
        
        // Priority 4: nth-child path
        return computeNthChildPath(el);
    }
    
    function computeNthChildPath(el) {
        var path = [];
        var current = el;
        
        while (current && current !== document.body && current !== document.documentElement) {
            var parent = current.parentElement;
            if (!parent) break;
            
            var siblings = Array.prototype.slice.call(parent.children);
            var index = siblings.indexOf(current) + 1;
            var tag = current.tagName.toLowerCase();
            
            path.unshift(tag + ':nth-child(' + index + ')');
            current = parent;
        }
        
        return 'body > ' + path.join(' > ');
    }
    
    function computeXPath(el) {
        var path = [];
        var current = el;
        
        while (current && current.nodeType === Node.ELEMENT_NODE) {
            var index = 1;
            var sibling = current.previousSibling;
            
            while (sibling) {
                if (sibling.nodeType === Node.ELEMENT_NODE && sibling.tagName === current.tagName) {
                    index++;
                }
                sibling = sibling.previousSibling;
            }
            
            var tag = current.tagName.toLowerCase();
            path.unshift(tag + '[' + index + ']');
            current = current.parentNode;
        }
        
        return '/' + path.join('/');
    }
    
    function captureElementData(el) {
        var rect = el.getBoundingClientRect();
        var text = (el.textContent || '').trim();
        
        return {
            selector: computeSelector(el),
            xpath: computeXPath(el),
            elementText: text.substring(0, 100) + (text.length > 100 ? '...' : ''),
            boundingBox: {
                x: Math.round(rect.left + window.scrollX),
                y: Math.round(rect.top + window.scrollY),
                width: Math.round(rect.width),
                height: Math.round(rect.height)
            }
        };
    }
    
    // Check if element is part of the widget
    function isWidgetElement(el) {
        if (!el) return false;
        var widgetIds = ['spikes-btn', 'spikes-modal', 'spikes-popover', 'spikes-container', 'spikes-reviewer', 'spikes-name-overlay'];
        if (widgetIds.indexOf(el.id) !== -1) return true;
        if (el.closest) {
            for (var i = 0; i < widgetIds.length; i++) {
                if (el.closest('#' + widgetIds[i])) return true;
            }
        }
        return false;
    }
    
    // Check if element should be excluded from spike mode
    function isExcludedElement(el) {
        if (!el) return true;
        var tag = el.tagName.toLowerCase();
        if (tag === 'html' || tag === 'body') return true;
        if (isWidgetElement(el)) return true;
        return false;
    }

    function createButton() {
        // Create container for button + indicator
        var container = document.createElement('div');
        container.id = 'spikes-container';
        container.style.cssText = [
            'position:fixed',
            getPositionStyles(),
            'z-index:2147483646',
            'display:flex',
            'flex-direction:column',
            'align-items:center',
            'gap:4px'
        ].join(';');
        
        btn = document.createElement('button');
        btn.id = 'spikes-btn';
        btn.innerHTML = '💬';
        btn.setAttribute('aria-label', 'Give Feedback');
        btn.style.cssText = [
            'width:56px',
            'height:56px',
            'background:' + config.color,
            'color:white',
            'border:none',
            'border-radius:50%',
            'font-size:24px',
            'cursor:pointer',
            'box-shadow:0 4px 12px rgba(0,0,0,0.3)',
            'transition:transform 0.2s,box-shadow 0.2s',
            'display:flex',
            'align-items:center',
            'justify-content:center',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif'
        ].join(';');

        btn.onmouseenter = function() {
            if (spikeMode !== 'armed') {
                btn.style.transform = 'scale(1.1)';
                btn.style.boxShadow = '0 6px 16px rgba(0,0,0,0.4)';
            }
        };
        btn.onmouseleave = function() {
            if (spikeMode !== 'armed') {
                btn.style.transform = 'scale(1)';
                btn.style.boxShadow = '0 4px 12px rgba(0,0,0,0.3)';
            }
        };
        btn.onclick = handleButtonClick;
        
        // Create reviewer indicator (U10)
        reviewerIndicator = document.createElement('div');
        reviewerIndicator.id = 'spikes-reviewer';
        reviewerIndicator.style.cssText = [
            'background:white',
            'padding:4px 10px',
            'border-radius:12px',
            'box-shadow:0 2px 8px rgba(0,0,0,0.15)',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif',
            'display:none',
            'white-space:nowrap',
            'max-width:200px',
            'overflow:hidden',
            'text-overflow:ellipsis'
        ].join(';');
        
        reviewerIndicator.onclick = function(e) {
            e.stopPropagation();
            showChangeNamePrompt();
        };
        
        container.appendChild(btn);
        container.appendChild(reviewerIndicator);
        document.body.appendChild(container);
        
        // Update indicator with current reviewer
        updateReviewerIndicator();
    }
    
    function handleButtonClick(e) {
        e.stopPropagation();
        
        if (spikeMode === 'idle') {
            enterArmedMode();
        } else if (spikeMode === 'armed') {
            exitSpikeMode();
            openModal();
        } else if (spikeMode === 'capturing') {
            // Already capturing, ignore
        }
    }
    
    function enterArmedMode() {
        spikeMode = 'armed';
        document.body.style.cursor = 'crosshair';
        
        // Button visual: pulse animation
        btn.style.animation = 'spikes-pulse 1.5s ease-in-out infinite';
        btn.setAttribute('title', 'Click element to spike, or click here for page feedback');
        
        // Add event listeners
        document.addEventListener('mouseover', handleMouseOver, true);
        document.addEventListener('mouseout', handleMouseOut, true);
        document.addEventListener('click', handleElementClick, true);
        document.addEventListener('keydown', handleEscapeKey);
    }
    
    function exitSpikeMode() {
        spikeMode = 'idle';
        document.body.style.cursor = '';
        
        // Reset button
        btn.style.animation = '';
        btn.style.transform = 'scale(1)';
        btn.removeAttribute('title');
        
        // Remove highlight
        removeHighlight();
        
        // Remove event listeners
        document.removeEventListener('mouseover', handleMouseOver, true);
        document.removeEventListener('mouseout', handleMouseOut, true);
        document.removeEventListener('click', handleElementClick, true);
        document.removeEventListener('keydown', handleEscapeKey);
        
        // Reset captured element
        capturedElement = null;
        capturedElementData = null;
    }
    
    function handleMouseOver(e) {
        if (spikeMode !== 'armed') return;
        if (isExcludedElement(e.target)) return;
        
        highlightElement(e.target);
    }
    
    function handleMouseOut(e) {
        if (spikeMode !== 'armed') return;
        removeHighlight();
    }
    
    function highlightElement(el) {
        if (highlightedElement === el) return;
        
        removeHighlight();
        highlightedElement = el;
        
        // Store original outline
        el._spikesOriginalOutline = el.style.outline;
        el._spikesOriginalOutlineOffset = el.style.outlineOffset;
        
        // Apply highlight
        el.style.outline = '2px solid #e74c3c';
        el.style.outlineOffset = '2px';
    }
    
    function removeHighlight() {
        if (highlightedElement) {
            highlightedElement.style.outline = highlightedElement._spikesOriginalOutline || '';
            highlightedElement.style.outlineOffset = highlightedElement._spikesOriginalOutlineOffset || '';
            delete highlightedElement._spikesOriginalOutline;
            delete highlightedElement._spikesOriginalOutlineOffset;
            highlightedElement = null;
        }
    }
    
    function handleElementClick(e) {
        if (spikeMode !== 'armed') return;
        
        // If clicking on widget elements, let them handle it
        if (isWidgetElement(e.target)) return;
        
        // If clicking on excluded elements, ignore
        if (isExcludedElement(e.target)) return;
        
        e.preventDefault();
        e.stopPropagation();
        
        // Capture the element
        capturedElement = e.target;
        capturedElementData = captureElementData(capturedElement);
        
        // Brief pulse animation on captured element
        pulseElement(capturedElement);
        
        // Enter capturing mode and show popover
        spikeMode = 'capturing';
        removeHighlight();
        document.body.style.cursor = '';
        btn.style.animation = '';
        
        // Remove hover listeners but keep others for cleanup
        document.removeEventListener('mouseover', handleMouseOver, true);
        document.removeEventListener('mouseout', handleMouseOut, true);
        
        showPopover(capturedElement, capturedElementData);
    }
    
    function pulseElement(el) {
        var originalBg = el.style.backgroundColor;
        el.style.transition = 'background-color 0.15s ease-out';
        el.style.backgroundColor = 'rgba(231, 76, 60, 0.3)';
        
        setTimeout(function() {
            el.style.backgroundColor = originalBg;
            setTimeout(function() {
                el.style.transition = '';
            }, 150);
        }, 150);
    }
    
    function handleEscapeKey(e) {
        if (e.key === 'Escape') {
            if (spikeMode === 'armed') {
                exitSpikeMode();
                openModal();
            } else if (spikeMode === 'capturing') {
                closePopover();
            }
        }
    }

    function createModal() {
        modal = document.createElement('div');
        modal.id = 'spikes-modal';
        modal.style.cssText = [
            'position:fixed',
            'top:0',
            'left:0',
            'right:0',
            'bottom:0',
            'background:rgba(0,0,0,0.6)',
            'display:none',
            'justify-content:center',
            'align-items:center',
            'z-index:2147483647',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif'
        ].join(';');

        var pageName = document.title || location.pathname;

        modal.innerHTML = [
            '<div id="spikes-modal-content" style="background:white;padding:24px;border-radius:12px;max-width:420px;width:90%;box-shadow:0 20px 60px rgba(0,0,0,0.3);">',
            '  <h2 style="margin:0 0 4px;color:#1a1a1a;font-size:18px;font-weight:600;">Page Feedback</h2>',
            '  <p style="color:#666;margin:0 0 20px;font-size:14px;">' + escapeHtml(pageName) + '</p>',
            '  <div id="spikes-modal-name-prompt-area"></div>',
            '  <div style="margin-bottom:16px;">',
            '    <div id="spikes-ratings" style="display:flex;gap:8px;flex-wrap:wrap;">',
            '      <button data-rating="love" style="' + ratingBtnStyle() + '">❤️ Love</button>',
            '      <button data-rating="like" style="' + ratingBtnStyle() + '">👍 Like</button>',
            '      <button data-rating="meh" style="' + ratingBtnStyle() + '">😐 Meh</button>',
            '      <button data-rating="no" style="' + ratingBtnStyle() + '">👎 No</button>',
            '    </div>',
            '  </div>',
            '  <textarea id="spikes-comments" placeholder="Any thoughts or suggestions?" style="' + textareaStyle() + '"></textarea>',
            '  <div style="display:flex;gap:8px;margin-top:16px;">',
            '    <button id="spikes-save" style="' + saveBtnStyle() + '">Save</button>',
            '    <button id="spikes-cancel" style="' + cancelBtnStyle() + '">Cancel</button>',
            '  </div>',
            '</div>'
        ].join('');

        document.body.appendChild(modal);

        // Rating buttons
        var ratingBtns = modal.querySelectorAll('#spikes-ratings button');
        for (var i = 0; i < ratingBtns.length; i++) {
            ratingBtns[i].onclick = function() {
                for (var j = 0; j < ratingBtns.length; j++) {
                    ratingBtns[j].style.borderColor = '#e0e0e0';
                    ratingBtns[j].style.background = 'white';
                }
                this.style.borderColor = config.color;
                this.style.background = '#fef5f5';
                selectedRating = this.getAttribute('data-rating');
            };
        }

        // Cancel button
        modal.querySelector('#spikes-cancel').onclick = closeModal;

        // Save button
        modal.querySelector('#spikes-save').onclick = handleSave;

        // Click outside to close
        modal.onclick = function(e) {
            if (e.target === modal) closeModal();
        };

        // Escape key for modal
        document.addEventListener('keydown', function(e) {
            if (e.key === 'Escape' && modal.style.display === 'flex') {
                closeModal();
            }
        });
    }
    
    function createPopover() {
        popover = document.createElement('div');
        popover.id = 'spikes-popover';
        popover.style.cssText = [
            'position:absolute',
            'background:white',
            'padding:20px',
            'border-radius:12px',
            'max-width:360px',
            'width:90vw',
            'box-shadow:0 20px 60px rgba(0,0,0,0.3)',
            'z-index:2147483647',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif',
            'display:none'
        ].join(';');
        
        document.body.appendChild(popover);
    }
    
    function showPopover(element, elementData) {
        var rect = element.getBoundingClientRect();
        var scrollX = window.scrollX;
        var scrollY = window.scrollY;
        
        // Popover content
        var truncatedSelector = elementData.selector.length > 40 
            ? elementData.selector.substring(0, 37) + '...' 
            : elementData.selector;
        var truncatedText = elementData.elementText || '<no text>';
        if (truncatedText.length > 50) {
            truncatedText = truncatedText.substring(0, 47) + '...';
        }
        
        popover.innerHTML = [
            '<div style="margin-bottom:16px;">',
            '  <div style="font-size:12px;color:#666;margin-bottom:4px;">Element Feedback</div>',
            '  <div style="background:#f5f5f5;padding:8px 10px;border-radius:6px;font-family:monospace;font-size:12px;color:#333;word-break:break-all;">' + escapeHtml(truncatedSelector) + '</div>',
            '  <div style="color:#888;font-size:12px;margin-top:4px;max-height:40px;overflow:hidden;">"' + escapeHtml(truncatedText) + '"</div>',
            '</div>',
            '<div id="spikes-popover-name-prompt-area"></div>',
            '<div style="margin-bottom:16px;">',
            '  <div id="spikes-popover-ratings" style="display:flex;gap:8px;flex-wrap:wrap;">',
            '    <button data-rating="love" style="' + ratingBtnStyle() + '">❤️ Love</button>',
            '    <button data-rating="like" style="' + ratingBtnStyle() + '">👍 Like</button>',
            '    <button data-rating="meh" style="' + ratingBtnStyle() + '">😐 Meh</button>',
            '    <button data-rating="no" style="' + ratingBtnStyle() + '">👎 No</button>',
            '  </div>',
            '</div>',
            '<textarea id="spikes-popover-comments" placeholder="Any thoughts about this element?" style="' + textareaStyle() + 'height:80px;"></textarea>',
            '<div style="display:flex;gap:8px;margin-top:16px;">',
            '  <button id="spikes-popover-save" style="' + saveBtnStyle() + '">Save</button>',
            '  <button id="spikes-popover-cancel" style="' + cancelBtnStyle() + '">Cancel</button>',
            '</div>'
        ].join('');
        
        // Position the popover
        var popoverWidth = 360;
        var popoverHeight = 280; // Approximate
        
        // Try to position above the element
        var left = rect.left + scrollX + (rect.width / 2) - (popoverWidth / 2);
        var top = rect.top + scrollY - popoverHeight - 10;
        
        // If not enough space above, position below
        if (top < scrollY + 10) {
            top = rect.bottom + scrollY + 10;
        }
        
        // Keep within viewport horizontally
        if (left < scrollX + 10) {
            left = scrollX + 10;
        } else if (left + popoverWidth > scrollX + window.innerWidth - 10) {
            left = scrollX + window.innerWidth - popoverWidth - 10;
        }
        
        popover.style.left = left + 'px';
        popover.style.top = top + 'px';
        popover.style.display = 'block';
        
        // Reset rating state for popover
        selectedRating = null;
        
        // Wire up rating buttons
        var ratingBtns = popover.querySelectorAll('#spikes-popover-ratings button');
        for (var i = 0; i < ratingBtns.length; i++) {
            ratingBtns[i].onclick = function(e) {
                e.stopPropagation();
                for (var j = 0; j < ratingBtns.length; j++) {
                    ratingBtns[j].style.borderColor = '#e0e0e0';
                    ratingBtns[j].style.background = 'white';
                }
                this.style.borderColor = config.color;
                this.style.background = '#fef5f5';
                selectedRating = this.getAttribute('data-rating');
            };
        }
        
        // Wire up cancel
        popover.querySelector('#spikes-popover-cancel').onclick = function(e) {
            e.stopPropagation();
            closePopover();
        };
        
        // Wire up save
        popover.querySelector('#spikes-popover-save').onclick = function(e) {
            e.stopPropagation();
            handlePopoverSave();
        };
        
        // Show name prompt if no reviewer, else focus textarea
        var popoverNamePromptArea = popover.querySelector('#spikes-popover-name-prompt-area');
        if (!currentReviewer) {
            showNamePrompt(popoverNamePromptArea, function() {
                popover.querySelector('#spikes-popover-comments').focus();
            });
        } else {
            popover.querySelector('#spikes-popover-comments').focus();
        }
        
        // Add click-outside listener
        setTimeout(function() {
            document.addEventListener('click', handleClickOutsidePopover);
        }, 0);
    }
    
    function handleClickOutsidePopover(e) {
        if (popover && popover.style.display === 'block') {
            if (!popover.contains(e.target) && !isWidgetElement(e.target)) {
                closePopover();
            }
        }
    }
    
    function closePopover() {
        if (popover) {
            popover.style.display = 'none';
        }
        document.removeEventListener('click', handleClickOutsidePopover);
        document.removeEventListener('click', handleElementClick, true);
        document.removeEventListener('keydown', handleEscapeKey);
        
        spikeMode = 'idle';
        capturedElement = null;
        capturedElementData = null;
        selectedRating = null;
    }
    
    function handlePopoverSave() {
        var comments = popover.querySelector('#spikes-popover-comments').value.trim();
        var saveBtn = popover.querySelector('#spikes-popover-save');
        
        // Require reviewer before saving
        if (!currentReviewer) {
            var namePromptArea = popover.querySelector('#spikes-popover-name-prompt-area');
            if (!namePromptArea.querySelector('#spikes-name-prompt')) {
                showNamePrompt(namePromptArea, function() {
                    handlePopoverSave(); // Retry save after name entered
                });
            }
            // Highlight the name input
            var nameInput = namePromptArea.querySelector('#spikes-name-input');
            if (nameInput) {
                nameInput.style.borderColor = '#ef4444';
                nameInput.focus();
            }
            return;
        }
        
        // Create and save element spike
        var spike = createSpike(selectedRating, comments, capturedElementData);
        saveSpike(spike);
        
        // Visual confirmation
        saveBtn.textContent = '✓ Saved!';
        saveBtn.style.background = '#16a34a';
        
        setTimeout(function() {
            closePopover();
            saveBtn.textContent = 'Save';
            saveBtn.style.background = '#22c55e';
        }, 800);
    }

    function escapeHtml(str) {
        var div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }

    function ratingBtnStyle() {
        return [
            'padding:8px 12px',
            'border:2px solid #e0e0e0',
            'background:white',
            'border-radius:6px',
            'cursor:pointer',
            'font-size:14px',
            'transition:all 0.15s',
            'font-family:inherit'
        ].join(';');
    }

    function textareaStyle() {
        return [
            'width:100%',
            'height:100px',
            'padding:12px',
            'border:2px solid #e0e0e0',
            'border-radius:8px',
            'font-size:14px',
            'font-family:inherit',
            'resize:none',
            'box-sizing:border-box'
        ].join(';');
    }

    function saveBtnStyle() {
        return [
            'flex:1',
            'padding:12px',
            'background:#22c55e',
            'color:white',
            'border:none',
            'border-radius:8px',
            'font-size:14px',
            'font-weight:500',
            'cursor:pointer',
            'transition:background 0.15s',
            'font-family:inherit'
        ].join(';');
    }

    function cancelBtnStyle() {
        return [
            'padding:12px 20px',
            'background:#f5f5f5',
            'color:#333',
            'border:none',
            'border-radius:8px',
            'font-size:14px',
            'cursor:pointer',
            'transition:background 0.15s',
            'font-family:inherit'
        ].join(';');
    }

    function openModal() {
        modal.style.display = 'flex';
        
        // Show name prompt if no reviewer yet
        var namePromptArea = modal.querySelector('#spikes-modal-name-prompt-area');
        namePromptArea.innerHTML = '';
        
        if (!currentReviewer) {
            showNamePrompt(namePromptArea, function() {
                // After name entered, focus the comments
                modal.querySelector('#spikes-comments').focus();
            });
        } else {
            modal.querySelector('#spikes-comments').focus();
        }
    }

    function closeModal() {
        modal.style.display = 'none';
        resetForm();
    }

    function resetForm() {
        selectedRating = null;
        modal.querySelector('#spikes-comments').value = '';
        var ratingBtns = modal.querySelectorAll('#spikes-ratings button');
        for (var i = 0; i < ratingBtns.length; i++) {
            ratingBtns[i].style.borderColor = '#e0e0e0';
            ratingBtns[i].style.background = 'white';
        }
    }

    function handleSave() {
        var comments = modal.querySelector('#spikes-comments').value.trim();
        var saveBtn = modal.querySelector('#spikes-save');
        
        // Require reviewer before saving
        if (!currentReviewer) {
            var namePromptArea = modal.querySelector('#spikes-modal-name-prompt-area');
            if (!namePromptArea.querySelector('#spikes-name-prompt')) {
                showNamePrompt(namePromptArea, function() {
                    handleSave(); // Retry save after name entered
                });
            }
            // Highlight the name input
            var nameInput = namePromptArea.querySelector('#spikes-name-input');
            if (nameInput) {
                nameInput.style.borderColor = '#ef4444';
                nameInput.focus();
            }
            return;
        }

        // Create and save spike
        var spike = createSpike(selectedRating, comments);
        saveSpike(spike);

        // Visual confirmation
        saveBtn.textContent = '✓ Saved!';
        saveBtn.style.background = '#16a34a';

        setTimeout(function() {
            closeModal();
            saveBtn.textContent = 'Save';
            saveBtn.style.background = '#22c55e';
        }, 800);
    }

    // Inject pulse animation keyframes
    function injectStyles() {
        var style = document.createElement('style');
        style.textContent = [
            '@keyframes spikes-pulse {',
            '  0%, 100% { transform: scale(1); box-shadow: 0 4px 12px rgba(0,0,0,0.3); }',
            '  50% { transform: scale(1.1); box-shadow: 0 6px 20px rgba(231,76,60,0.5); }',
            '}'
        ].join('');
        document.head.appendChild(style);
    }
    
    // Initialize when DOM is ready
    function init() {
        initReviewer();
        injectStyles();
        createButton();
        createModal();
        createPopover();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose config for debugging
    window.Spikes = {
        config: config,
        getSpikes: loadSpikes,
        getReviewer: function() { return currentReviewer; },
        setReviewerName: function(name) {
            if (currentReviewer) {
                currentReviewer.name = name;
                saveReviewer(currentReviewer);
            } else {
                createReviewer(name);
            }
        },
        clearReviewer: function() {
            localStorage.removeItem(REVIEWER_KEY);
            currentReviewer = null;
            updateReviewerIndicator();
        }
    };
})();
