/* ═══════════════════════════════════════════════════════════════
   Synker — Editor Logic
   ═══════════════════════════════════════════════════════════════ */

(() => {
    "use strict";

    // ─── DOM References ───
    const $ = (sel, ctx = document) => ctx.querySelector(sel);
    const $$ = (sel, ctx = document) => [...ctx.querySelectorAll(sel)];

    const editor        = $("#editor");
    const preview       = $("#preview");
    const lineNumbers   = $("#line-numbers");
    const charCount     = $("#char-count");
    const wordCount     = $("#word-count");
    const statusCursor  = $("#status-cursor");
    const statusModified = $("#status-modified");
    const statusFile    = $("#status-file");
    const statusFileLabel = $("#status-file-label");
    const statusType    = $("#status-type");
    const editorPanes   = $("#editor-panes");
    const viewModeSelect = $("#view-mode");
    const sidebar       = $("#sidebar");
    const btnToggleSidebar = $("#btn-toggle-sidebar");
    const btnNewFile    = $("#btn-new-file");
    const btnUploadFile = $("#btn-upload-file");
    const btnDownload   = $("#btn-download");
    const fileTree      = $("#file-tree");
    const paneResizer   = $("#pane-resizer");
    const themeToggle   = $("#theme-toggle");

    // Properties / tag panel
    const tagPanel      = $("#tag-panel");
    const fileTags      = $("#file-tags");
    const tagInput      = $("#tag-input");
    const tagSuggestions = $("#tag-suggestions");
    const btnAddTag     = $("#btn-add-tag");
    const btnCloseTags  = $("#btn-close-tags");
    const btnTagsToggle = $("#btn-tags-toggle");
    const sidebarTags   = $("#sidebar-tags");

    // Frontmatter properties
    const fmTitle       = $("#fm-title");
    const fmDescription = $("#fm-description");
    const fmVisibility  = $("#fm-visibility");
    const fmDate        = $("#fm-date");
    const fmLastMod     = $("#fm-last-mod");

    // Rename
    const renameInput   = $("#rename-input");

    // Image viewer
    const imageViewer   = $("#image-viewer");
    const imagePreview  = $("#image-preview");
    const btnZoomIn     = $("#btn-zoom-in");
    const btnZoomOut    = $("#btn-zoom-out");
    const btnZoomReset  = $("#btn-zoom-reset");

    // Upload modal
    const uploadModal   = $("#upload-modal");
    const uploadDropzone = $("#upload-dropzone");
    const uploadInput   = $("#upload-input");
    const uploadFileInfo = $("#upload-file-info");
    const uploadFileName = $("#upload-file-name");
    const uploadFileSize = $("#upload-file-size");
    const uploadTagsInput = $("#upload-tags");
    const btnConfirmUpload = $("#btn-confirm-upload");
    const btnCancelUpload = $("#btn-cancel-upload");
    const btnCloseUpload = $("#btn-close-upload");

    // ─── State ───
    let isModified = false;
    let savedContent = "";
    let currentFileId = null;
    let currentFileName = "untitled";
    let currentFileExt = "md";
    let currentFileTags = [];
    let currentFileMime = "text/markdown";
    let fileList = [];
    let allTags = [];
    let pendingUploadFile = null;
    let imageZoom = 1;
    let isImageFile = false;
    let currentFrontmatter = null;  // parsed frontmatter object

    // ═══════════════════════════════════════════════════════════
    // Pane visibility helper
    // ═══════════════════════════════════════════════════════════
    function showPane(which) {
        // which: "editor" | "image"
        editorPanes.style.display = which === "editor" ? "" : "none";
        imageViewer.style.display = which === "image" ? "" : "none";
    }

    // ═══════════════════════════════════════════════════════════
    // API Helpers
    // ═══════════════════════════════════════════════════════════
    const API_BASE = "/api";

    async function apiGet(path) {
        const res = await fetch(`${API_BASE}${path}`);
        if (!res.ok) {
            const err = await res.json().catch(() => ({ error: res.statusText }));
            throw new Error(err.error || res.statusText);
        }
        return res.status === 204 ? null : res.json();
    }

    async function apiPost(path, body) {
        const res = await fetch(`${API_BASE}${path}`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        });
        if (!res.ok) {
            const err = await res.json().catch(() => ({ error: res.statusText }));
            throw new Error(err.error || res.statusText);
        }
        return res.json();
    }

    async function apiPut(path, body) {
        const res = await fetch(`${API_BASE}${path}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        });
        if (!res.ok) {
            const err = await res.json().catch(() => ({ error: res.statusText }));
            throw new Error(err.error || res.statusText);
        }
        return res.json();
    }

    async function apiDelete(path) {
        const res = await fetch(`${API_BASE}${path}`, { method: "DELETE" });
        if (!res.ok && res.status !== 204) {
            const err = await res.json().catch(() => ({ error: res.statusText }));
            throw new Error(err.error || res.statusText);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Utility
    // ═══════════════════════════════════════════════════════════
    function escapeHtml(text) {
        const map = { "&": "&amp;", "<": "&lt;", ">": "&gt;" };
        return text.replace(/[&<>]/g, c => map[c]);
    }

    function formatBytes(bytes) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
    }

    const IMAGE_EXTS = new Set(["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "avif"]);

    // ═══════════════════════════════════════════════════════════
    // YAML Frontmatter  (parse / serialize / helpers)
    // ═══════════════════════════════════════════════════════════

    /**
     * Parse YAML frontmatter from markdown text.
     * Returns { fm: { title, description, tags, date, last_mod, visibility, ... } | null, body: string }
     */
    function parseFrontmatter(text) {
        if (!text || !text.startsWith("---")) return { fm: null, body: text };
        const end = text.indexOf("\n---", 3);
        if (end === -1) return { fm: null, body: text };

        const yamlBlock = text.slice(4, end); // skip opening ---\n
        const body = text.slice(end + 4).replace(/^\n/, ""); // skip closing ---\n
        const fm = {};

        let currentKey = null;
        let inList = false;

        for (const rawLine of yamlBlock.split("\n")) {
            const line = rawLine;
            // Multi-line list item:  - value
            if (inList && /^\s+-\s+(.*)/.test(line)) {
                const val = line.match(/^\s+-\s+(.*)/)[1].replace(/^["']|["']$/g, "").trim();
                if (val) fm[currentKey].push(val);
                continue;
            }
            inList = false;

            const kv = line.match(/^([\w][\w_]*):\s*(.*)?$/);
            if (!kv) continue;

            const key = kv[1];
            let val = (kv[2] || "").trim();

            // Inline array: [a, b, c]
            if (val.startsWith("[") && val.endsWith("]")) {
                fm[key] = val.slice(1, -1).split(",").map(s => s.trim().replace(/^["']|["']$/g, "")).filter(Boolean);
            }
            // Start of block list (value is empty, next lines are - items)
            else if (val === "" || val === "[]") {
                fm[key] = [];
                currentKey = key;
                inList = true;
            }
            // Boolean
            else if (val === "true" || val === "false") {
                fm[key] = val === "true";
            }
            // Quoted string
            else if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
                fm[key] = val.slice(1, -1);
            }
            // Plain value
            else {
                fm[key] = val;
            }
        }

        return { fm, body };
    }

    /**
     * Serialize frontmatter object + body into full markdown text.
     */
    function serializeFrontmatter(fm, body) {
        if (!fm || Object.keys(fm).length === 0) return body;

        const lines = ["---"];
        // Output in a stable order
        const order = ["title", "description", "tags", "date", "last_mod", "visibility"];
        const written = new Set();

        for (const key of order) {
            if (fm[key] === undefined || fm[key] === null) continue;
            lines.push(fmField(key, fm[key]));
            written.add(key);
        }
        // Any extra keys not in the standard order
        for (const key of Object.keys(fm)) {
            if (written.has(key)) continue;
            lines.push(fmField(key, fm[key]));
        }
        lines.push("---");
        return lines.join("\n") + "\n" + body;
    }

    function fmField(key, value) {
        if (Array.isArray(value)) {
            return `${key}: [${value.join(", ")}]`;
        }
        if (typeof value === "boolean") {
            return `${key}: ${value}`;
        }
        // Quote strings containing special chars
        const str = String(value);
        if (/[:#{}[\],&*?|>\-!@`]/.test(str) || str !== str.trim()) {
            return `${key}: "${str.replace(/"/g, '\\"')}"`;
        }
        return `${key}: ${str}`;
    }

    /**
     * Read frontmatter from the current editor content.
     * Returns the fm object (never null — creates defaults if absent).
     */
    function getCurrentFrontmatter() {
        const { fm } = parseFrontmatter(editor.value);
        return fm || {};
    }

    /**
     * Get the body (content without frontmatter) from the editor.
     */
    function getEditorBody() {
        return parseFrontmatter(editor.value).body;
    }

    /**
     * Update frontmatter in the editor textarea, preserving cursor position.
     */
    function updateEditorFrontmatter(fm) {
        const body = getEditorBody();
        const cursorPos = editor.selectionStart;
        const oldLen = editor.value.length;
        editor.value = serializeFrontmatter(fm, body);
        // Adjust cursor position by the length difference
        const delta = editor.value.length - oldLen;
        editor.selectionStart = editor.selectionEnd = Math.max(0, cursorPos + delta);
        currentFrontmatter = fm;
        updateModified();
    }

    /**
     * Ensure frontmatter exists for current file. Creates with defaults if absent.
     */
    function ensureFrontmatter() {
        let fm = getCurrentFrontmatter();
        if (Object.keys(fm).length === 0) {
            fm = {
                title: currentFileName,
                description: "",
                tags: [...currentFileTags],
                date: new Date().toISOString().slice(0, 10),
                last_mod: new Date().toISOString().slice(0, 10),
                visibility: "public",
            };
            updateEditorFrontmatter(fm);
        }
        return fm;
    }

    /**
     * Populate the properties panel fields from frontmatter.
     */
    function populatePropertiesPanel(fm) {
        if (!fm) fm = {};
        fmTitle.value = fm.title || "";
        fmDescription.value = fm.description || "";
        fmVisibility.value = fm.visibility || "public";
        fmDate.textContent = fm.date || "—";
        fmLastMod.textContent = fm.last_mod || "—";
    }

    function isImageExtension(ext) {
        return IMAGE_EXTS.has(ext.toLowerCase());
    }

    // ═══════════════════════════════════════════════════════════
    // Tags
    // ═══════════════════════════════════════════════════════════
    async function loadAllTags() {
        try {
            allTags = await apiGet("/tags");
        } catch (e) {
            console.error("Failed to load tags:", e);
            allTags = [];
        }
        renderSidebarTags();
        updateTagSuggestions();
    }

    /**
     * Extract tags from frontmatter and sync them to backend metadata.
     * Returns the normalised tag array, or null if no FM / no tags.
     */
    function extractFrontmatterTags() {
        if (currentFileExt !== "md") return null;
        const { fm } = parseFrontmatter(editor.value);
        if (!fm) return null;
        if (!Array.isArray(fm.tags)) return null;
        return fm.tags.map(t => String(t).trim().toLowerCase()).filter(Boolean);
    }

    /**
     * Push tags to backend metadata for the current file and refresh the
     * global tag list so the sidebar + search stay in sync.
     */
    async function syncTagsToBackend(tags) {
        if (!currentFileId || !Array.isArray(tags)) return;
        try {
            await apiPut(`/files/${currentFileId}`, { tags });
        } catch (e) {
            console.warn("Failed to sync tags to backend:", e);
        }
        await loadAllTags();
    }

    function renderSidebarTags() {
        if (!sidebarTags) return;
        if (allTags.length === 0) {
            sidebarTags.innerHTML = "";
            return;
        }
        sidebarTags.innerHTML = `
            <div class="sidebar-tags-header">Tags</div>
            <div class="sidebar-tags-list">
                ${allTags.map(t => `<button class="sidebar-tag-chip" data-tag="${escapeHtml(t)}">${escapeHtml(t)}</button>`).join("")}
            </div>
        `;
    }

    function updateTagSuggestions() {
        if (!tagSuggestions) return;
        tagSuggestions.innerHTML = allTags
            .map(t => `<option value="${escapeHtml(t)}">`)
            .join("");
    }

    function renderFileTags() {
        if (!fileTags) return;
        if (currentFileTags.length === 0) {
            fileTags.innerHTML = '<span class="tag-empty">No tags</span>';
            return;
        }
        fileTags.innerHTML = currentFileTags.map(t => `
            <span class="tag-chip">
                ${escapeHtml(t)}
                <button class="tag-chip-remove" data-tag="${escapeHtml(t)}" title="Remove tag" aria-label="Remove tag ${escapeHtml(t)}">&times;</button>
            </span>
        `).join("");
    }

    async function addTagToFile(tag) {
        tag = tag.trim().toLowerCase();
        if (!tag || !currentFileId) return;
        if (currentFileTags.includes(tag)) {
            showToast(`Tag "${tag}" already exists`, "info");
            return;
        }

        currentFileTags.push(tag);
        renderFileTags();

        // Update frontmatter first (source of truth for .md)
        if (currentFileExt === "md") {
            const fm = ensureFrontmatter();
            fm.tags = [...currentFileTags];
            updateEditorFrontmatter(fm);
        }

        // Push to backend
        try {
            const file = await apiPut(`/files/${currentFileId}`, { tags: currentFileTags });
            currentFileTags = file.tags || currentFileTags;
            renderFileTags();
            await loadAllTags();
            showToast(`Added tag "${tag}"`, "success");
        } catch (e) {
            // Roll back
            currentFileTags = currentFileTags.filter(t => t !== tag);
            renderFileTags();
            showToast(`Failed to add tag: ${e.message}`, "error");
        }
    }

    async function removeTagFromFile(tag) {
        if (!currentFileId) return;
        const oldTags = [...currentFileTags];
        currentFileTags = currentFileTags.filter(t => t !== tag);
        renderFileTags();

        // Update frontmatter first (source of truth for .md)
        if (currentFileExt === "md") {
            const fm = getCurrentFrontmatter();
            if (Object.keys(fm).length > 0) {
                fm.tags = [...currentFileTags];
                updateEditorFrontmatter(fm);
            }
        }

        // Push to backend
        try {
            const file = await apiPut(`/files/${currentFileId}`, { tags: currentFileTags });
            currentFileTags = file.tags || currentFileTags;
            renderFileTags();
            await loadAllTags();
            showToast(`Removed tag "${tag}"`, "success");
        } catch (e) {
            // Roll back
            currentFileTags = oldTags;
            renderFileTags();
            showToast(`Failed to remove tag: ${e.message}`, "error");
        }
    }

    // ═══════════════════════════════════════════════════════════
    // File CRUD
    // ═══════════════════════════════════════════════════════════
    async function loadFileList() {
        try {
            fileList = await apiGet("/files");
            renderFileTree();
        } catch (e) {
            console.error("Failed to load file list:", e);
            fileList = [];
            renderFileTree();
        }
    }

    function renderFileTree() {
        fileTree.innerHTML = "";

        // Filter by search
        const searchVal = ($("#file-search")?.value || "").toLowerCase();
        const filtered = searchVal
            ? fileList.filter(f => `${f.name}.${f.ext}`.toLowerCase().includes(searchVal))
            : fileList;

        if (filtered.length === 0) {
            fileTree.innerHTML = `
                <div class="file-tree-empty">
                    <p>No files${searchVal ? " matching" : " yet"}</p>
                </div>
            `;
            return;
        }

        for (const file of filtered) {
            const active = file.id === currentFileId ? " active" : "";
            const isImg = isImageExtension(file.ext);
            const btn = document.createElement("button");
            btn.className = `file-tree-item${active}`;
            btn.dataset.fileId = file.id;
            btn.title = `${file.name}.${file.ext}`;

            const icon = isImg
                ? `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>`
                : `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>`;

            btn.innerHTML = `
                ${icon}
                <span class="file-name">${escapeHtml(file.name)}</span>
                <span class="file-ext">.${escapeHtml(file.ext)}</span>
                <button class="file-tree-delete" data-file-id="${file.id}" title="Delete file" aria-label="Delete ${file.name}.${file.ext}">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                </button>
            `;
            fileTree.appendChild(btn);
        }
    }

    async function openFile(fileId) {
        if (isModified && currentFileId) {
            if (!confirm("You have unsaved changes. Discard them?")) return;
        }

        try {
            const file = await apiGet(`/files/${fileId}`);
            currentFileId = file.id;
            currentFileName = file.name;
            currentFileExt = file.ext;
            currentFileTags = file.tags || [];
            currentFileMime = file.mime || "text/plain";
            isImageFile = isImageExtension(currentFileExt);

            if (isImageFile) {
                // Show image viewer, hide editor panes
                showPane("image");
                imagePreview.src = `/api/files/${fileId}/raw`;
                imageZoom = 1;
                imagePreview.style.transform = "";
                $$(".toolbar-btn[data-action]").forEach(b => b.disabled = true);
                statusType.textContent = currentFileMime;
                currentFrontmatter = null;
                populatePropertiesPanel(null);
            } else {
                // Show editor, hide image viewer
                showPane("editor");
                editor.value = file.content || "";
                savedContent = file.content || "";
                isModified = false;
                $$(".toolbar-btn[data-action]").forEach(b => b.disabled = false);
                statusType.textContent = "Markdown";

                // Parse frontmatter — it is the source of truth for tags
                const { fm } = parseFrontmatter(editor.value);
                currentFrontmatter = fm;
                if (fm && Array.isArray(fm.tags)) {
                    currentFileTags = fm.tags.map(t => String(t).trim().toLowerCase()).filter(Boolean);
                    // Sync FM tags → backend metadata so sidebar/search work
                    await syncTagsToBackend(currentFileTags);
                }
                populatePropertiesPanel(fm);
                updateAll();
            }

            btnDownload.disabled = false;
            updateStatusFileName();
            renderFileTags();
            renderFileTree();
        } catch (e) {
            console.error("Failed to open file:", e);
            showToast(`Failed to open file: ${e.message}`, "error");
        }
    }

    async function createFile() {
        const name = prompt("File name:", "untitled");
        if (!name || !name.trim()) return;

        const ext = prompt("Extension:", "md");
        if (!ext || !ext.trim()) return;

        try {
            // For new markdown files, create with frontmatter
            const isMd = ext.trim() === "md";
            const today = new Date().toISOString().slice(0, 10);
            const initialContent = isMd
                ? serializeFrontmatter({
                    title: name.trim(),
                    description: "",
                    tags: [],
                    date: today,
                    last_mod: today,
                    visibility: "public",
                }, "")
                : "";

            const file = await apiPost("/files", {
                name: name.trim(),
                ext: ext.trim(),
                mime: isMd ? "text/markdown" : "text/plain",
                content: initialContent,
            });

            currentFileId = file.id;
            currentFileName = file.name;
            currentFileExt = file.ext;
            currentFileTags = file.tags || [];
            currentFileMime = file.mime || "text/plain";
            isImageFile = false;

            showPane("editor");
            editor.value = file.content || "";
            savedContent = file.content || "";
            isModified = false;
            btnDownload.disabled = false;
            $$(".toolbar-btn[data-action]").forEach(b => b.disabled = false);
            statusType.textContent = "Markdown";

            // Parse and display frontmatter
            const { fm } = parseFrontmatter(editor.value);
            currentFrontmatter = fm;
            populatePropertiesPanel(fm);

            updateStatusFileName();
            renderFileTags();
            updateAll();
            await loadFileList();
            showToast(`Created ${file.name}.${file.ext}`, "success");
        } catch (e) {
            console.error("Failed to create file:", e);
            showToast(`Failed to create file: ${e.message}`, "error");
        }
    }

    async function saveFile() {
        if (!currentFileId) {
            await createFile();
            return;
        }
        if (isImageFile) return; // can't edit image content

        // ── Build payload ──────────────────────────────────────
        const payload = { content: editor.value };

        // For markdown: update last_mod, extract tags from frontmatter
        if (currentFileExt === "md") {
            const fm = getCurrentFrontmatter();
            if (Object.keys(fm).length > 0) {
                fm.last_mod = new Date().toISOString().slice(0, 10);
                updateEditorFrontmatter(fm);
                populatePropertiesPanel(fm);
                // Content changed after FM update
                payload.content = editor.value;
            }

            // Tags from frontmatter are the source of truth
            const fmTags = extractFrontmatterTags();
            if (fmTags) {
                payload.tags = fmTags;
                currentFileTags = fmTags;
                renderFileTags();
            } else {
                // No FM tags — use chip-managed tags
                payload.tags = [...currentFileTags];
            }
        } else {
            // Non-md file: always sync current tags
            payload.tags = [...currentFileTags];
        }

        // ── Send to backend ────────────────────────────────────
        try {
            const file = await apiPut(`/files/${currentFileId}`, payload);

            savedContent = file.content || "";
            isModified = false;
            statusModified.hidden = true;
            showToast("Saved", "success");

            // Refresh sidebar file list + global tag list
            await loadFileList();
            await loadAllTags();
        } catch (e) {
            console.error("Failed to save file:", e);
            showToast(`Failed to save: ${e.message}`, "error");
        }
    }

    async function deleteFile(fileId) {
        const file = fileList.find(f => f.id === fileId);
        const label = file ? `${file.name}.${file.ext}` : fileId;

        if (!confirm(`Delete "${label}"? This cannot be undone.`)) return;

        try {
            await apiDelete(`/files/${fileId}`);

            if (currentFileId === fileId) {
                resetEditorState();
            }

            await loadFileList();
            await loadAllTags();
            showToast(`Deleted ${label}`, "success");
        } catch (e) {
            console.error("Failed to delete file:", e);
            showToast(`Failed to delete: ${e.message}`, "error");
        }
    }

    function resetEditorState() {
        currentFileId = null;
        currentFileName = "untitled";
        currentFileExt = "md";
        currentFileTags = [];
        currentFileMime = "text/markdown";
        isImageFile = false;
        currentFrontmatter = null;
        editor.value = "";
        savedContent = "";
        isModified = false;
        btnDownload.disabled = true;
        showPane("editor");
        tagPanel.style.display = "none";
        $$(".toolbar-btn[data-action]").forEach(b => b.disabled = false);
        statusType.textContent = "Markdown";
        updateStatusFileName();
        renderFileTags();
        populatePropertiesPanel(null);
        updateAll();
    }

    // ═══════════════════════════════════════════════════════════
    // Rename
    // ═══════════════════════════════════════════════════════════
    function startRename() {
        if (!currentFileId) return;
        renameInput.value = currentFileName;
        renameInput.hidden = false;
        statusFile.hidden = true;
        renameInput.focus();
        renameInput.select();
    }

    async function commitRename() {
        renameInput.hidden = true;
        statusFile.hidden = false;
        const newName = renameInput.value.trim();
        if (!newName || newName === currentFileName || !currentFileId) return;

        try {
            const file = await apiPut(`/files/${currentFileId}`, { name: newName });
            currentFileName = file.name;
            updateStatusFileName();
            await loadFileList();
            showToast(`Renamed to ${file.name}.${file.ext}`, "success");
        } catch (e) {
            showToast(`Failed to rename: ${e.message}`, "error");
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Download
    // ═══════════════════════════════════════════════════════════
    function downloadCurrentFile() {
        if (!currentFileId) return;
        window.location.href = `/api/files/${currentFileId}/download`;
    }

    // ═══════════════════════════════════════════════════════════
    // Upload
    // ═══════════════════════════════════════════════════════════
    function openUploadModal() {
        pendingUploadFile = null;
        uploadInput.value = "";
        uploadTagsInput.value = "";
        uploadFileInfo.style.display = "none";
        btnConfirmUpload.disabled = true;
        uploadModal.classList.add("open");
    }

    function closeUploadModal() {
        uploadModal.classList.remove("open");
        pendingUploadFile = null;
    }

    function handleUploadFileSelect(file) {
        if (!file) return;
        pendingUploadFile = file;
        uploadFileName.textContent = file.name;
        uploadFileSize.textContent = formatBytes(file.size);
        uploadFileInfo.style.display = "";
        btnConfirmUpload.disabled = false;
    }

    async function performUpload() {
        if (!pendingUploadFile) return;

        const formData = new FormData();
        formData.append("file", pendingUploadFile);
        const tags = uploadTagsInput.value.trim();
        if (tags) {
            formData.append("tags", tags);
        }

        btnConfirmUpload.disabled = true;
        btnConfirmUpload.textContent = "Uploading...";

        try {
            const res = await fetch(`${API_BASE}/files/upload`, {
                method: "POST",
                body: formData,
            });

            if (!res.ok) {
                const err = await res.json().catch(() => ({ error: res.statusText }));
                throw new Error(err.error || res.statusText);
            }

            const file = await res.json();
            closeUploadModal();
            await loadFileList();
            await loadAllTags();
            showToast(`Uploaded ${file.name}.${file.ext}`, "success");

            // Open the uploaded file
            openFile(file.id);
        } catch (e) {
            showToast(`Upload failed: ${e.message}`, "error");
        } finally {
            btnConfirmUpload.textContent = "Upload";
            btnConfirmUpload.disabled = false;
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Image Viewer
    // ═══════════════════════════════════════════════════════════
    function zoomImage(delta) {
        imageZoom = Math.max(0.1, Math.min(5, imageZoom + delta));
        imagePreview.style.transform = `scale(${imageZoom})`;
    }

    function resetImageZoom() {
        imageZoom = 1;
        imagePreview.style.transform = "";
    }

    // ═══════════════════════════════════════════════════════════
    // Toast Notifications
    // ═══════════════════════════════════════════════════════════
    function showToast(message, type = "info") {
        let container = $("#toast-container");
        if (!container) {
            container = document.createElement("div");
            container.id = "toast-container";
            document.body.appendChild(container);
        }

        const toast = document.createElement("div");
        toast.className = `toast toast-${type}`;
        toast.textContent = message;
        container.appendChild(toast);

        requestAnimationFrame(() => toast.classList.add("show"));

        setTimeout(() => {
            toast.classList.remove("show");
            toast.addEventListener("transitionend", () => toast.remove());
        }, 2500);
    }

    function updateStatusFileName() {
        if (statusFileLabel) {
            statusFileLabel.textContent = `${currentFileName}.${currentFileExt}`;
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Theme
    // ═══════════════════════════════════════════════════════════
    const THEMES = [
        { id: "nord",     label: "Nord",          icon: "❄" },
        { id: "matrix",   label: "Matrix",        icon: "⌨" },
        { id: "nerd",     label: "Nerd",          icon: "🤓" },
        { id: "contrast", label: "High Contrast", icon: "◑" },
        { id: "pinky",    label: "Pinky",         icon: "🌸" },
    ];

    function initTheme() {
        const saved = localStorage.getItem("synker-theme");
        if (saved) {
            document.documentElement.dataset.theme = saved;
        }
        updateThemeUI();
    }

    function setTheme(themeId) {
        document.documentElement.dataset.theme = themeId;
        localStorage.setItem("synker-theme", themeId);
        updateThemeUI();
    }

    function updateThemeUI() {
        const current = document.documentElement.dataset.theme || "nord";
        const theme = THEMES.find(t => t.id === current) || THEMES[0];

        const btnIcon = $("#theme-btn-icon");
        const btnLabel = $("#theme-btn-label");
        if (btnIcon) btnIcon.textContent = theme.icon;
        if (btnLabel) btnLabel.textContent = theme.label;

        // Update active state in menu
        $$("#theme-menu .theme-option").forEach(opt => {
            opt.classList.toggle("active", opt.dataset.themeValue === current);
        });
    }

    function toggleThemeMenu() {
        const menu = $("#theme-menu");
        const btn = $("#theme-toggle");
        if (!menu) return;
        const isOpen = menu.classList.contains("open");
        menu.classList.toggle("open", !isOpen);
        if (btn) btn.setAttribute("aria-expanded", !isOpen);
    }

    function closeThemeMenu() {
        const menu = $("#theme-menu");
        const btn = $("#theme-toggle");
        if (menu) menu.classList.remove("open");
        if (btn) btn.setAttribute("aria-expanded", "false");
    }

    // ═══════════════════════════════════════════════════════════
    // Line Numbers
    // ═══════════════════════════════════════════════════════════
    function updateLineNumbers() {
        const text = editor.value;
        const lines = text.split("\n").length;
        const cursorLine = getCurrentLine();

        let html = "";
        for (let i = 1; i <= lines; i++) {
            const active = i === cursorLine ? " active" : "";
            html += `<div class="line-number${active}">${i}</div>`;
        }
        lineNumbers.innerHTML = html;
    }

    function syncLineNumberScroll() {
        lineNumbers.scrollTop = editor.scrollTop;
    }

    // ═══════════════════════════════════════════════════════════
    // Cursor & Stats
    // ═══════════════════════════════════════════════════════════
    function getCurrentLine() {
        const pos = editor.selectionStart;
        return editor.value.substring(0, pos).split("\n").length;
    }

    function getCurrentCol() {
        const pos = editor.selectionStart;
        const textBefore = editor.value.substring(0, pos);
        const lastNewline = textBefore.lastIndexOf("\n");
        return pos - lastNewline;
    }

    function updateCursorStatus() {
        const ln = getCurrentLine();
        const col = getCurrentCol();
        statusCursor.textContent = `Ln ${ln}, Col ${col}`;
    }

    function updateStats() {
        const text = editor.value;
        charCount.textContent = `${text.length} chars`;

        const words = text.trim() === ""
            ? 0
            : text.trim().split(/\s+/).length;
        wordCount.textContent = `${words} word${words !== 1 ? "s" : ""}`;
    }

    function updateModified() {
        isModified = editor.value !== savedContent;
        statusModified.hidden = !isModified;
    }

    // ═══════════════════════════════════════════════════════════
    // Markdown Preview  (delegates to SynkerMD — see markdown.js)
    // ═══════════════════════════════════════════════════════════

    function updatePreview() {
        const body = getEditorBody();
        preview.innerHTML = window.SynkerMD.render(body);

        // Syntax-highlight all code blocks in the preview
        if (window.hljs) {
            preview.querySelectorAll("pre code").forEach(block => {
                const langClass = [...block.classList].find(c => c.startsWith("language-"));
                const lang = langClass ? langClass.replace("language-", "") : null;
                const code = block.textContent;
                try {
                    let result;
                    if (lang && hljs.getLanguage(lang)) {
                        result = hljs.highlight(code, { language: lang });
                    } else {
                        result = hljs.highlightAuto(code);
                    }
                    block.innerHTML = result.value;
                    block.classList.add("hljs");
                } catch (e) {
                    // Silently skip — show unhighlighted code
                }
            });
        }
    }

    // Handle internal markdown link clicks in preview
    preview.addEventListener("click", (e) => {
        const link = e.target.closest(".md-internal-link");
        if (!link) return;
        e.preventDefault();

        const target = link.dataset.target;
        if (!target) return;

        const targetClean = target.replace(/\.md$/, "").toLowerCase();

        const match = fileList.find(f => {
            const full = `${f.name}.${f.ext}`.toLowerCase();
            const nameOnly = f.name.toLowerCase();
            return full === targetClean || nameOnly === targetClean ||
                   full === target.toLowerCase() || nameOnly === target.toLowerCase();
        });

        if (match) {
            openFile(match.id);
        } else {
            showToast(`Page not found: ${target}`, "error");
        }
    });

    // ═══════════════════════════════════════════════════════════
    // Toolbar Actions
    // ═══════════════════════════════════════════════════════════
    const ACTIONS = {
        heading:       { prefix: "# ",    suffix: "",   placeholder: "Heading" },
        bold:          { prefix: "**",     suffix: "**", placeholder: "bold text" },
        italic:        { prefix: "*",      suffix: "*",  placeholder: "italic text" },
        strikethrough: { prefix: "~~",     suffix: "~~", placeholder: "strikethrough" },
        code:          { prefix: "`",      suffix: "`",  placeholder: "code" },
        link:          { prefix: "[",      suffix: "](url)", placeholder: "link text" },
        image:         { prefix: "![",    suffix: "](url)", placeholder: "alt text" },
        blockquote:    { prefix: "> ",     suffix: "",   placeholder: "quote" },
        ul:            { prefix: "- ",     suffix: "",   placeholder: "list item" },
        ol:            { prefix: "1. ",    suffix: "",   placeholder: "list item" },
        checklist:     { prefix: "- [ ] ", suffix: "",   placeholder: "task" },
        hr:            { prefix: "\n---\n", suffix: "",  placeholder: "" },
        codeblock:     { prefix: "\n```\n", suffix: "\n```\n", placeholder: "code" },
    };

    function applyAction(action) {
        const spec = ACTIONS[action];
        if (!spec) return;

        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        const selected = editor.value.substring(start, end);
        const text = selected || spec.placeholder;

        const before = editor.value.substring(0, start);
        const after = editor.value.substring(end);

        editor.value = before + spec.prefix + text + spec.suffix + after;

        const newStart = start + spec.prefix.length;
        const newEnd = newStart + text.length;
        editor.setSelectionRange(newStart, newEnd);
        editor.focus();

        onEditorInput();
    }

    // ═══════════════════════════════════════════════════════════
    // View Mode
    // ═══════════════════════════════════════════════════════════
    function setViewMode(mode) {
        editorPanes.dataset.view = mode;
        viewModeSelect.value = mode;

        if (mode === "preview" || mode === "split") {
            updatePreview();
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Sidebar Toggle
    // ═══════════════════════════════════════════════════════════
    function toggleSidebar() {
        const collapsed = sidebar.dataset.collapsed === "true";
        sidebar.dataset.collapsed = !collapsed ? "true" : "false";
    }

    // ═══════════════════════════════════════════════════════════
    // Pane Resizer
    // ═══════════════════════════════════════════════════════════
    function initResizer() {
        let isResizing = false;

        paneResizer.addEventListener("pointerdown", (e) => {
            isResizing = true;
            paneResizer.classList.add("active");
            paneResizer.setPointerCapture(e.pointerId);
            document.body.style.cursor = "col-resize";
            document.body.style.userSelect = "none";
        });

        document.addEventListener("pointermove", (e) => {
            if (!isResizing) return;

            const container = editorPanes.getBoundingClientRect();
            const offset = e.clientX - container.left;
            const pct = (offset / container.width) * 100;
            const clamped = Math.max(20, Math.min(80, pct));

            const editorPane = $("#pane-editor");
            const previewPane = $("#pane-preview");
            editorPane.style.flex = `0 0 ${clamped}%`;
            previewPane.style.flex = `0 0 ${100 - clamped}%`;
        });

        document.addEventListener("pointerup", () => {
            if (!isResizing) return;
            isResizing = false;
            paneResizer.classList.remove("active");
            document.body.style.cursor = "";
            document.body.style.userSelect = "";
        });
    }

    // ═══════════════════════════════════════════════════════════
    // Keyboard Shortcuts
    // ═══════════════════════════════════════════════════════════
    function handleKeyboard(e) {
        if (!e.ctrlKey && !e.metaKey) return;

        const shortcuts = {
            "b": "bold",
            "i": "italic",
            "k": "link",
            "h": "heading",
            "`": "code",
        };

        if (e.key === "p") {
            e.preventDefault();
            const current = editorPanes.dataset.view;
            setViewMode(current === "split" ? "preview" : "split");
            return;
        }

        if (e.key === "s") {
            e.preventDefault();
            saveFile();
            return;
        }

        const action = shortcuts[e.key];
        if (action) {
            e.preventDefault();
            applyAction(action);
        }
    }

    function handleTab(e) {
        if (e.key !== "Tab") return;
        e.preventDefault();

        const start = editor.selectionStart;
        const end = editor.selectionEnd;

        if (e.shiftKey) {
            const before = editor.value.substring(0, start);
            const selected = editor.value.substring(start, end);
            const after = editor.value.substring(end);
            const outdented = selected.replace(/^(\t|    )/gm, "");
            editor.value = before + outdented + after;
            editor.setSelectionRange(start, start + outdented.length);
        } else {
            editor.value =
                editor.value.substring(0, start) +
                "\t" +
                editor.value.substring(end);
            editor.setSelectionRange(start + 1, start + 1);
        }

        onEditorInput();
    }

    // ═══════════════════════════════════════════════════════════
    // On Editor Input
    // ═══════════════════════════════════════════════════════════
    let previewTimer;
    let fmSyncTimer;

    function onEditorInput() {
        updateLineNumbers();
        updateStats();
        updateModified();
        updateCursorStatus();

        clearTimeout(previewTimer);
        previewTimer = setTimeout(updatePreview, 150);

        // Debounced re-parse of frontmatter to live-sync tags & properties
        clearTimeout(fmSyncTimer);
        fmSyncTimer = setTimeout(syncFrontmatterToUI, 300);
    }

    /**
     * Re-parse frontmatter from the editor and sync tags + properties panel.
     * Called on a debounce timer so typing is not interrupted.
     */
    function syncFrontmatterToUI() {
        if (currentFileExt !== "md") return;
        const { fm } = parseFrontmatter(editor.value);
        if (!fm) return;
        currentFrontmatter = fm;

        // Sync tags from frontmatter → tag chips
        if (Array.isArray(fm.tags)) {
            const fmTags = fm.tags.map(t => t.trim().toLowerCase()).filter(Boolean);
            const changed = fmTags.length !== currentFileTags.length ||
                            fmTags.some((t, i) => t !== currentFileTags[i]);
            if (changed) {
                currentFileTags = fmTags;
                renderFileTags();
            }
        }

        // Sync other properties to the panel
        populatePropertiesPanel(fm);
    }

    function updateAll() {
        updateLineNumbers();
        updateStats();
        updateModified();
        updateCursorStatus();
        updatePreview();
    }

    // ═══════════════════════════════════════════════════════════
    // Initialization
    // ═══════════════════════════════════════════════════════════
    function init() {
        initTheme();
        updateLineNumbers();
        updateStats();
        updateCursorStatus();
        initResizer();

        // Editor events
        editor.addEventListener("input", onEditorInput);
        editor.addEventListener("click", () => {
            updateCursorStatus();
            updateLineNumbers();
        });
        editor.addEventListener("keyup", (e) => {
            if (["ArrowUp","ArrowDown","ArrowLeft","ArrowRight","Home","End"].includes(e.key)) {
                updateCursorStatus();
                updateLineNumbers();
            }
        });
        editor.addEventListener("scroll", syncLineNumberScroll);
        editor.addEventListener("keydown", handleTab);

        // Keyboard shortcuts
        document.addEventListener("keydown", handleKeyboard);

        // Toolbar actions
        $$(".toolbar-btn[data-action]").forEach(btn => {
            btn.addEventListener("click", () => {
                const action = btn.dataset.action;
                if (action === "toggle-preview") {
                    const current = editorPanes.dataset.view;
                    setViewMode(current === "split" ? "preview" : "split");
                } else {
                    applyAction(action);
                }
            });
        });

        // View mode selector
        viewModeSelect.addEventListener("change", (e) => {
            setViewMode(e.target.value);
        });

        // Sidebar toggle
        btnToggleSidebar.addEventListener("click", toggleSidebar);

        // New file button
        btnNewFile.addEventListener("click", createFile);

        // File tree clicks (delegation)
        fileTree.addEventListener("click", (e) => {
            const deleteBtn = e.target.closest(".file-tree-delete");
            if (deleteBtn) {
                e.stopPropagation();
                deleteFile(deleteBtn.dataset.fileId);
                return;
            }
            const item = e.target.closest(".file-tree-item");
            if (item && item.dataset.fileId) {
                openFile(item.dataset.fileId);
            }
        });

        // File search filter
        const fileSearch = $("#file-search");
        if (fileSearch) {
            fileSearch.addEventListener("input", () => renderFileTree());
        }

        // Theme switcher
        themeToggle.addEventListener("click", (e) => {
            e.stopPropagation();
            toggleThemeMenu();
        });
        const themeMenu = $("#theme-menu");
        if (themeMenu) {
            themeMenu.addEventListener("click", (e) => {
                const opt = e.target.closest(".theme-option");
                if (opt && opt.dataset.themeValue) {
                    setTheme(opt.dataset.themeValue);
                    closeThemeMenu();
                }
            });
        }
        document.addEventListener("click", (e) => {
            if (!e.target.closest("#theme-switcher")) closeThemeMenu();
        });

        // ─── Download ───
        btnDownload.addEventListener("click", downloadCurrentFile);

        // ─── Upload ───
        btnUploadFile.addEventListener("click", openUploadModal);
        btnCancelUpload.addEventListener("click", closeUploadModal);
        btnCloseUpload.addEventListener("click", closeUploadModal);
        btnConfirmUpload.addEventListener("click", performUpload);

        uploadInput.addEventListener("change", (e) => {
            handleUploadFileSelect(e.target.files[0]);
        });

        uploadDropzone.addEventListener("dragover", (e) => {
            e.preventDefault();
            uploadDropzone.classList.add("dragover");
        });
        uploadDropzone.addEventListener("dragleave", () => {
            uploadDropzone.classList.remove("dragover");
        });
        uploadDropzone.addEventListener("drop", (e) => {
            e.preventDefault();
            uploadDropzone.classList.remove("dragover");
            const file = e.dataTransfer.files[0];
            if (file) handleUploadFileSelect(file);
        });

        uploadModal.addEventListener("click", (e) => {
            if (e.target === uploadModal) closeUploadModal();
        });

        // ─── Rename ───
        statusFile.addEventListener("click", startRename);
        renameInput.addEventListener("blur", commitRename);
        renameInput.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                commitRename();
            }
            if (e.key === "Escape") {
                renameInput.hidden = true;
                statusFile.hidden = false;
            }
        });

        // ─── Tags ───
        btnTagsToggle.addEventListener("click", () => {
            const isHidden = tagPanel.style.display === "none" || tagPanel.style.display === "";
            tagPanel.style.display = isHidden ? "block" : "none";
        });
        btnCloseTags.addEventListener("click", () => {
            tagPanel.style.display = "none";
        });

        btnAddTag.addEventListener("click", () => {
            const tag = tagInput.value.trim();
            if (tag) {
                addTagToFile(tag);
                tagInput.value = "";
            }
        });
        tagInput.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                e.preventDefault();
                const tag = tagInput.value.trim();
                if (tag) {
                    addTagToFile(tag);
                    tagInput.value = "";
                }
            }
        });

        // Tag chip removal (delegation)
        fileTags.addEventListener("click", (e) => {
            const removeBtn = e.target.closest(".tag-chip-remove");
            if (removeBtn) {
                removeTagFromFile(removeBtn.dataset.tag);
            }
        });

        // ─── Properties panel field changes → update frontmatter ───
        function onPropertyChange() {
            if (!currentFileId || currentFileExt !== "md") return;
            const fm = ensureFrontmatter();
            fm.title = fmTitle.value;
            fm.description = fmDescription.value;
            fm.visibility = fmVisibility.value;
            updateEditorFrontmatter(fm);
        }

        fmTitle.addEventListener("change", onPropertyChange);
        fmDescription.addEventListener("change", onPropertyChange);
        fmVisibility.addEventListener("change", onPropertyChange);

        // Sidebar tag clicks — filter file tree by tag
        sidebarTags.addEventListener("click", (e) => {
            const chip = e.target.closest(".sidebar-tag-chip");
            if (chip) {
                const tag = chip.dataset.tag;
                // Toggle active state to filter
                const isActive = chip.classList.contains("active");
                $$(".sidebar-tag-chip", sidebarTags).forEach(c => c.classList.remove("active"));
                if (!isActive) {
                    chip.classList.add("active");
                    // Filter fileList by tag
                    const filtered = fileList.filter(f => (f.tags || []).includes(tag));
                    renderFilteredFileTree(filtered);
                } else {
                    renderFileTree();
                }
            }
        });

        // ─── Image Viewer ───
        btnZoomIn.addEventListener("click", () => zoomImage(0.25));
        btnZoomOut.addEventListener("click", () => zoomImage(-0.25));
        btnZoomReset.addEventListener("click", resetImageZoom);

        // Load data on boot
        Promise.all([loadFileList(), loadAllTags()]).then(() => {
            const params = new URLSearchParams(window.location.search);
            const fileId = params.get("file");
            if (fileId) {
                openFile(fileId);
            }
        });
    }

    function renderFilteredFileTree(files) {
        fileTree.innerHTML = "";
        if (files.length === 0) {
            fileTree.innerHTML = `<div class="file-tree-empty"><p>No matching files</p></div>`;
            return;
        }
        for (const file of files) {
            const active = file.id === currentFileId ? " active" : "";
            const isImg = isImageExtension(file.ext);
            const btn = document.createElement("button");
            btn.className = `file-tree-item${active}`;
            btn.dataset.fileId = file.id;
            btn.title = `${file.name}.${file.ext}`;

            const icon = isImg
                ? `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>`
                : `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>`;

            btn.innerHTML = `
                ${icon}
                <span class="file-name">${escapeHtml(file.name)}</span>
                <span class="file-ext">.${escapeHtml(file.ext)}</span>
                <button class="file-tree-delete" data-file-id="${file.id}" title="Delete file" aria-label="Delete ${file.name}.${file.ext}">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                </button>
            `;
            fileTree.appendChild(btn);
        }
    }

    // Boot
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
