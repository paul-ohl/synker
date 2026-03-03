/* ═══════════════════════════════════════════════════════════════
   Synker — Search Page Logic
   ═══════════════════════════════════════════════════════════════ */

(() => {
    "use strict";

    const $ = (sel, ctx = document) => ctx.querySelector(sel);
    const $$ = (sel, ctx = document) => [...ctx.querySelectorAll(sel)];

    // ─── DOM ───
    const searchInput      = $("#search-input");
    const searchClear      = $("#search-clear");
    const btnFilterToggle  = $("#btn-filter-toggle");
    const filtersPanel     = $("#filters-panel");
    const filterCount      = $("#filter-count");
    const activeFiltersBar = $("#active-filters");
    const resultsList      = $("#results-list");
    const resultsCount     = $("#results-count");
    const btnClearFilters  = $("#btn-clear-filters");
    const btnApplyFilters  = $("#btn-apply-filters");
    const viewGrid         = $("#view-grid");
    const viewList         = $("#view-list");
    const themeToggle      = $("#theme-toggle");

    // Filter inputs
    const filterExt           = $("#filter-ext");
    const filterContent       = $("#filter-content");
    const filterSizeMin       = $("#filter-size-min");
    const filterSizeMax       = $("#filter-size-max");
    const filterModifiedAfter = $("#filter-modified-after");
    const filterModifiedBefore = $("#filter-modified-before");
    const filterOrphans       = $("#filter-orphans");

    // ─── State ───
    let currentFilters = {};
    let selectedTags = new Set();

    // ═════════════════════════════════════════════════
    // Theme (reuse from global)
    // ═════════════════════════════════════════════════
    function initTheme() {
        const saved = localStorage.getItem("synker-theme");
        if (saved) document.documentElement.dataset.theme = saved;
    }

    function toggleTheme() {
        const html = document.documentElement;
        const next = (html.dataset.theme || "dark") === "dark" ? "light" : "dark";
        html.dataset.theme = next;
        localStorage.setItem("synker-theme", next);
    }

    // ═════════════════════════════════════════════════
    // Filter Panel Toggle
    // ═════════════════════════════════════════════════
    function toggleFilters() {
        const open = filtersPanel.dataset.open === "true";
        filtersPanel.dataset.open = !open;
        btnFilterToggle.setAttribute("aria-expanded", !open);
    }

    // ═════════════════════════════════════════════════
    // Collect & Apply Filters
    // ═════════════════════════════════════════════════
    function collectFilters() {
        const filters = {};

        const query = searchInput.value.trim();
        if (query) filters.name_contains = query;

        const ext = filterExt.value.trim();
        if (ext) filters.extension = ext;

        const content = filterContent.value.trim();
        if (content) filters.file_contains = content;

        const sizeMin = filterSizeMin.value;
        if (sizeMin) filters.size_greater_than = parseInt(sizeMin, 10);

        const sizeMax = filterSizeMax.value;
        if (sizeMax) filters.size_smaller_than = parseInt(sizeMax, 10);

        const after = filterModifiedAfter.value;
        if (after) filters.modified_after = after;

        const before = filterModifiedBefore.value;
        if (before) filters.modified_before = before;

        if (filterOrphans.checked) filters.orphans = true;

        if (selectedTags.size > 0) filters.tags = [...selectedTags];

        return filters;
    }

    function countActive(filters) {
        return Object.keys(filters).filter(k => k !== "name_contains").length;
    }

    function applyFilters() {
        currentFilters = collectFilters();
        const count = countActive(currentFilters);

        // Update badge
        if (count > 0) {
            filterCount.textContent = count;
            filterCount.hidden = false;
        } else {
            filterCount.hidden = true;
        }

        // Update active-filter chips
        renderActiveChips(currentFilters);

        // TODO: Call API → POST /api/files/find with currentFilters as FileQuery
        // For now, log and show visual feedback
        console.log("Applying filters:", currentFilters);

        // Simulate search (placeholder)
        performSearch(currentFilters);
    }

    function clearFilters() {
        searchInput.value = "";
        filterExt.value = "";
        filterContent.value = "";
        filterSizeMin.value = "";
        filterSizeMax.value = "";
        filterModifiedAfter.value = "";
        filterModifiedBefore.value = "";
        filterOrphans.checked = false;
        selectedTags.clear();

        $$(".tag.active", filtersPanel).forEach(t => t.classList.remove("active"));

        currentFilters = {};
        filterCount.hidden = true;
        activeFiltersBar.hidden = true;
        activeFiltersBar.innerHTML = "";

        performSearch({});
    }

    // ═════════════════════════════════════════════════
    // Active Filter Chips
    // ═════════════════════════════════════════════════
    const FILTER_LABELS = {
        name_contains: "Name",
        extension: "Extension",
        file_contains: "Content",
        size_greater_than: "Min size",
        size_smaller_than: "Max size",
        modified_after: "After",
        modified_before: "Before",
        orphans: "Orphans",
        tags: "Tags",
    };

    function renderActiveChips(filters) {
        activeFiltersBar.innerHTML = "";
        const keys = Object.keys(filters);

        if (keys.length === 0 || (keys.length === 1 && keys[0] === "name_contains")) {
            activeFiltersBar.hidden = true;
            return;
        }

        activeFiltersBar.hidden = false;

        for (const [key, value] of Object.entries(filters)) {
            if (key === "name_contains") continue;

            const label = FILTER_LABELS[key] || key;
            const display = Array.isArray(value) ? value.join(", ") : String(value);

            const chip = document.createElement("span");
            chip.className = "filter-chip";
            chip.innerHTML = `
                <span>${label}: ${display}</span>
                <button class="filter-chip-remove" data-filter="${key}" aria-label="Remove ${label} filter">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                </button>
            `;
            activeFiltersBar.appendChild(chip);
        }
    }

    function removeFilter(key) {
        switch (key) {
            case "extension":      filterExt.value = ""; break;
            case "file_contains":  filterContent.value = ""; break;
            case "size_greater_than": filterSizeMin.value = ""; break;
            case "size_smaller_than": filterSizeMax.value = ""; break;
            case "modified_after":  filterModifiedAfter.value = ""; break;
            case "modified_before": filterModifiedBefore.value = ""; break;
            case "orphans":         filterOrphans.checked = false; break;
            case "tags":
                selectedTags.clear();
                $$(".tag.active", filtersPanel).forEach(t => t.classList.remove("active"));
                break;
        }
        applyFilters();
    }

    // ═════════════════════════════════════════════════
    // Search (calls API)
    // ═════════════════════════════════════════════════
    async function performSearch(filters) {
        try {
            const res = await fetch("/api/files");
            if (!res.ok) throw new Error(res.statusText);
            let results = await res.json();

            // Client-side filtering (the API list returns all files,
            // we apply filters client-side for responsiveness)
            results = results.filter(file => {
                if (filters.name_contains) {
                    if (!file.name.toLowerCase().includes(filters.name_contains.toLowerCase())) return false;
                }
                if (filters.extension) {
                    if (!file.ext.toLowerCase().includes(filters.extension.toLowerCase())) return false;
                }
                if (filters.size_greater_than && file.size < filters.size_greater_than) return false;
                if (filters.size_smaller_than && file.size > filters.size_smaller_than) return false;
                if (filters.tags && filters.tags.length > 0) {
                    const fileTags = (file.tags || []).map(t => t.toLowerCase());
                    if (!filters.tags.every(t => fileTags.includes(t.toLowerCase()))) return false;
                }
                return true;
            });

            renderResults(results);
        } catch (e) {
            console.error("Search failed:", e);
            renderResults([]);
        }
    }

    function renderResults(results) {
        const empty = $("#results-empty");
        resultsCount.textContent = `${results.length} file${results.length !== 1 ? "s" : ""}`;

        if (results.length === 0) {
            resultsList.innerHTML = "";
            if (empty) {
                empty.style.display = "";
                const hasFilters = Object.keys(currentFilters).length > 0;
                empty.querySelector(".results-empty-title").textContent = hasFilters ? "No results found" : "No files yet";
                empty.querySelector(".results-empty-subtitle").textContent = hasFilters
                    ? "Try adjusting your search or filters"
                    : "Create a file in the editor to get started";
            }
            return;
        }

        if (empty) empty.style.display = "none";

        resultsList.innerHTML = results.map(file => `
            <div class="result-card" data-file-id="${file.id}">
                <div class="result-card-header">
                    <svg class="result-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>
                    <span class="result-name">${escapeHtml(file.name)}</span>
                    <span class="result-ext">.${escapeHtml(file.ext)}</span>
                    <button class="result-delete" data-file-id="${file.id}" title="Delete file">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                    </button>
                </div>
                <div class="result-card-meta">
                    <span class="result-size">${formatSize(file.size)}</span>
                    <span class="result-date">${formatDate(file.modified_at)}</span>
                </div>
                ${file.tags && file.tags.length > 0 ? `
                    <div class="result-card-tags">
                        ${file.tags.map(t => `<span class="result-tag">${escapeHtml(t)}</span>`).join("")}
                    </div>
                ` : ""}
            </div>
        `).join("");
    }

    function escapeHtml(text) {
        const map = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" };
        return text.replace(/[&<>"]/g, c => map[c]);
    }

    function formatSize(bytes) {
        if (bytes === 0) return "0 B";
        const units = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
    }

    function formatDate(iso) {
        if (!iso) return "";
        const d = new Date(iso);
        return d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
    }

    async function deleteFileFromSearch(fileId) {
        if (!confirm("Delete this file? This cannot be undone.")) return;

        try {
            const res = await fetch(`/api/files/${fileId}`, { method: "DELETE" });
            if (!res.ok && res.status !== 204) throw new Error(res.statusText);
            await loadTags();
            performSearch(currentFilters);
        } catch (e) {
            console.error("Delete failed:", e);
            alert(`Failed to delete file: ${e.message}`);
        }
    }

    // ═════════════════════════════════════════════════
    // View Toggle
    // ═════════════════════════════════════════════════
    function setView(mode) {
        resultsList.dataset.view = mode;
        viewGrid.classList.toggle("active", mode === "grid");
        viewList.classList.toggle("active", mode === "list");
        localStorage.setItem("synker-search-view", mode);
    }

    // ═════════════════════════════════════════════════
    // Tags
    // ═════════════════════════════════════════════════
    async function loadTags() {
        const filterTagsEl = $("#filter-tags");
        if (!filterTagsEl) return;

        try {
            const res = await fetch("/api/tags");
            if (!res.ok) throw new Error(res.statusText);
            const tags = await res.json();

            if (tags.length === 0) {
                filterTagsEl.innerHTML = '<span class="filter-empty">No tags available</span>';
                return;
            }

            filterTagsEl.innerHTML = tags.map(t =>
                `<button class="tag" data-tag="${escapeHtml(t)}">${escapeHtml(t)}</button>`
            ).join("");
        } catch (e) {
            console.error("Failed to load tags:", e);
        }
    }

    function handleTagClick(e) {
        const tagBtn = e.target.closest(".tag[data-tag]");
        if (!tagBtn) return;

        const tag = tagBtn.dataset.tag;
        if (selectedTags.has(tag)) {
            selectedTags.delete(tag);
            tagBtn.classList.remove("active");
        } else {
            selectedTags.add(tag);
            tagBtn.classList.add("active");
        }
    }

    // ═════════════════════════════════════════════════
    // Init
    // ═════════════════════════════════════════════════
    function init() {
        initTheme();

        // Restore view preference
        const savedView = localStorage.getItem("synker-search-view") || "list";
        setView(savedView);

        // Search input events
        let debounce;
        searchInput.addEventListener("input", () => {
            searchClear.hidden = searchInput.value.length === 0;
            clearTimeout(debounce);
            debounce = setTimeout(applyFilters, 300);
        });

        searchClear.addEventListener("click", () => {
            searchInput.value = "";
            searchClear.hidden = true;
            applyFilters();
            searchInput.focus();
        });

        // Keyboard shortcut: / to focus search
        document.addEventListener("keydown", (e) => {
            if (e.key === "/" && document.activeElement.tagName !== "INPUT" && document.activeElement.tagName !== "TEXTAREA") {
                e.preventDefault();
                searchInput.focus();
            }
            // Escape to close filters
            if (e.key === "Escape" && filtersPanel.dataset.open === "true") {
                toggleFilters();
            }
        });

        // Filter toggle
        btnFilterToggle.addEventListener("click", toggleFilters);

        // Apply / Clear
        btnApplyFilters.addEventListener("click", () => {
            applyFilters();
            // Close panel after applying on mobile
            if (window.innerWidth <= 480) {
                filtersPanel.dataset.open = "false";
                btnFilterToggle.setAttribute("aria-expanded", "false");
            }
        });
        btnClearFilters.addEventListener("click", clearFilters);

        // Active chip removal
        activeFiltersBar.addEventListener("click", (e) => {
            const removeBtn = e.target.closest(".filter-chip-remove");
            if (removeBtn) removeFilter(removeBtn.dataset.filter);
        });

        // View toggles
        viewGrid.addEventListener("click", () => setView("grid"));
        viewList.addEventListener("click", () => setView("list"));

        // Tag clicks
        const filterTagsEl = $("#filter-tags");
        if (filterTagsEl) filterTagsEl.addEventListener("click", handleTagClick);

        // Result card clicks (delete, open)
        resultsList.addEventListener("click", (e) => {
            const deleteBtn = e.target.closest(".result-delete");
            if (deleteBtn) {
                e.stopPropagation();
                deleteFileFromSearch(deleteBtn.dataset.fileId);
                return;
            }
            const card = e.target.closest(".result-card");
            if (card && card.dataset.fileId) {
                // Navigate to editor with this file
                window.location.href = `/?file=${card.dataset.fileId}`;
            }
        });

        // Theme
        if (themeToggle) themeToggle.addEventListener("click", toggleTheme);

        // Initial load
        loadTags();
        performSearch({});
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
