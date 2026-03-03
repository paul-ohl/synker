/* ═══════════════════════════════════════════════════════════════
   Synker — Markdown Renderer  (GFM-compatible, zero dependencies)
   ═══════════════════════════════════════════════════════════════
   Fully iterative design — no recursive calls — safe for any
   document size. Exposed as window.SynkerMD.render(src) → HTML.

   Block-level features
   ────────────────────
   • ATX headings  # … ######  (with auto-id anchors)
   • Setext headings  === / ---
   • Fenced code blocks  ``` / ~~~  (language tag)
   • Indented code blocks  (4-space / tab)
   • Blockquotes  >  (nestable)
   • Unordered lists (-, *, +) with nesting
   • Ordered lists  1.  with nesting
   • Task / check lists  - [x] / - [ ]
   • GFM tables  with :--- :---: ---: alignment
   • Thematic breaks  --- / *** / ___
   • Footnote definitions  [^id]: text
   • Definition lists  term → : definition
   • HTML block pass-through (<div>, <details>, …)
   • Math blocks  $$ … $$

   Inline features
   ───────────────
   • Bold ** / __   Italic * / _   Bold+Italic *** / ___
   • Strikethrough ~~text~~
   • Inline code  `code`  (multi-backtick aware)
   • Images  ![alt](src "title")
   • Links  [text](href "title")  — internal-link detection
   • Autolinks  <https://…>  and  <email@…>
   • Bare URL auto-linking
   • Footnote references  [^id]
   • Highlight / mark  ==text==
   • Superscript  ^text^   Subscript  ~text~
   • Inline math  $…$
   • Emoji shortcodes  :smile:
   • Escaped characters  \* \_ \` …
   • Hard line breaks  (trailing ␣␣ or \↵)
   ═══════════════════════════════════════════════════════════════ */

