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

    // Wiki-link autocomplete
    const linkAutocomplete = $("#link-autocomplete");
    const linkAutocompleteList = $("#link-autocomplete-list");

    // Wiki-link toolbar file picker
    const btnWikiLink       = $("#btn-wiki-link");
    const linkPicker        = $("#link-picker");
    const linkPickerSearch  = $("#link-picker-search");
    const linkPickerList    = $("#link-picker-list");

    // Image-link autocomplete
    const imgAutocomplete     = $("#image-autocomplete");
    const imgAutocompleteList = $("#image-autocomplete-list");

    // Image-link toolbar file picker
    const btnImagePicker    = $("#btn-image-picker");
    const imagePicker       = $("#image-picker");
    const imagePickerSearch = $("#image-picker-search");
    const imagePickerList   = $("#image-picker-list");

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

    // Wiki-link autocomplete state
    let acActive = false;       // autocomplete is visible
    let acStartPos = -1;        // position of the opening "[[" in editor
    let acSelectedIdx = 0;      // highlighted item index
    let acMatches = [];         // filtered file list for autocomplete

    // Image-link autocomplete state  (![[…]])
    let imgAcActive = false;
    let imgAcStartPos = -1;     // position of the opening "![[" in editor
    let imgAcSelectedIdx = 0;
    let imgAcMatches = [];

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

    /** Get display name for a file: title if available, otherwise file name */
    function fileDisplayName(file) {
        return file.title || file.name;
    }

    function renderFileTree() {
        fileTree.innerHTML = "";

        // Filter by search (match both title and filename)
        const searchVal = ($("#file-search")?.value || "").toLowerCase();
        const filtered = searchVal
            ? fileList.filter(f => {
                  const haystack = `${f.title || ""} ${f.name}.${f.ext}`.toLowerCase();
                  return haystack.includes(searchVal);
              })
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

            const displayName = fileDisplayName(file);
            const hasTitle = !!file.title;

            const icon = isImg
                ? `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>`
                : `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>`;

            btn.innerHTML = `
                ${icon}
                <span class="file-name">${escapeHtml(displayName)}</span>
                ${hasTitle ? `<span class="file-ext file-ext-subtle" title="${escapeHtml(file.name)}.${escapeHtml(file.ext)}">.${escapeHtml(file.ext)}</span>` : `<span class="file-ext">.${escapeHtml(file.ext)}</span>`}
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

                // Async backlinks — don't block file open
                if (currentFileExt === "md") {
                    findBacklinks(currentFileName).then(renderBacklinks);
                }
            }

            btnDownload.disabled = false;
            updateStatusFileName();
            renderFileTags();
            renderFileTree();

            // On mobile, close the sidebar after opening a file
            if (window.matchMedia("(max-width: 768px)").matches) {
                sidebar.dataset.collapsed = "true";
                const overlay = $("#sidebar-overlay");
                if (overlay) overlay.classList.remove("visible");
            }
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
            // Show title if the current file has one (from frontmatter or file list)
            const fileEntry = fileList.find(f => f.id === currentFileId);
            const displayTitle = (currentFrontmatter && currentFrontmatter.title)
                || (fileEntry && fileEntry.title)
                || null;
            if (displayTitle) {
                statusFileLabel.textContent = displayTitle;
                statusFileLabel.title = `${currentFileName}.${currentFileExt}`;
            } else {
                statusFileLabel.textContent = `${currentFileName}.${currentFileExt}`;
                statusFileLabel.title = "";
            }
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

        // Annotate internal links (mark broken ones)
        annotateInternalLinks();
    }

    // ═══════════════════════════════════════════════════════════
    // Wiki-Link Autocomplete  ([[…]] as-you-type suggestions)
    // ═══════════════════════════════════════════════════════════

    /**
     * Get markdown-only files sorted alphabetically for link suggestions.
     */
    function getLinkableFiles() {
        return fileList
            .filter(f => f.ext === "md")
            .sort((a, b) => a.name.localeCompare(b.name));
    }

    /**
     * Open the autocomplete popup at the cursor position.
     */
    function acOpen(startPos) {
        acActive = true;
        acStartPos = startPos;
        acSelectedIdx = 0;
        acUpdate();
        linkAutocomplete.hidden = false;
        positionAutocomplete();
    }

    /**
     * Close the autocomplete popup.
     */
    function acClose() {
        acActive = false;
        acStartPos = -1;
        acMatches = [];
        linkAutocomplete.hidden = true;
    }

    /**
     * Update the match list based on what the user has typed after "[[".
     */
    function acUpdate() {
        const query = editor.value.slice(acStartPos + 2, editor.selectionStart).toLowerCase();
        const files = getLinkableFiles();
        acMatches = query
            ? files.filter(f => f.name.toLowerCase().includes(query))
            : files;
        acSelectedIdx = Math.min(acSelectedIdx, Math.max(0, acMatches.length - 1));
        acRender();
    }

    /**
     * Render the autocomplete list items.
     */
    function acRender() {
        if (acMatches.length === 0) {
            linkAutocompleteList.innerHTML = `<div class="link-ac-empty">No matching pages</div>`;
            return;
        }
        linkAutocompleteList.innerHTML = acMatches.map((f, i) => {
            const active = i === acSelectedIdx ? " link-ac-item--active" : "";
            return `<div class="link-ac-item${active}" role="option" data-idx="${i}">
                <span class="link-ac-name">${escapeHtml(f.name)}</span>
                <span class="link-ac-ext">.${escapeHtml(f.ext)}</span>
            </div>`;
        }).join("");

        // Scroll active item into view
        const activeEl = linkAutocompleteList.querySelector(".link-ac-item--active");
        if (activeEl) activeEl.scrollIntoView({ block: "nearest" });
    }

    /**
     * Position the autocomplete popup near the cursor in the textarea.
     * Uses a mirror div for accurate caret coordinate calculation.
     */
    function positionAutocomplete() {
        const coords = getCaretCoordinates();
        if (coords) {
            linkAutocomplete.style.left = coords.left + "px";
            linkAutocomplete.style.top  = coords.top  + "px";
        }
    }

    /**
     * Calculate caret coordinates in the textarea using a mirror element.
     */
    function getCaretCoordinates() {
        const mirror = document.createElement("div");
        const style = getComputedStyle(editor);
        const props = [
            "fontFamily","fontSize","fontWeight","lineHeight","letterSpacing",
            "wordSpacing","textIndent","whiteSpace","wordWrap","overflowWrap",
            "paddingTop","paddingLeft","paddingRight","paddingBottom",
            "borderTopWidth","borderLeftWidth","borderRightWidth","borderBottomWidth",
            "boxSizing","tabSize"
        ];
        props.forEach(p => mirror.style[p] = style[p]);
        mirror.style.position = "absolute";
        mirror.style.visibility = "hidden";
        mirror.style.overflow = "hidden";
        mirror.style.width = editor.clientWidth + "px";
        mirror.style.height = "auto";

        const text = editor.value.substring(0, editor.selectionStart);
        mirror.textContent = text;
        const span = document.createElement("span");
        span.textContent = "|";
        mirror.appendChild(span);
        document.body.appendChild(mirror);

        const container = editor.closest(".editor-container");
        const containerRect = container.getBoundingClientRect();
        const editorRect = editor.getBoundingClientRect();
        const scrollTop = editor.scrollTop;

        const caretLeft = span.offsetLeft;
        const caretTop = span.offsetTop;

        document.body.removeChild(mirror);

        // Position relative to the editor-container
        const left = (editorRect.left - containerRect.left) + caretLeft;
        const top  = (editorRect.top  - containerRect.top)  + caretTop - scrollTop + parseInt(style.lineHeight || 20);

        return { left: Math.min(left, containerRect.width - 260), top };
    }

    /**
     * Accept the currently highlighted autocomplete suggestion.
     */
    function acAccept() {
        if (!acActive || acMatches.length === 0) return;
        const file = acMatches[acSelectedIdx];
        const before = editor.value.substring(0, acStartPos);
        const after  = editor.value.substring(editor.selectionStart);
        const insert = `[[${file.name}]]`;
        editor.value = before + insert + after;
        const newPos = acStartPos + insert.length;
        editor.setSelectionRange(newPos, newPos);
        editor.focus();
        acClose();
        onEditorInput();
    }

    /**
     * Handle input events to detect [[ and update autocomplete.
     */
    function acHandleInput() {
        const pos = editor.selectionStart;
        const val = editor.value;

        // Don't activate wiki-link autocomplete if image autocomplete is active
        if (imgAcActive) return;

        if (acActive) {
            // Check if cursor moved before the trigger or "]]" was typed
            if (pos <= acStartPos + 1 || val.substring(acStartPos, pos).includes("]]") || val.substring(acStartPos, pos).includes("\n")) {
                acClose();
                return;
            }
            acUpdate();
            positionAutocomplete();
            return;
        }

        // Detect "[[" trigger — look at the two characters just before the cursor
        // But NOT if preceded by "!" (that's an image link ![[)
        if (pos >= 2 && val[pos - 2] === "[" && val[pos - 1] === "[") {
            if (pos >= 3 && val[pos - 3] === "!") return; // image link, not wiki-link
            acOpen(pos - 2);
        }
    }

    /**
     * Handle keyboard navigation in the autocomplete popup.
     * Returns true if the event was consumed.
     */
    function acHandleKeydown(e) {
        if (!acActive) return false;

        if (e.key === "ArrowDown") {
            e.preventDefault();
            acSelectedIdx = Math.min(acSelectedIdx + 1, acMatches.length - 1);
            acRender();
            return true;
        }
        if (e.key === "ArrowUp") {
            e.preventDefault();
            acSelectedIdx = Math.max(acSelectedIdx - 1, 0);
            acRender();
            return true;
        }
        if (e.key === "Enter" || e.key === "Tab") {
            e.preventDefault();
            acAccept();
            return true;
        }
        if (e.key === "Escape") {
            e.preventDefault();
            acClose();
            return true;
        }
        return false;
    }

    // Click on autocomplete item
    linkAutocompleteList.addEventListener("click", (e) => {
        const item = e.target.closest(".link-ac-item");
        if (!item) return;
        acSelectedIdx = parseInt(item.dataset.idx, 10);
        acAccept();
    });

    // ═══════════════════════════════════════════════════════════
    // Toolbar Wiki-Link File Picker (📄+ button)
    // ═══════════════════════════════════════════════════════════

    function openLinkPicker() {
        linkPicker.hidden = false;
        linkPickerSearch.value = "";
        renderLinkPicker("");
        // Position is already relative to button via CSS
        requestAnimationFrame(() => linkPickerSearch.focus());
    }

    function closeLinkPicker() {
        linkPicker.hidden = true;
    }

    function renderLinkPicker(query) {
        const files = getLinkableFiles();
        const q = query.toLowerCase();
        const filtered = q ? files.filter(f => f.name.toLowerCase().includes(q)) : files;

        if (filtered.length === 0) {
            linkPickerList.innerHTML = `<div class="link-ac-empty">No matching pages</div>`;
            return;
        }
        linkPickerList.innerHTML = filtered.map(f =>
            `<div class="link-picker-item" data-file-id="${f.id}" data-file-name="${escapeHtml(f.name)}">
                <svg class="link-picker-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>
                <span class="link-picker-name">${escapeHtml(f.name)}</span>
            </div>`
        ).join("");
    }

    function insertWikiLink(fileName) {
        const start = editor.selectionStart;
        const end   = editor.selectionEnd;
        const selected = editor.value.substring(start, end);
        const before = editor.value.substring(0, start);
        const after  = editor.value.substring(end);

        // If text is selected, use it as display and fileName as target
        const insert = selected
            ? `[[${fileName}|${selected}]]`
            : `[[${fileName}]]`;

        editor.value = before + insert + after;
        const newPos = start + insert.length;
        editor.setSelectionRange(newPos, newPos);
        editor.focus();
        closeLinkPicker();
        onEditorInput();
    }

    btnWikiLink.addEventListener("click", (e) => {
        e.stopPropagation();
        if (linkPicker.hidden) {
            openLinkPicker();
        } else {
            closeLinkPicker();
        }
    });

    linkPickerSearch.addEventListener("input", () => {
        renderLinkPicker(linkPickerSearch.value);
    });

    linkPickerList.addEventListener("click", (e) => {
        const item = e.target.closest(".link-picker-item");
        if (!item) return;
        insertWikiLink(item.dataset.fileName);
    });

    // Close picker on outside click
    document.addEventListener("click", (e) => {
        if (!linkPicker.hidden && !e.target.closest("#link-picker") && !e.target.closest("#btn-wiki-link")) {
            closeLinkPicker();
        }
    });

    // Keyboard navigation inside link picker
    linkPickerSearch.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            closeLinkPicker();
            editor.focus();
            return;
        }
        if (e.key === "Enter") {
            const first = linkPickerList.querySelector(".link-picker-item");
            if (first) insertWikiLink(first.dataset.fileName);
            return;
        }
        if (e.key === "ArrowDown") {
            e.preventDefault();
            const first = linkPickerList.querySelector(".link-picker-item");
            if (first) first.focus();
        }
    });

    // ═══════════════════════════════════════════════════════════
    // Image-Link Autocomplete  (![[…]] as-you-type suggestions)
    // ═══════════════════════════════════════════════════════════

    /**
     * Get image files sorted alphabetically for image link suggestions.
     */
    function getImageFiles() {
        return fileList
            .filter(f => IMAGE_EXTS.has(f.ext.toLowerCase()))
            .sort((a, b) => a.name.localeCompare(b.name));
    }

    /**
     * Resolve an image link target to a file in fileList.
     * Handles: "name.jpg", "name", case-insensitively.
     */
    function resolveImageLink(target) {
        if (!target) return null;
        const t = target.trim().toLowerCase();
        // Strip any known image extension for base-name matching
        const tBase = t.replace(/\.(png|jpe?g|gif|bmp|webp|svg|ico|avif)$/i, "");
        return fileList.find(f => {
            if (!IMAGE_EXTS.has(f.ext.toLowerCase())) return false;
            const name = f.name.toLowerCase();
            const full = `${name}.${f.ext}`.toLowerCase();
            return full === t || name === t || name === tBase || full === tBase;
        }) || null;
    }

    function imgAcOpen(startPos) {
        imgAcActive = true;
        imgAcStartPos = startPos;
        imgAcSelectedIdx = 0;
        imgAcUpdate();
        imgAutocomplete.hidden = false;
        positionImgAutocomplete();
    }

    function imgAcClose() {
        imgAcActive = false;
        imgAcStartPos = -1;
        imgAcMatches = [];
        imgAutocomplete.hidden = true;
    }

    function imgAcUpdate() {
        const query = editor.value.slice(imgAcStartPos + 3, editor.selectionStart).toLowerCase();
        const files = getImageFiles();
        imgAcMatches = query
            ? files.filter(f => {
                const full = `${f.name}.${f.ext}`.toLowerCase();
                return full.includes(query) || f.name.toLowerCase().includes(query);
            })
            : files;
        imgAcSelectedIdx = Math.min(imgAcSelectedIdx, Math.max(0, imgAcMatches.length - 1));
        imgAcRender();
    }

    function imgAcRender() {
        if (imgAcMatches.length === 0) {
            imgAutocompleteList.innerHTML = `<div class="link-ac-empty">No matching images</div>`;
            return;
        }
        imgAutocompleteList.innerHTML = imgAcMatches.map((f, i) => {
            const active = i === imgAcSelectedIdx ? " link-ac-item--active" : "";
            const thumbUrl = `/api/files/${f.id}/raw`;
            return `<div class="link-ac-item${active}" role="option" data-idx="${i}">
                <img class="link-ac-thumb" src="${thumbUrl}" alt="" loading="lazy">
                <span class="link-ac-name">${escapeHtml(f.name)}</span>
                <span class="link-ac-ext">.${escapeHtml(f.ext)}</span>
            </div>`;
        }).join("");
        const activeEl = imgAutocompleteList.querySelector(".link-ac-item--active");
        if (activeEl) activeEl.scrollIntoView({ block: "nearest" });
    }

    function positionImgAutocomplete() {
        const coords = getCaretCoordinates();
        if (coords) {
            imgAutocomplete.style.left = coords.left + "px";
            imgAutocomplete.style.top  = coords.top  + "px";
        }
    }

    function imgAcAccept() {
        if (!imgAcActive || imgAcMatches.length === 0) return;
        const file = imgAcMatches[imgAcSelectedIdx];
        const before = editor.value.substring(0, imgAcStartPos);
        const after  = editor.value.substring(editor.selectionStart);
        const insert = `![[${file.name}.${file.ext}]]`;
        editor.value = before + insert + after;
        const newPos = imgAcStartPos + insert.length;
        editor.setSelectionRange(newPos, newPos);
        editor.focus();
        imgAcClose();
        onEditorInput();
    }

    function imgAcHandleInput() {
        const pos = editor.selectionStart;
        const val = editor.value;

        if (imgAcActive) {
            if (pos <= imgAcStartPos + 2 || val.substring(imgAcStartPos, pos).includes("]]") || val.substring(imgAcStartPos, pos).includes("\n")) {
                imgAcClose();
                return;
            }
            imgAcUpdate();
            positionImgAutocomplete();
            return;
        }

        // Detect "![[" trigger
        if (pos >= 3 && val[pos - 3] === "!" && val[pos - 2] === "[" && val[pos - 1] === "[") {
            imgAcOpen(pos - 3);
        }
    }

    function imgAcHandleKeydown(e) {
        if (!imgAcActive) return false;
        if (e.key === "ArrowDown") {
            e.preventDefault();
            imgAcSelectedIdx = Math.min(imgAcSelectedIdx + 1, imgAcMatches.length - 1);
            imgAcRender();
            return true;
        }
        if (e.key === "ArrowUp") {
            e.preventDefault();
            imgAcSelectedIdx = Math.max(imgAcSelectedIdx - 1, 0);
            imgAcRender();
            return true;
        }
        if (e.key === "Enter" || e.key === "Tab") {
            e.preventDefault();
            imgAcAccept();
            return true;
        }
        if (e.key === "Escape") {
            e.preventDefault();
            imgAcClose();
            return true;
        }
        return false;
    }

    imgAutocompleteList.addEventListener("click", (e) => {
        const item = e.target.closest(".link-ac-item");
        if (!item) return;
        imgAcSelectedIdx = parseInt(item.dataset.idx, 10);
        imgAcAccept();
    });

    // ═══════════════════════════════════════════════════════════
    // Toolbar Image-Link File Picker (🖼 button)
    // ═══════════════════════════════════════════════════════════

    function openImagePicker() {
        imagePicker.hidden = false;
        imagePickerSearch.value = "";
        renderImagePicker("");
        requestAnimationFrame(() => imagePickerSearch.focus());
    }

    function closeImagePicker() {
        imagePicker.hidden = true;
    }

    function renderImagePicker(query) {
        const files = getImageFiles();
        const q = query.toLowerCase();
        const filtered = q
            ? files.filter(f => `${f.name}.${f.ext}`.toLowerCase().includes(q) || f.name.toLowerCase().includes(q))
            : files;

        if (filtered.length === 0) {
            imagePickerList.innerHTML = `<div class="link-ac-empty">No matching images</div>`;
            return;
        }
        imagePickerList.innerHTML = filtered.map(f => {
            const thumbUrl = `/api/files/${f.id}/raw`;
            return `<div class="link-picker-item" data-file-name="${escapeHtml(f.name)}.${escapeHtml(f.ext)}">
                <img class="link-picker-thumb" src="${thumbUrl}" alt="" loading="lazy">
                <span class="link-picker-name">${escapeHtml(f.name)}.${escapeHtml(f.ext)}</span>
            </div>`;
        }).join("");
    }

    function insertImageLink(fileName) {
        const start = editor.selectionStart;
        const end   = editor.selectionEnd;
        const selected = editor.value.substring(start, end);
        const before = editor.value.substring(0, start);
        const after  = editor.value.substring(end);

        // If text selected, use it as alt text
        const insert = selected
            ? `![[${fileName}|${selected}]]`
            : `![[${fileName}]]`;

        editor.value = before + insert + after;
        const newPos = start + insert.length;
        editor.setSelectionRange(newPos, newPos);
        editor.focus();
        closeImagePicker();
        onEditorInput();
    }

    btnImagePicker.addEventListener("click", (e) => {
        e.stopPropagation();
        if (imagePicker.hidden) {
            openImagePicker();
        } else {
            closeImagePicker();
        }
    });

    imagePickerSearch.addEventListener("input", () => {
        renderImagePicker(imagePickerSearch.value);
    });

    imagePickerList.addEventListener("click", (e) => {
        const item = e.target.closest(".link-picker-item");
        if (!item) return;
        insertImageLink(item.dataset.fileName);
    });

    document.addEventListener("click", (e) => {
        if (!imagePicker.hidden && !e.target.closest("#image-picker") && !e.target.closest("#btn-image-picker")) {
            closeImagePicker();
        }
    });

    imagePickerSearch.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            closeImagePicker();
            editor.focus();
            return;
        }
        if (e.key === "Enter") {
            const first = imagePickerList.querySelector(".link-picker-item");
            if (first) insertImageLink(first.dataset.fileName);
            return;
        }
        if (e.key === "ArrowDown") {
            e.preventDefault();
            const first = imagePickerList.querySelector(".link-picker-item");
            if (first) first.focus();
        }
    });

    // ═══════════════════════════════════════════════════════════
    // Backlinks (shown in preview when a file is open)
    // ═══════════════════════════════════════════════════════════

    /**
     * Find all files that link to the current file via [[name]] or [text](name).
     * Fetches content in parallel for speed.
     */
    async function findBacklinks(targetName) {
        const targetLower = targetName.toLowerCase();
        const candidates = fileList.filter(f => f.id !== currentFileId && f.ext === "md");
        if (candidates.length === 0) return [];

        const results = await Promise.allSettled(
            candidates.map(f => apiGet(`/files/${f.id}`).then(data => ({ f, content: (data.content || "").toLowerCase() })))
        );

        const backlinks = [];
        for (const r of results) {
            if (r.status !== "fulfilled") continue;
            const { f, content } = r.value;
            if (
                content.includes(`[[${targetLower}`)   ||
                content.includes(`[[${targetLower}|`)   ||
                content.includes(`(${targetLower})`)    ||
                content.includes(`(${targetLower}.md)`)
            ) {
                backlinks.push({ id: f.id, name: f.name });
            }
        }
        return backlinks;
    }

    function renderBacklinks(backlinks) {
        const existing = preview.querySelector(".backlinks-section");
        if (existing) existing.remove();

        if (backlinks.length === 0) return;

        const section = document.createElement("div");
        section.className = "backlinks-section";
        section.innerHTML = `
            <h4 class="backlinks-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 17H7A5 5 0 0 1 7 7h2"/><path d="M15 7h2a5 5 0 1 1 0 10h-2"/><line x1="8" x2="16" y1="12" y2="12"/></svg>
                Linked by ${backlinks.length} page${backlinks.length > 1 ? "s" : ""}
            </h4>
            <ul class="backlinks-list">
                ${backlinks.map(bl => `<li><a href="#" class="md-internal-link backlink-item" data-target="${escapeHtml(bl.name)}">${escapeHtml(bl.name)}</a></li>`).join("")}
            </ul>
        `;
        preview.appendChild(section);
    }

    // ═══════════════════════════════════════════════════════════
    // Internal Link Resolution
    // ═══════════════════════════════════════════════════════════

    /**
     * Resolve an internal link target string to a file in fileList.
     * Handles: "name", "name.md", "Name", case-insensitively.
     * Returns the file object or null.
     */
    function resolveInternalLink(target) {
        if (!target) return null;
        const t = target.trim().toLowerCase();
        const tNoExt = t.replace(/\.md$/, "");
        return fileList.find(f => {
            const name = f.name.toLowerCase();
            const full = `${name}.${f.ext}`;
            return name === tNoExt || full === t || name === t || full === tNoExt;
        }) || null;
    }

    /**
     * After rendering preview, annotate broken internal links
     * with a CSS class so they're visually distinct.
     * Also resolve image wiki-links (![[image]]) to actual <img> src URLs.
     */
    function annotateInternalLinks() {
        preview.querySelectorAll(".md-internal-link").forEach(link => {
            const target = link.dataset.target;
            const resolved = resolveInternalLink(target);
            if (resolved) {
                link.classList.remove("md-link-broken");
                link.title = `Go to: ${resolved.name}.${resolved.ext}`;
            } else {
                link.classList.add("md-link-broken");
                link.title = `Page not found: ${target} — click to create`;
            }
        });

        // Resolve image wiki-links
        preview.querySelectorAll("img.md-image-link").forEach(img => {
            const target = img.dataset.target;
            const resolved = resolveImageLink(target);
            if (resolved) {
                img.src = `/api/files/${resolved.id}/raw`;
                img.classList.remove("md-image-broken");
                img.title = `${resolved.name}.${resolved.ext}`;
            } else {
                img.removeAttribute("src");
                img.classList.add("md-image-broken");
                img.title = `Image not found: ${target}`;
                img.alt = `⚠ ${target} (not found)`;
            }
        });
    }

    /**
     * Create a new markdown page from a broken link click.
     */
    async function createLinkedPage(name) {
        const cleanName = name.replace(/\.md$/, "").trim();
        if (!cleanName) return;

        try {
            const today = new Date().toISOString().slice(0, 10);
            const initialContent = serializeFrontmatter({
                title: cleanName,
                description: "",
                tags: [],
                date: today,
                last_mod: today,
                visibility: "public",
            }, "");

            const file = await apiPost("/files", {
                name: cleanName,
                ext: "md",
                mime: "text/markdown",
                content: initialContent,
            });

            await loadFileList();
            showToast(`Created page: ${cleanName}`, "success");
            openFile(file.id);
        } catch (e) {
            showToast(`Failed to create page: ${e.message}`, "error");
        }
    }

    // Handle internal markdown link clicks in preview
    preview.addEventListener("click", (e) => {
        const link = e.target.closest(".md-internal-link");
        if (!link) return;
        e.preventDefault();

        const target = link.dataset.target;
        if (!target) return;

        const match = resolveInternalLink(target);
        if (match) {
            openFile(match.id);
        } else {
            // Offer to create the missing page
            if (confirm(`Page "${target}" doesn't exist.\n\nCreate it?`)) {
                createLinkedPage(target);
            }
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

        // Update mobile pane toggle icon
        updateMobilePaneToggle(mode);

        if (mode === "preview" || mode === "split") {
            updatePreview();
        }
    }

    /** Update the mobile pane toggle button icon and label */
    function updateMobilePaneToggle(mode) {
        const toggleBtn = $("#btn-mobile-pane-toggle");
        if (!toggleBtn) return;
        if (mode === "preview") {
            toggleBtn.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>`;
            toggleBtn.title = "Switch to editor";
        } else {
            toggleBtn.innerHTML = `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/><circle cx="12" cy="12" r="3"/></svg>`;
            toggleBtn.title = "Switch to preview";
        }
    }

    /** Init mobile pane toggle — switch between editor and preview on narrow screens */
    function initMobilePaneToggle() {
        const toggleBtn = $("#btn-mobile-pane-toggle");
        if (!toggleBtn) return;

        toggleBtn.addEventListener("click", () => {
            const current = editorPanes.dataset.view;
            if (current === "preview") {
                setViewMode("editor");
            } else {
                setViewMode("preview");
            }
        });

        // On mobile, default to editor-only
        const mq = window.matchMedia("(max-width: 768px)");
        function handleMq(e) {
            if (e.matches) {
                if (editorPanes.dataset.view === "split") {
                    setViewMode("editor");
                }
            }
        }
        mq.addEventListener("change", handleMq);
        if (mq.matches) {
            setViewMode("editor");
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Sidebar Toggle
    // ═══════════════════════════════════════════════════════════
    function toggleSidebar() {
        const collapsed = sidebar.dataset.collapsed === "true";
        sidebar.dataset.collapsed = !collapsed ? "true" : "false";

        // On mobile, also toggle overlay
        if (window.matchMedia("(max-width: 768px)").matches) {
            const overlay = $("#sidebar-overlay");
            if (overlay) {
                overlay.classList.toggle("visible", collapsed); // show when opening
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Sidebar Resize (drag handle)
    // ═══════════════════════════════════════════════════════════
    function initSidebarResize() {
        const handle = $("#sidebar-resize-handle");
        if (!handle) return;

        let isDragging = false;

        // Restore saved width
        const saved = localStorage.getItem("synker-sidebar-width");
        if (saved) {
            const w = parseInt(saved, 10);
            if (w >= 150 && w <= 600) {
                sidebar.style.width = w + "px";
                sidebar.style.minWidth = w + "px";
            }
        }

        handle.addEventListener("pointerdown", (e) => {
            // Don't resize if sidebar is collapsed
            if (sidebar.dataset.collapsed === "true") return;

            isDragging = true;
            handle.classList.add("dragging");
            handle.setPointerCapture(e.pointerId);
            document.body.style.cursor = "col-resize";
            document.body.style.userSelect = "none";
            e.preventDefault();
        });

        document.addEventListener("pointermove", (e) => {
            if (!isDragging) return;

            const newWidth = Math.max(150, Math.min(600, e.clientX));
            sidebar.style.width = newWidth + "px";
            sidebar.style.minWidth = newWidth + "px";
        });

        document.addEventListener("pointerup", () => {
            if (!isDragging) return;
            isDragging = false;
            handle.classList.remove("dragging");
            document.body.style.cursor = "";
            document.body.style.userSelect = "";

            // Save to localStorage
            const w = parseInt(sidebar.style.width, 10);
            if (w) {
                localStorage.setItem("synker-sidebar-width", w);
            }
        });
    }

    // ═══════════════════════════════════════════════════════════
    // Responsive Sidebar (mobile overlay + auto-collapse)
    // ═══════════════════════════════════════════════════════════
    function initResponsiveSidebar() {
        const overlay = $("#sidebar-overlay");
        const mobileFab = $("#btn-mobile-sidebar");

        // Overlay click closes sidebar
        if (overlay) {
            overlay.addEventListener("click", () => {
                sidebar.dataset.collapsed = "true";
                overlay.classList.remove("visible");
            });
        }

        // Mobile FAB opens sidebar
        if (mobileFab) {
            mobileFab.addEventListener("click", () => {
                const isCollapsed = sidebar.dataset.collapsed === "true";
                sidebar.dataset.collapsed = isCollapsed ? "false" : "true";
                if (overlay) {
                    overlay.classList.toggle("visible", isCollapsed);
                }
            });
        }

        // Auto-collapse sidebar on narrow viewport
        const mq = window.matchMedia("(max-width: 768px)");
        function handleMq(e) {
            if (e.matches) {
                sidebar.dataset.collapsed = "true";
                if (overlay) overlay.classList.remove("visible");
            }
        }
        mq.addEventListener("change", handleMq);
        // Initial check
        if (mq.matches) {
            sidebar.dataset.collapsed = "true";
        }
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

        if (e.key === "l") {
            e.preventDefault();
            openLinkPicker();
            return;
        }

        // Ctrl+Shift+I — open image picker
        if (e.key === "I" && e.shiftKey) {
            e.preventDefault();
            openImagePicker();
            return;
        }

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
        // Let autocomplete handle Tab/Enter/Arrow keys first
        if (acHandleKeydown(e)) return;
        if (imgAcHandleKeydown(e)) return;
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

        // Image-link autocomplete detection (must run before wiki-link)
        imgAcHandleInput();

        // Wiki-link autocomplete detection
        acHandleInput();

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
        initSidebarResize();
        initResponsiveSidebar();
        initMobilePaneToggle();

        // Editor events
        editor.addEventListener("input", onEditorInput);
        editor.addEventListener("click", () => {
            updateCursorStatus();
            updateLineNumbers();
            // Close autocomplete if user clicks elsewhere
            if (acActive) acClose();
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

            const displayName = fileDisplayName(file);
            const hasTitle = !!file.title;

            const icon = isImg
                ? `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/></svg>`
                : `<svg class="file-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>`;

            btn.innerHTML = `
                ${icon}
                <span class="file-name">${escapeHtml(displayName)}</span>
                ${hasTitle ? `<span class="file-ext file-ext-subtle" title="${escapeHtml(file.name)}.${escapeHtml(file.ext)}">.${escapeHtml(file.ext)}</span>` : `<span class="file-ext">.${escapeHtml(file.ext)}</span>`}
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
