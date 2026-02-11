// Spikes Review Mode — In-context feedback overlay
// https://spikes.sh
(function() {
    'use strict';

    // State
    var allSpikes = [];
    var pageSpikes = [];  // Spikes filtered to current page (preserved for refresh)
    var filteredSpikes = [];
    var currentFilters = { reviewer: '', rating: '' };
    var markers = [];
    var activePopover = null;

    // Rating colors
    var ratingColors = {
        love: '#27ae60',
        like: '#3498db',
        meh: '#f39c12',
        no: '#e74c3c',
        none: '#95a5a6'
    };

    var ratingEmojis = {
        love: '❤️',
        like: '👍',
        meh: '😐',
        no: '👎',
        none: '—'
    };

    // Fetch spikes from server
    function fetchSpikes() {
        var xhr = new XMLHttpRequest();
        xhr.open('GET', '/spikes', true);
        xhr.onreadystatechange = function() {
            if (xhr.readyState === 4 && xhr.status === 200) {
                try {
                    allSpikes = JSON.parse(xhr.responseText);
                    filterSpikesForCurrentPage();
                    applyFilters();
                } catch (e) {
                    console.error('Spikes Review: Failed to parse spikes', e);
                }
            }
        };
        xhr.send();
    }

    function filterSpikesForCurrentPage() {
        var currentPath = location.pathname;
        var currentTitle = document.title;
        var currentUrl = location.href;

        // Match spikes by URL, path, or page title (store in pageSpikes, not allSpikes)
        pageSpikes = allSpikes.filter(function(spike) {
            if (spike.url === currentUrl) return true;
            if (spike.url && spike.url.indexOf(currentPath) !== -1) return true;
            if (spike.page === currentTitle) return true;
            if (spike.page === currentPath) return true;
            // Also match if the spike URL path matches current path
            try {
                var spikeUrl = new URL(spike.url);
                if (spikeUrl.pathname === currentPath) return true;
            } catch (e) {}
            return false;
        });
    }

    function applyFilters() {
        filteredSpikes = pageSpikes.filter(function(spike) {
            if (currentFilters.reviewer && 
                (!spike.reviewer || spike.reviewer.name !== currentFilters.reviewer)) {
                return false;
            }
            if (currentFilters.rating && spike.rating !== currentFilters.rating) {
                return false;
            }
            return true;
        });

        clearMarkers();
        renderMarkers();
        updateReviewBar();
    }

    function clearMarkers() {
        markers.forEach(function(marker) {
            if (marker.parentNode) {
                marker.parentNode.removeChild(marker);
            }
        });
        markers = [];
        closePopover();
    }

    function renderMarkers() {
        // Group spikes by type
        var elementTypeSpikes = filteredSpikes.filter(function(s) { return s.type === 'element'; });
        var pageTypeSpikes = filteredSpikes.filter(function(s) { return s.type === 'page'; });

        // Group element spikes by selector
        var spikesBySelector = {};
        elementTypeSpikes.forEach(function(spike) {
            var key = spike.selector || 'unknown';
            if (!spikesBySelector[key]) {
                spikesBySelector[key] = [];
            }
            spikesBySelector[key].push(spike);
        });

        // Render element markers
        Object.keys(spikesBySelector).forEach(function(selector) {
            var spikesForElement = spikesBySelector[selector];
            renderElementMarker(selector, spikesForElement);
        });

        // Render page spikes indicator
        if (pageTypeSpikes.length > 0) {
            renderPageSpikesIndicator(pageTypeSpikes);
        }
    }

    function renderElementMarker(selector, spikes) {
        var element = null;
        try {
            element = document.querySelector(selector);
        } catch (e) {
            // Invalid selector
        }

        var marker = document.createElement('div');
        marker.className = 'spikes-review-marker';

        // Determine dominant rating for color
        var dominantRating = getDominantRating(spikes);
        var color = ratingColors[dominantRating] || ratingColors.none;

        marker.style.cssText = [
            'position:absolute',
            'width:24px',
            'height:24px',
            'background:' + color,
            'border-radius:50%',
            'display:flex',
            'align-items:center',
            'justify-content:center',
            'font-size:12px',
            'font-weight:600',
            'color:white',
            'cursor:pointer',
            'box-shadow:0 2px 8px rgba(0,0,0,0.3)',
            'z-index:2147483640',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif',
            'transition:transform 0.15s',
            'border:2px solid white'
        ].join(';');

        marker.textContent = spikes.length > 1 ? spikes.length : '🗡️';
        if (spikes.length > 1) {
            marker.style.fontSize = '11px';
        } else {
            marker.style.fontSize = '14px';
        }

        if (element) {
            // Position at top-right of element
            var rect = element.getBoundingClientRect();
            marker.style.top = (rect.top + window.scrollY - 12) + 'px';
            marker.style.left = (rect.right + window.scrollX - 12) + 'px';
        } else {
            // Orphaned spike - position based on stored bounding box or show in orphan area
            marker.style.opacity = '0.6';
            marker.title = 'Element not found: ' + selector;
            
            if (spikes[0].boundingBox) {
                var bb = spikes[0].boundingBox;
                marker.style.top = (bb.y - 12) + 'px';
                marker.style.left = (bb.x + bb.width - 12) + 'px';
            } else {
                console.warn('Spikes Review: Cannot position marker for selector:', selector);
                return;
            }
        }

        marker.onmouseenter = function() {
            marker.style.transform = 'scale(1.2)';
        };
        marker.onmouseleave = function() {
            marker.style.transform = 'scale(1)';
        };
        marker.onclick = function(e) {
            e.stopPropagation();
            showPopover(marker, spikes, element);
        };

        document.body.appendChild(marker);
        markers.push(marker);
    }

    function renderPageSpikesIndicator(spikes) {
        var indicator = document.createElement('div');
        indicator.className = 'spikes-review-page-indicator';

        var dominantRating = getDominantRating(spikes);
        var color = ratingColors[dominantRating] || ratingColors.none;

        indicator.style.cssText = [
            'position:fixed',
            'bottom:20px',
            'left:20px',
            'background:white',
            'padding:12px 16px',
            'border-radius:8px',
            'box-shadow:0 4px 16px rgba(0,0,0,0.2)',
            'z-index:2147483640',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif',
            'cursor:pointer',
            'display:flex',
            'align-items:center',
            'gap:8px',
            'border-left:4px solid ' + color
        ].join(';');

        indicator.innerHTML = '<span style="font-size:18px;">📄</span>' +
            '<span style="font-size:14px;color:#333;">' + spikes.length + ' page spike' + 
            (spikes.length === 1 ? '' : 's') + '</span>';

        indicator.onclick = function(e) {
            e.stopPropagation();
            showPopover(indicator, spikes, null);
        };

        document.body.appendChild(indicator);
        markers.push(indicator);
    }

    function getDominantRating(spikes) {
        var counts = { love: 0, like: 0, meh: 0, no: 0, none: 0 };
        spikes.forEach(function(s) {
            var r = s.rating || 'none';
            counts[r] = (counts[r] || 0) + 1;
        });

        var max = 0;
        var dominant = 'none';
        Object.keys(counts).forEach(function(r) {
            if (counts[r] > max) {
                max = counts[r];
                dominant = r;
            }
        });
        return dominant;
    }

    function showPopover(anchor, spikes, element) {
        closePopover();

        var popover = document.createElement('div');
        popover.className = 'spikes-review-popover';
        popover.style.cssText = [
            'position:absolute',
            'background:white',
            'padding:16px',
            'border-radius:10px',
            'box-shadow:0 10px 40px rgba(0,0,0,0.25)',
            'z-index:2147483645',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif',
            'max-width:360px',
            'width:90vw',
            'max-height:400px',
            'overflow-y:auto'
        ].join(';');

        var header = element 
            ? '<div style="font-size:12px;color:#666;margin-bottom:12px;">Element Feedback</div>'
            : '<div style="font-size:12px;color:#666;margin-bottom:12px;">Page Feedback</div>';

        var spikeCards = spikes.map(function(spike) {
            var ratingColor = ratingColors[spike.rating] || ratingColors.none;
            var emoji = ratingEmojis[spike.rating] || ratingEmojis.none;
            var reviewerName = spike.reviewer ? spike.reviewer.name : 'Anonymous';
            var timeAgo = formatTimeAgo(spike.timestamp);

            return '<div style="border-left:3px solid ' + ratingColor + ';padding:8px 12px;margin-bottom:8px;background:#f9f9f9;border-radius:0 6px 6px 0;">' +
                '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:4px;">' +
                    '<span style="font-weight:500;font-size:13px;color:#333;">' + escapeHtml(reviewerName) + '</span>' +
                    '<span style="font-size:18px;">' + emoji + '</span>' +
                '</div>' +
                (spike.comments ? '<div style="font-size:13px;color:#555;line-height:1.4;">' + escapeHtml(spike.comments) + '</div>' : '') +
                '<div style="font-size:11px;color:#999;margin-top:6px;">' + timeAgo + '</div>' +
            '</div>';
        }).join('');

        popover.innerHTML = header + spikeCards;

        // Position the popover
        var anchorRect = anchor.getBoundingClientRect();
        var scrollY = window.scrollY;
        var scrollX = window.scrollX;
        var popoverHeight = 400; // max-height from styles

        // Try positioning below the anchor
        var top = anchorRect.bottom + scrollY + 8;
        var left = anchorRect.left + scrollX;

        // If popover would extend below viewport, position above instead
        if (top + popoverHeight > window.innerHeight + scrollY) {
            top = Math.max(scrollY + 60, anchorRect.top + scrollY - popoverHeight - 8);
        }

        // Adjust horizontal position if off-screen
        if (left + 360 > window.innerWidth + scrollX) {
            left = window.innerWidth + scrollX - 370;
        }
        if (left < scrollX + 10) {
            left = scrollX + 10;
        }

        popover.style.top = top + 'px';
        popover.style.left = left + 'px';

        document.body.appendChild(popover);
        activePopover = popover;

        // Close on click outside
        setTimeout(function() {
            document.addEventListener('click', handleClickOutside);
        }, 0);
    }

    function closePopover() {
        if (activePopover) {
            activePopover.parentNode.removeChild(activePopover);
            activePopover = null;
        }
        document.removeEventListener('click', handleClickOutside);
    }

    function handleClickOutside(e) {
        if (activePopover && !activePopover.contains(e.target)) {
            closePopover();
        }
    }

    function formatTimeAgo(timestamp) {
        if (!timestamp) return '';
        var date = new Date(timestamp);
        var now = new Date();
        var seconds = Math.floor((now - date) / 1000);

        if (seconds < 60) return 'just now';
        if (seconds < 3600) return Math.floor(seconds / 60) + 'm ago';
        if (seconds < 86400) return Math.floor(seconds / 3600) + 'h ago';
        if (seconds < 604800) return Math.floor(seconds / 86400) + 'd ago';
        return date.toLocaleDateString();
    }

    function escapeHtml(str) {
        if (!str) return '';
        var div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }

    // Review bar at top of page
    function createReviewBar() {
        var bar = document.createElement('div');
        bar.id = 'spikes-review-bar';
        bar.style.cssText = [
            'position:fixed',
            'top:0',
            'left:0',
            'right:0',
            'background:linear-gradient(135deg,#1a1a1a 0%,#2d2d2d 100%)',
            'color:white',
            'padding:10px 20px',
            'display:flex',
            'align-items:center',
            'gap:16px',
            'flex-wrap:wrap',
            'z-index:2147483646',
            'font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif',
            'font-size:13px',
            'box-shadow:0 2px 10px rgba(0,0,0,0.2)'
        ].join(';');

        bar.innerHTML = [
            '<div style="display:flex;align-items:center;gap:8px;">',
            '  <span style="font-size:18px;">🗡️</span>',
            '  <span style="font-weight:600;">Review Mode</span>',
            '  <span id="spikes-review-count" style="color:#aaa;">Loading...</span>',
            '</div>',
            '<div style="display:flex;align-items:center;gap:12px;margin-left:auto;">',
            '  <select id="spikes-review-filter-reviewer" style="' + selectStyle() + '">',
            '    <option value="">All reviewers</option>',
            '  </select>',
            '  <select id="spikes-review-filter-rating" style="' + selectStyle() + '">',
            '    <option value="">All ratings</option>',
            '    <option value="love">❤️ Love</option>',
            '    <option value="like">👍 Like</option>',
            '    <option value="meh">😐 Meh</option>',
            '    <option value="no">👎 No</option>',
            '  </select>',
            '  <a href="' + removeReviewParam() + '" style="color:#aaa;text-decoration:none;font-size:12px;">Exit review mode</a>',
            '</div>'
        ].join('');

        document.body.appendChild(bar);

        // Add padding to body to account for bar (preserve existing padding)
        var existingPadding = parseInt(window.getComputedStyle(document.body).paddingTop, 10) || 0;
        document.body.style.paddingTop = (existingPadding + 50) + 'px';

        // Wire up filters
        var reviewerSelect = bar.querySelector('#spikes-review-filter-reviewer');
        var ratingSelect = bar.querySelector('#spikes-review-filter-rating');

        reviewerSelect.onchange = function() {
            currentFilters.reviewer = this.value;
            applyFilters();
        };

        ratingSelect.onchange = function() {
            currentFilters.rating = this.value;
            applyFilters();
        };
    }

    function updateReviewBar() {
        var countEl = document.getElementById('spikes-review-count');
        var reviewerSelect = document.getElementById('spikes-review-filter-reviewer');

        if (countEl) {
            var uniqueReviewers = {};
            pageSpikes.forEach(function(s) {
                if (s.reviewer && s.reviewer.name) {
                    uniqueReviewers[s.reviewer.name] = true;
                }
            });
            var reviewerCount = Object.keys(uniqueReviewers).length;

            countEl.textContent = filteredSpikes.length + ' spike' + 
                (filteredSpikes.length === 1 ? '' : 's') +
                ' from ' + reviewerCount + ' reviewer' + 
                (reviewerCount === 1 ? '' : 's');
        }

        // Populate reviewer filter
        if (reviewerSelect && reviewerSelect.options.length <= 1) {
            var reviewers = {};
            pageSpikes.forEach(function(s) {
                if (s.reviewer && s.reviewer.name) {
                    reviewers[s.reviewer.name] = true;
                }
            });

            Object.keys(reviewers).sort().forEach(function(name) {
                var opt = document.createElement('option');
                opt.value = name;
                opt.textContent = name;
                reviewerSelect.appendChild(opt);
            });
        }
    }

    function selectStyle() {
        return [
            'padding:6px 10px',
            'border-radius:4px',
            'border:1px solid #444',
            'background:#333',
            'color:white',
            'font-size:12px',
            'cursor:pointer'
        ].join(';');
    }

    function removeReviewParam() {
        var url = new URL(location.href);
        url.searchParams.delete('review');
        return url.toString();
    }

    // Handle window resize - reposition markers
    var resizeTimeout;
    window.addEventListener('resize', function() {
        clearTimeout(resizeTimeout);
        resizeTimeout = setTimeout(function() {
            clearMarkers();
            renderMarkers();
        }, 100);
    });

    // Handle scroll - markers are absolutely positioned so they move with content
    // For fixed positioning we'd need to update on scroll, but absolute is fine

    // Initialize
    function init() {
        createReviewBar();
        fetchSpikes();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Expose API for debugging
    window.SpikesReview = {
        refresh: fetchSpikes,
        getSpikes: function() { return allSpikes; },
        getFiltered: function() { return filteredSpikes; }
    };
})();