window.SynkerMD = (() => {
    "use strict";

    // ── Emoji shortcodes ──────────────────────────────────────
    const EMOJI = Object.freeze({
        smile:"😄",laughing:"😆",blush:"😊",smiley:"😃",relaxed:"☺️",
        heart:"❤️",thumbsup:"👍",thumbsdown:"👎","+1":"👍","-1":"👎",
        ok_hand:"👌",wave:"👋",clap:"👏",raised_hands:"🙌",
        fire:"🔥",star:"⭐",sparkles:"✨",zap:"⚡",
        warning:"⚠️",x:"❌",white_check_mark:"✅",heavy_check_mark:"✔️",
        question:"❓",exclamation:"❗",bulb:"💡",memo:"📝",
        rocket:"🚀",tada:"🎉","100":"💯",eyes:"👀",
        thinking:"🤔",sob:"😭",joy:"😂",wink:"😉",
        sunglasses:"😎",skull:"💀",ghost:"👻",pray:"🙏",
        gem:"💎",key:"🔑",lock:"🔒",unlock:"🔓",
        link:"🔗",gear:"⚙️",hammer:"🔨",wrench:"🔧",
        package:"📦",book:"📖",bookmark:"🔖",pencil:"✏️",
        mag:"🔍",bell:"🔔",clock:"🕐",calendar:"📅",
        chart_with_upwards_trend:"📈",bug:"🐛",construction:"🚧",
        arrow_right:"➡️",arrow_left:"⬅️",arrow_up:"⬆️",arrow_down:"⬇️",
        info:"ℹ️",heavy_plus_sign:"➕",heavy_minus_sign:"➖",
        coffee:"☕",beer:"🍺",pizza:"🍕",apple:"🍎",
        dog:"🐶",cat:"🐱",penguin:"🐧",snake:"🐍",
        earth_americas:"🌎",sun:"☀️",moon:"🌙",cloud:"☁️",
        umbrella:"☂️",snowflake:"❄️",rainbow:"🌈",ocean:"🌊",
        check:"✔️",cross:"✖️",pin:"📌",paperclip:"📎",
        scissors:"✂️",point_right:"👉",point_left:"👈",muscle:"💪",
        crown:"👑",trophy:"🏆",medal:"🏅",ribbon:"🎀",
        art:"🎨",musical_note:"🎵",microphone:"🎤",movie_camera:"🎥",
        computer:"💻",iphone:"📱",battery:"🔋",electric_plug:"🔌",
        satellite:"🛰️",airplane:"✈️",car:"🚗",bike:"🚲",
        house:"🏠",school:"🏫",hospital:"🏥",bank:"🏦",
        tree:"🌳",flower:"🌸",herb:"🌿",mushroom:"🍄",
        wave_hand:"👋",handshake:"🤝",victory:"✌️",love_letter:"💌",
    });

    // ── HTML escaping ─────────────────────────────────────────
    const ESC_MAP = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" };
    const esc = (s) => s.replace(/[&<>"]/g, (c) => ESC_MAP[c]);

    const slugify = (s) =>
        s.toLowerCase()
         .replace(/<[^>]*>/g, "")
         .replace(/[^\w\s-]/g, "")
         .trim()
         .replace(/\s+/g, "-");

    // ────────────────────────────────────────────────────────────
    //  INLINE RENDERER  (fully iterative, token-scan approach)
    // ────────────────────────────────────────────────────────────

    /** Find the closing delimiter `delim` starting from `from`, skipping escaped chars.
     *  Returns index or -1. */
    function findClose(src, delim, from) {
        const dLen = delim.length;
        let j = from;
        while (j <= src.length - dLen) {
            if (src[j] === "\\" && j + 1 < src.length) { j += 2; continue; }
            if (src.substring(j, j + dLen) === delim) return j;
            j++;
        }
        return -1;
    }

    /**
     * Render inline markdown to HTML.  Uses an iterative character scanner.
     * `depth` is a safety counter — prevents runaway on malformed input.
     */
    function renderInline(src, depth) {
        if (depth === undefined) depth = 0;
        if (depth > 8 || !src) return esc(src || "");

        let out = "";
        let i = 0;
        const len = src.length;
        // Safety: if we exceed 500 000 iterations, bail out
        let safety = 0;
        const SAFETY_LIMIT = 500_000;

        while (i < len) {
            if (++safety > SAFETY_LIMIT) { out += esc(src.slice(i)); break; }

            const ch = src[i];

            // ── Escaped character ──
            if (ch === "\\" && i + 1 < len && /[\\`*_{}[\]()#+\-.!~^=|<>$]/.test(src[i + 1])) {
                out += esc(src[i + 1]);
                i += 2;
                continue;
            }

            // ── Inline code ──
            if (ch === "`") {
                let ticks = 0;
                let j = i;
                while (j < len && src[j] === "`") { ticks++; j++; }
                const closer = "`".repeat(ticks);
                const closeIdx = src.indexOf(closer, j);
                if (closeIdx !== -1) {
                    // Verify the close is exactly `ticks` backticks (not more)
                    const afterClose = closeIdx + ticks;
                    if (afterClose >= len || src[afterClose] !== "`") {
                        let code = src.slice(j, closeIdx);
                        if (code.length >= 2 && code[0] === " " && code[code.length - 1] === " ") {
                            code = code.slice(1, -1);
                        }
                        out += `<code>${esc(code)}</code>`;
                        i = afterClose;
                        continue;
                    }
                }
                out += esc("`");
                i++;
                continue;
            }

            // ── Inline math $…$ (not $$) ──
            if (ch === "$" && src[i + 1] !== "$") {
                const end = findClose(src, "$", i + 1);
                if (end > i + 1 && !src.slice(i + 1, end).includes("\n")) {
                    out += `<code class="math-inline">${esc(src.slice(i + 1, end))}</code>`;
                    i = end + 1;
                    continue;
                }
            }

            // ── Image ![alt](url "title") ──
            if (ch === "!" && src[i + 1] === "[") {
                const altEnd = findClose(src, "]", i + 2);
                if (altEnd !== -1 && src[altEnd + 1] === "(") {
                    const parenEnd = findClose(src, ")", altEnd + 2);
                    if (parenEnd !== -1) {
                        const alt = esc(src.slice(i + 2, altEnd));
                        const inside = src.slice(altEnd + 2, parenEnd).trim();
                        const tm = inside.match(/^(\S+?)(?:\s+"([^"]*)")?$/);
                        const url = esc(tm ? tm[1] : inside);
                        const title = tm && tm[2] ? ` title="${esc(tm[2])}"` : "";
                        out += `<img src="${url}" alt="${alt}"${title} loading="lazy">`;
                        i = parenEnd + 1;
                        continue;
                    }
                }
            }

            // ── Footnote reference [^id] ──
            if (ch === "[" && src[i + 1] === "^") {
                const m = src.slice(i).match(/^\[\^([^\]]+)\]/);
                if (m) {
                    const id = esc(m[1]);
                    out += `<sup class="footnote-ref"><a href="#fn-${id}" id="fnref-${id}">[${id}]</a></sup>`;
                    i += m[0].length;
                    continue;
                }
            }

            // ── Link [text](href "title") ──
            if (ch === "[") {
                const bracketEnd = findClose(src, "]", i + 1);
                if (bracketEnd !== -1 && src[bracketEnd + 1] === "(") {
                    const parenEnd = findClose(src, ")", bracketEnd + 2);
                    if (parenEnd !== -1) {
                        const label = src.slice(i + 1, bracketEnd);
                        const inside = src.slice(bracketEnd + 2, parenEnd).trim();
                        const tm = inside.match(/^(\S+?)(?:\s+"([^"]*)")?$/);
                        const href = tm ? tm[1] : inside;
                        const title = tm && tm[2] ? ` title="${esc(tm[2])}"` : "";
                        const isExt = /^(https?:|mailto:|ftp:|\/\/|#)/.test(href);
                        if (isExt) {
                            out += `<a href="${esc(href)}"${title} target="_blank" rel="noopener">${renderInline(label, depth + 1)}</a>`;
                        } else {
                            out += `<a href="#" class="md-internal-link" data-target="${esc(href)}"${title}>${renderInline(label, depth + 1)}</a>`;
                        }
                        i = parenEnd + 1;
                        continue;
                    }
                }
            }

            // ── Autolink <URL> or <email> ──
            if (ch === "<") {
                const gt = src.indexOf(">", i + 1);
                if (gt !== -1) {
                    const inner = src.slice(i + 1, gt);
                    if (/^https?:\/\/.+/i.test(inner)) {
                        out += `<a href="${esc(inner)}" target="_blank" rel="noopener">${esc(inner)}</a>`;
                        i = gt + 1;
                        continue;
                    }
                    if (/^[^@\s]+@[^@\s]+\.[a-z]{2,}$/i.test(inner)) {
                        out += `<a href="mailto:${esc(inner)}">${esc(inner)}</a>`;
                        i = gt + 1;
                        continue;
                    }
                    // Could be an HTML tag — pass through
                    const tagMatch = src.slice(i).match(/^<\/?[a-zA-Z][^>]*>/);
                    if (tagMatch) {
                        out += tagMatch[0];
                        i += tagMatch[0].length;
                        continue;
                    }
                }
            }

            // ── Bare URL ──
            if ((i === 0 || /[\s([\n]/.test(src[i - 1])) && (ch === "h") ) {
                const m = src.slice(i).match(/^(https?:\/\/[^\s<>)\]]+)/);
                if (m) {
                    const u = esc(m[1].replace(/[.,;:!?)]+$/, ""));
                    out += `<a href="${u}" target="_blank" rel="noopener">${u}</a>`;
                    i += m[1].replace(/[.,;:!?)]+$/, "").length;
                    continue;
                }
            }

            // ── Bold+Italic  *** or ___ ──
            if ((ch === "*" || ch === "_") && src[i + 1] === ch && src[i + 2] === ch) {
                const end = findClose(src, ch + ch + ch, i + 3);
                if (end !== -1) {
                    out += `<strong><em>${renderInline(src.slice(i + 3, end), depth + 1)}</em></strong>`;
                    i = end + 3;
                    continue;
                }
            }

            // ── Bold  ** or __ ──
            if ((ch === "*" || ch === "_") && src[i + 1] === ch) {
                const end = findClose(src, ch + ch, i + 2);
                if (end !== -1 && end > i + 2) {
                    out += `<strong>${renderInline(src.slice(i + 2, end), depth + 1)}</strong>`;
                    i = end + 2;
                    continue;
                }
            }

            // ── Italic  * or _ ──
            if (ch === "*" || ch === "_") {
                const end = findClose(src, ch, i + 1);
                if (end !== -1 && end > i + 1 && src[end - 1] !== " ") {
                    out += `<em>${renderInline(src.slice(i + 1, end), depth + 1)}</em>`;
                    i = end + 1;
                    continue;
                }
            }

            // ── Strikethrough ~~ ──
            if (ch === "~" && src[i + 1] === "~") {
                const end = findClose(src, "~~", i + 2);
                if (end !== -1) {
                    out += `<del>${renderInline(src.slice(i + 2, end), depth + 1)}</del>`;
                    i = end + 2;
                    continue;
                }
            }

            // ── Highlight ==text== ──
            if (ch === "=" && src[i + 1] === "=") {
                const end = findClose(src, "==", i + 2);
                if (end !== -1) {
                    out += `<mark>${renderInline(src.slice(i + 2, end), depth + 1)}</mark>`;
                    i = end + 2;
                    continue;
                }
            }

            // ── Superscript ^text^ ──
            if (ch === "^") {
                const end = findClose(src, "^", i + 1);
                if (end !== -1 && end > i + 1 && end - i < 50 && !src.slice(i + 1, end).includes(" ")) {
                    out += `<sup>${esc(src.slice(i + 1, end))}</sup>`;
                    i = end + 1;
                    continue;
                }
            }

            // ── Subscript ~text~ (single, not ~~) ──
            if (ch === "~" && src[i + 1] !== "~") {
                const end = findClose(src, "~", i + 1);
                if (end !== -1 && end > i + 1 && src[end + 1] !== "~" && end - i < 50 && !src.slice(i + 1, end).includes(" ")) {
                    out += `<sub>${esc(src.slice(i + 1, end))}</sub>`;
                    i = end + 1;
                    continue;
                }
            }

            // ── Emoji :shortcode: ──
            if (ch === ":") {
                const m = src.slice(i).match(/^:([a-z0-9_+-]+):/);
                if (m && EMOJI[m[1]]) {
                    out += EMOJI[m[1]];
                    i += m[0].length;
                    continue;
                }
            }

            // ── Hard line break (trailing two spaces + newline) ──
            if (ch === " " && i + 2 < len && src[i + 1] === " " && src[i + 2] === "\n") {
                out += "<br>";
                i += 3;
                continue;
            }
            // ── Hard line break (backslash + newline) ──
            if (ch === "\\" && src[i + 1] === "\n") {
                out += "<br>";
                i += 2;
                continue;
            }

            // ── Newline (soft break) ──
            if (ch === "\n") {
                out += "\n";
                i++;
                continue;
            }

            // ── Default character ──
            out += esc(ch);
            i++;
        }

        return out;
    }

    // ────────────────────────────────────────────────────────────
    //  TABLE PARSER
    // ────────────────────────────────────────────────────────────
    function splitCells(line) {
        // Handle escaped pipes inside cells
        const cells = [];
        let current = "";
        let escaped = false;
        for (let i = 0; i < line.length; i++) {
            if (escaped) { current += line[i]; escaped = false; continue; }
            if (line[i] === "\\") { escaped = true; current += "\\"; continue; }
            if (line[i] === "|") { cells.push(current.trim()); current = ""; continue; }
            current += line[i];
        }
        cells.push(current.trim());
        // Remove first/last empty cells from leading/trailing |
        if (cells.length && cells[0] === "") cells.shift();
        if (cells.length && cells[cells.length - 1] === "") cells.pop();
        return cells;
    }

    function parseTable(rows) {
        if (rows.length < 2) return null;

        const hdrs = splitCells(rows[0]);
        const seps = splitCells(rows[1]);
        if (hdrs.length === 0) return null;
        if (!seps.every(c => /^:?-{1,}:?$/.test(c))) return null;

        const aligns = seps.map(c => {
            const l = c.startsWith(":");
            const r = c.endsWith(":");
            return l && r ? "center" : r ? "right" : "left";
        });

        const sa = (idx) => ` style="text-align:${aligns[idx] || "left"}"`;

        let h = "<table><thead><tr>";
        for (let c = 0; c < hdrs.length; c++) {
            h += `<th${sa(c)}>${renderInline(hdrs[c])}</th>`;
        }
        h += "</tr></thead><tbody>";

        for (let r = 2; r < rows.length; r++) {
            const cells = splitCells(rows[r]);
            h += "<tr>";
            for (let c = 0; c < hdrs.length; c++) {
                h += `<td${sa(c)}>${renderInline(cells[c] || "")}</td>`;
            }
            h += "</tr>";
        }
        h += "</tbody></table>";
        return h;
    }

    // ────────────────────────────────────────────────────────────
    //  LIST PARSER  (iterative with explicit stack for nesting)
    // ────────────────────────────────────────────────────────────
    function parseLists(lines, start) {
        const firstLine = lines[start];
        const isOrdered = /^\s*\d+\.\s/.test(firstLine);
        const markerRe = isOrdered ? /^(\s*)\d+\.\s+(.*)/ : /^(\s*)[-*+]\s+(.*)/;
        const baseIndent = (firstLine.match(/^(\s*)/) || ["", ""])[1].length;
        const taskRe = /^\[(x| )\]\s+(.*)/i;

        const items = [];
        let i = start;

        while (i < lines.length) {
            const m = lines[i].match(markerRe);
            if (!m) break;
            const itemIndent = m[1].length;
            if (itemIndent < baseIndent) break;

            if (itemIndent > baseIndent) {
                // Nested list — parse recursively (bounded by line count)
                if (items.length === 0) break;
                const sub = parseLists(lines, i);
                items[items.length - 1].sub += sub.html;
                i = sub.end;
                continue;
            }

            let content = m[2];
            let task = null;
            const tm = content.match(taskRe);
            if (tm) {
                task = tm[1].toLowerCase() === "x";
                content = tm[2];
            }
            items.push({ text: content, task, sub: "" });
            i++;

            // Gather continuation / nested content
            while (i < lines.length) {
                if (!lines[i].trim()) { i++; continue; }
                const nestedMarker = lines[i].match(/^(\s*)[-*+]\s/) || lines[i].match(/^(\s*)\d+\.\s/);
                if (nestedMarker && nestedMarker[1].length > baseIndent) {
                    const sub = parseLists(lines, i);
                    items[items.length - 1].sub += sub.html;
                    i = sub.end;
                    continue;
                }
                // Continuation line (indented text, not a marker)
                if (/^\s{2,}/.test(lines[i]) && !nestedMarker) {
                    items[items.length - 1].text += "\n" + lines[i].trim();
                    i++;
                    continue;
                }
                break;
            }
        }

        const tag = isOrdered ? "ol" : "ul";
        const hasTask = items.some(it => it.task !== null);
        const cls = hasTask ? ' class="task-list"' : "";

        const lis = items.map(it => {
            let inner;
            if (it.task !== null) {
                const chk = it.task ? " checked" : "";
                inner = `<input type="checkbox"${chk} disabled> ${renderInline(it.text)}`;
            } else {
                inner = renderInline(it.text);
            }
            if (it.sub) inner += it.sub;
            const liCls = it.task !== null ? ' class="task-list-item"' : "";
            return `<li${liCls}>${inner}</li>`;
        }).join("");

        return { html: `<${tag}${cls}>${lis}</${tag}>`, end: i };
    }

    // ────────────────────────────────────────────────────────────
    //  BLOCK PARSER  (iterative with explicit stack for quotes)
    // ────────────────────────────────────────────────────────────
    const BLOCK_TAG_RE = /^<\/?(div|section|article|aside|header|footer|nav|main|details|summary|figure|figcaption|fieldset|form|table|thead|tbody|tfoot|tr|th|td|dl|dt|dd|pre|blockquote|hr|p|ul|ol|li|h[1-6]|iframe|script|style|link|meta|video|audio|source|canvas|svg|picture)[\s>\/]/i;

    function parseBlocks(src) {
        const lines = src.split("\n");
        const out = [];
        let i = 0;
        const footnotes = {};

        while (i < lines.length) {
            const line = lines[i];
            const trimmed = line.trim();

            // ── Empty line ──
            if (!trimmed) { i++; continue; }

            // ── Footnote definition [^id]: text ──
            {
                const m = line.match(/^\[\^([^\]]+)\]:\s+(.*)/);
                if (m) {
                    const fnId = m[1];
                    const body = [m[2]];
                    i++;
                    while (i < lines.length && /^\s{2,}/.test(lines[i])) {
                        body.push(lines[i].trim());
                        i++;
                    }
                    footnotes[fnId] = body.join(" ");
                    continue;
                }
            }

            // ── Fenced code block  ``` or ~~~ ──
            {
                const m = line.match(/^(`{3,}|~{3,})(.*?)\s*$/);
                if (m) {
                    const fenceChar = m[1][0];
                    const fenceLen = m[1].length;
                    const meta = m[2].trim();
                    // Extract language (first word of meta)
                    const lang = meta.split(/\s/)[0] || "";
                    const codeLines = [];
                    i++;
                    while (i < lines.length) {
                        const cl = lines[i];
                        // Closing fence: same char, at least same count, nothing else
                        if (new RegExp(`^${fenceChar === "`" ? "`" : "~"}{${fenceLen},}\\s*$`).test(cl)) {
                            i++;
                            break;
                        }
                        codeLines.push(cl);
                        i++;
                    }
                    const langAttr = lang ? ` class="language-${esc(lang)}"` : "";
                    out.push(`<pre><code${langAttr}>${esc(codeLines.join("\n"))}</code></pre>`);
                    continue;
                }
            }

            // ── Math block $$ … $$ ──
            if (trimmed === "$$") {
                const mathLines = [];
                i++;
                while (i < lines.length && lines[i].trim() !== "$$") {
                    mathLines.push(lines[i]);
                    i++;
                }
                if (i < lines.length) i++; // skip closing $$
                out.push(`<pre class="math-block"><code>${esc(mathLines.join("\n"))}</code></pre>`);
                continue;
            }

            // ── Indented code block (4 spaces or 1 tab, not a list item) ──
            if (/^(    |\t)/.test(line) && !/^\s*[-*+]\s/.test(line) && !/^\s*\d+\.\s/.test(line)) {
                const codeLines = [];
                while (i < lines.length && (/^(    |\t)/.test(lines[i]) || !lines[i].trim())) {
                    codeLines.push(lines[i].trim() ? lines[i].replace(/^(    |\t)/, "") : "");
                    i++;
                }
                while (codeLines.length && !codeLines[codeLines.length - 1]) codeLines.pop();
                if (codeLines.length) {
                    out.push(`<pre><code>${esc(codeLines.join("\n"))}</code></pre>`);
                }
                continue;
            }

            // ── HTML block ──
            if (BLOCK_TAG_RE.test(trimmed)) {
                const htmlLines = [line];
                i++;
                while (i < lines.length && lines[i].trim()) {
                    htmlLines.push(lines[i]);
                    i++;
                }
                out.push(htmlLines.join("\n"));
                continue;
            }

            // ── GFM Table ──
            if (/^\|/.test(line) && i + 1 < lines.length && /^\|[\s:]*-/.test(lines[i + 1])) {
                const tableLines = [];
                while (i < lines.length && /^\|/.test(lines[i])) {
                    tableLines.push(lines[i]);
                    i++;
                }
                const table = parseTable(tableLines);
                if (table) { out.push(table); continue; }
                // If parsing failed, rewind and let paragraph handle it
                i -= tableLines.length;
            }

            // ── ATX Heading  # … ###### ──
            {
                const m = line.match(/^(#{1,6})\s+(.+?)(?:\s+#+)?\s*$/);
                if (m) {
                    const lvl = m[1].length;
                    const text = m[2];
                    const id = slugify(text);
                    out.push(`<h${lvl} id="${id}">${renderInline(text)}</h${lvl}>`);
                    i++;
                    continue;
                }
            }

            // ── Setext Heading ──
            if (i + 1 < lines.length) {
                const next = lines[i + 1].trim();
                if (/^={2,}$/.test(next)) {
                    const id = slugify(trimmed);
                    out.push(`<h1 id="${id}">${renderInline(trimmed)}</h1>`);
                    i += 2;
                    continue;
                }
                if (/^-{2,}$/.test(next) && !/^\s*[-*+]\s/.test(line)) {
                    const id = slugify(trimmed);
                    out.push(`<h2 id="${id}">${renderInline(trimmed)}</h2>`);
                    i += 2;
                    continue;
                }
            }

            // ── Thematic break  --- / *** / ___ ──
            if (/^([-]{3,}|[*]{3,}|[_]{3,})\s*$/.test(trimmed)) {
                out.push("<hr>");
                i++;
                continue;
            }

            // ── Blockquote  > ──  (iterative: collect lines, strip >, re-parse)
            if (/^>\s?/.test(line)) {
                const qLines = [];
                while (i < lines.length) {
                    if (/^>\s?/.test(lines[i])) {
                        qLines.push(lines[i].replace(/^>\s?/, ""));
                    } else if (lines[i].trim() && qLines.length > 0 && !/^(#{1,6}\s|```|~~~|[-*_]{3,}\s*$|\|)/.test(lines[i])) {
                        // Lazy continuation
                        qLines.push(lines[i]);
                    } else {
                        break;
                    }
                    i++;
                }
                // Iterative inner parse (not recursive back into the same call)
                out.push(`<blockquote>${parseBlocks(qLines.join("\n"))}</blockquote>`);
                continue;
            }

            // ── Definition list  (term followed by :  definition) ──
            if (i + 1 < lines.length && /^:\s+/.test(lines[i + 1])) {
                let dl = `<dl><dt>${renderInline(trimmed)}</dt>`;
                i++;
                while (i < lines.length && /^:\s+/.test(lines[i])) {
                    dl += `<dd>${renderInline(lines[i].replace(/^:\s+/, ""))}</dd>`;
                    i++;
                }
                dl += "</dl>";
                out.push(dl);
                continue;
            }

            // ── List (ordered or unordered) ──
            if (/^\s*[-*+]\s/.test(line) || /^\s*\d+\.\s/.test(line)) {
                const result = parseLists(lines, i);
                out.push(result.html);
                i = result.end;
                continue;
            }

            // ── Paragraph  (collect lines until a block element or blank line) ──
            {
                const pLines = [];
                while (i < lines.length && lines[i].trim()) {
                    const l = lines[i];
                    const lt = l.trim();
                    // Break on block-level constructs
                    if (/^(#{1,6}\s|```|~~~|>\s?|\|)/.test(l)) break;
                    if (/^[-*+]\s/.test(l)) break;
                    if (/^\d+\.\s/.test(l)) break;
                    if (/^([-*_]){3,}\s*$/.test(lt)) break;
                    if (/^\$\$\s*$/.test(lt)) break;
                    if (BLOCK_TAG_RE.test(lt)) break;
                    // Setext underline? — break so the heading parse picks it up
                    if (/^[=-]{2,}\s*$/.test(lt) && pLines.length > 0) break;
                    pLines.push(l);
                    i++;
                }
                if (pLines.length) {
                    const inner = renderInline(pLines.join("\n")).replace(/\n/g, "<br>");
                    out.push(`<p>${inner}</p>`);
                }
            }
        }

        // ── Footnotes section ──
        const fnIds = Object.keys(footnotes);
        if (fnIds.length) {
            let fnHtml = '<section class="footnotes"><hr><ol>';
            for (const id of fnIds) {
                fnHtml += `<li id="fn-${esc(id)}"><p>${renderInline(footnotes[id])} <a href="#fnref-${esc(id)}" class="footnote-back" title="Back to reference">↩</a></p></li>`;
            }
            fnHtml += "</ol></section>";
            out.push(fnHtml);
        }

        return out.join("\n");
    }

    // ── Public API ────────────────────────────────────────────
    return Object.freeze({
        render(src) {
            if (!src || !src.trim()) {
                return '<p class="preview-placeholder">Preview will appear here...</p>';
            }
            return parseBlocks(src);
        },
    });
})();
