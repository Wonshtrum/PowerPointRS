// fs.js — self-contained "binary forge" file picker.
//
// Usage (from any HTML page, no other markup/CSS required):
//
//   import { fs_fetch } from "https://raw.githack.com/<user>/<forge-repo>/<branch>/fs.js";
//
//   const wasmBuf = await fs_fetch("myuser", "forge", "main", [
//     "../target/wasm32-unknown-unknown/release/bin1.wasm",
//   ]);
//   const { instance } = await WebAssembly.instantiate(wasmBuf);
//
// fs_fetch() pops up a small overlay with three tabs:
//   - GitHub: browses the tree of <repo>@<branch> on GitHub and fetches the
//     selected blob's raw content.
//   - Local:  lists the paths passed in `localPaths` and fetches them
//     relative to the current page (handy while iterating on a fresh local
//     build, before it's pushed to the forge).
//   - Drop:   lets you drag a file straight from your filesystem, read via
//     File.arrayBuffer() (no network at all).
//
// Whichever one you pick, the returned promise resolves to an ArrayBuffer.
// Closing the popup without picking rejects the promise.
//
// Persistence: GitHub / Local selections are remembered in localStorage, so
// reloading the page auto-resolves the same artifact without showing the
// popup. A small floating "change artifact" button appears once something
// is loaded — click it to reopen the picker and switch (this reloads the
// page so the new artifact takes effect cleanly). Drop selections are never
// persisted (a File object can't be serialized), so they only apply to the
// page load you dropped them on.
//
// Options (5th argument, all optional):
//   {
//     key: "custom-storage-key",
//     showChangeButton: true,
//     rootPath: "wasm/feature-x",   // only browse this subtree of the repo
//     extensions: [".wasm"],        // only show/accept these file types
//   }
// `key` defaults to `${user}/${repo}@${branch}[:${rootPath}]` — set it
// explicitly if you call fs_fetch() more than once on the same page so
// their remembered selections and change-buttons don't collide.
//
// `rootPath`, if given, is resolved against the repo tree and used as the
// GitHub tab's root instead of the repo root — handy for keeping several
// unrelated artifact sets in one forge repo. fs_fetch() rejects if the path
// doesn't exist in the tree.
//
// `extensions`, if given, filters the GitHub tab to only files ending in
// one of those extensions (directories with no matching files are hidden),
// and is also set as the `accept` attribute on the Drop tab's file input.
// Entries can be given with or without the leading dot (".wasm" or "wasm").

const ICON_BASE = "https://raw.githack.com/PKief/vscode-material-icon-theme/master/icons/";
const ICONS_MODULE_URL = "https://raw.githack.com/minhazmiraz/file-extension-icon-JS/master/src/static/materialFileIcons.js";
const STORAGE_PREFIX = "ffx:last:";

// ---------------------------------------------------------------------------
// localStorage helpers
// ---------------------------------------------------------------------------

function loadLastSelection(key) {
    try {
        const raw = localStorage.getItem(STORAGE_PREFIX + key);
        return raw ? JSON.parse(raw) : null;
    } catch (err) {
        return null;
    }
}

function saveLastSelection(key, selection) {
    try {
        localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(selection));
    } catch (err) {
        console.warn("fs_fetch: could not persist selection", err);
    }
}

function clearLastSelection(key) {
    try {
        localStorage.removeItem(STORAGE_PREFIX + key);
    } catch (err) {
        /* ignore */
    }
}

function defaultKey(user, repo, branch, rootPath) {
    return rootPath ? `${user}/${repo}@${branch}:${rootPath}` : `${user}/${repo}@${branch}`;
}

function normalizeExtensions(extensions) {
    if (!extensions || extensions.length === 0) return null;
    return extensions.map((e) => (e.startsWith(".") ? e : "." + e).toLowerCase());
}

// ---------------------------------------------------------------------------
// Icon loading (cached at module scope, shared across fs_fetch() calls)
// ---------------------------------------------------------------------------

let iconDefsPromise = null;
function loadIconDefs() {
    if (!iconDefsPromise) {
        iconDefsPromise = import(ICONS_MODULE_URL).then((m) => m.materialFileIcons);
    }
    return iconDefsPromise;
}

let iconMapsPromise = null;
function loadIconMaps() {
    if (!iconMapsPromise) {
        iconMapsPromise = loadIconDefs().then((defs) => {
            const ext2icon = {};
            const name2icon = {};
            for (const icon of defs.icons) {
                (icon.fileExtensions || []).forEach((ext) => {
                    ext2icon["." + ext.toLowerCase()] = icon.name;
                });
                (icon.fileNames || []).forEach((name) => {
                    name2icon[name.toLowerCase()] = icon.name;
                });
            }
            return { ext2icon, name2icon };
        });
    }
    return iconMapsPromise;
}

function guessIcon(filename, ext2icon, name2icon) {
    const lower = filename.toLowerCase();
    if (lower in name2icon) return name2icon[lower];
    let best = null;
    let bestLen = 0;
    for (const [ext, name] of Object.entries(ext2icon)) {
        if (lower.endsWith(ext) && ext.length > bestLen) {
            best = name;
            bestLen = ext.length;
        }
    }
    return best;
}

const svgCache = new Map(); // icon name (or null) -> Promise<dataURI|null>

function loadSvgIcon(iconName) {
    if (svgCache.has(iconName)) return svgCache.get(iconName);
    const url = ICON_BASE + (iconName ?? "document") + ".svg";
    const promise = fetch(url)
        .then((r) => (r.ok ? r.text() : Promise.reject(new Error(`icon fetch failed: ${url}`))))
        .then((text) => {
            const doc = new DOMParser().parseFromString(text, "image/svg+xml");
            const svg = doc.querySelector("svg");
            const serialized = new XMLSerializer().serializeToString(svg);
            return "data:image/svg+xml;utf8," + encodeURIComponent(serialized);
        })
        .catch((err) => {
            console.warn(err);
            return null;
        });
    svgCache.set(iconName, promise);
    return promise;
}

// ---------------------------------------------------------------------------
// GitHub tree fetching
// ---------------------------------------------------------------------------

function findSubtree(root, rootPath) {
    const parts = rootPath.split("/").filter(Boolean);
    let node = root;
    for (const part of parts) {
        if (node.type !== "tree") return null;
        const child = node.files.find((f) => f.name === part);
        if (!child) return null;
        node = child;
    }
    return node.type === "tree" ? node : null;
}

function pruneByExtensions(node, exts) {
    if (node.type === "blob") {
        return exts.some((ext) => node.name.toLowerCase().endsWith(ext)) ? node : null;
    }
    const files = node.files.map((f) => pruneByExtensions(f, exts)).filter(Boolean);
    if (files.length === 0) return null;
    return { ...node, files };
}

async function fetchForgeTree(user, repo, branch, { rootPath, extensions } = {}) {
    const res = await fetch(
        `https://api.github.com/repos/${user}/${repo}/git/trees/${encodeURIComponent(branch)}?recursive=1`
    );
    if (!res.ok) {
        throw new Error(`GitHub API error ${res.status}: ${res.statusText}`);
    }
    const json = await res.json();
    if (json.truncated) {
        console.warn("fs_fetch: forge tree was truncated by the GitHub API; some files may be missing");
    }

    const { ext2icon, name2icon } = await loadIconMaps();

    const root = { name: `${user}/${repo}@${branch}`, type: "tree", path: "", files: [] };
    for (const entry of json.tree || []) {
        if (entry.type !== "blob" && entry.type !== "tree") continue;
        const parts = entry.path.split("/");
        let node = root;
        let pathSoFar = "";
        parts.forEach((part, i) => {
            pathSoFar = pathSoFar ? `${pathSoFar}/${part}` : part;
            const isLeaf = i === parts.length - 1;
            let child = node.files.find((f) => f.name === part);
            if (!child) {
                child = {
                    name: part,
                    path: pathSoFar,
                    type: isLeaf ? (entry.type === "tree" ? "tree" : "blob") : "tree",
                    icon: null,
                    files: [],
                };
                node.files.push(child);
            }
            node = child;
        });
        if (node.type === "blob") {
            node.icon = guessIcon(node.name, ext2icon, name2icon);
        }
    }

    (function sortNode(n) {
        if (n.type !== "tree") return;
        n.files.sort((a, b) => {
            if (a.type !== b.type) return a.type === "tree" ? -1 : 1;
            return a.name.localeCompare(b.name);
        });
        n.files.forEach(sortNode);
    })(root);

    let result = root;
    if (rootPath) {
        const subtree = findSubtree(root, rootPath);
        if (!subtree) {
            throw new Error(`Root path "${rootPath}" does not exist in ${user}/${repo}@${branch}`);
        }
        result = subtree;
    }

    const exts = normalizeExtensions(extensions);
    if (exts) {
        result = pruneByExtensions(result, exts) || { ...result, files: [] };
    }

    return result;
}

function collectIconNames(node, set) {
    if (node.type === "blob") {
        set.add(node.icon);
    } else {
        node.files.forEach((f) => collectIconNames(f, set));
    }
}

// ---------------------------------------------------------------------------
// Popup DOM
// ---------------------------------------------------------------------------

let stylesInjected = false;
function ensureStyles() {
    if (stylesInjected) return;
    stylesInjected = true;
    const style = document.createElement("style");
    style.textContent = `
        .ffx-overlay { position: fixed; inset: 0; background: rgba(0,0,0,.55); display: flex;
            align-items: center; justify-content: center; z-index: 999999; font-family: system-ui, sans-serif; }
        .ffx-popup { background: #1e1e1e; color: #eee; width: min(480px, 90vw); max-height: 80vh;
            display: flex; flex-direction: column; border-radius: 8px; overflow: hidden;
            box-shadow: 0 10px 40px rgba(0,0,0,.4); }
        .ffx-header { display: flex; align-items: center; justify-content: space-between;
            border-bottom: 1px solid #333; padding: 0 4px 0 8px; flex-shrink: 0; }
        .ffx-tabs { display: flex; gap: 4px; }
        .ffx-tab { padding: 10px 12px; cursor: pointer; opacity: .6; font-size: 14px; user-select: none; }
        .ffx-tab.ffx-active { opacity: 1; border-bottom: 2px solid #4ea1ff; }
        .ffx-close { background: none; border: none; color: #eee; font-size: 20px; cursor: pointer;
            line-height: 1; padding: 4px 10px; }
        .ffx-close:hover { color: #ff6b6b; }
        .ffx-content { overflow-y: auto; padding: 8px; }
        .ffx-tab-content { display: none; }
        .ffx-tab-content.ffx-active { display: block; }
        .ffx-status { opacity: .7; font-size: 13px; padding: 8px 4px; }
        .ffx-status.ffx-error { color: #ff6b6b; white-space: pre-wrap; }
        .ffx-line { display: flex; align-items: center; gap: 6px; padding: 4px 6px; border-radius: 4px;
            cursor: pointer; font-size: 13px; }
        .ffx-line:hover { background: #2a2a2a; }
        .ffx-icon { width: 16px; height: 16px; display: inline-flex; align-items: center;
            justify-content: center; flex-shrink: 0; }
        .ffx-icon img { width: 16px; height: 16px; }
        .ffx-children { margin-left: 16px; }
        .ffx-collapsed > .ffx-children { display: none; }
        .ffx-dropzone { border: 2px dashed #444; border-radius: 6px; padding: 40px 12px;
            text-align: center; opacity: .7; cursor: pointer; }
        .ffx-dropzone:hover, .ffx-dropzone:focus-visible { opacity: 1; border-color: #666; outline: none; }
        .ffx-dropzone.ffx-drag-over { border-color: #4ea1ff; opacity: 1; }
        .ffx-dropzone-sub { font-size: 12px; opacity: .7; }
        .ffx-file-input { display: none; }
        .ffx-change-container { position: fixed; right: 12px; bottom: 12px; display: flex;
            flex-direction: column; gap: 6px; z-index: 999998; align-items: flex-end; }
        .ffx-change-btn { display: flex; align-items: center; gap: 6px; background: #1e1e1e; color: #ccc;
            border: 1px solid #333; border-radius: 999px; padding: 6px 10px; font-size: 12px; cursor: pointer;
            opacity: .55; transition: opacity .15s; font-family: system-ui, sans-serif; }
        .ffx-change-btn:hover { opacity: 1; }
        .ffx-change-icon { opacity: .8; }
    `;
    document.head.appendChild(style);
}

function buildPopup() {
    const overlay = document.createElement("div");
    overlay.className = "ffx-overlay";

    const popup = document.createElement("div");
    popup.className = "ffx-popup";

    const header = document.createElement("div");
    header.className = "ffx-header";

    const tabsBar = document.createElement("div");
    tabsBar.className = "ffx-tabs";

    const closeBtn = document.createElement("button");
    closeBtn.className = "ffx-close";
    closeBtn.type = "button";
    closeBtn.textContent = "\u00D7";
    closeBtn.setAttribute("aria-label", "Close");

    const contentWrap = document.createElement("div");
    contentWrap.className = "ffx-content";

    const tabDefs = [
        { id: "github", label: "GitHub" },
        { id: "local", label: "Local" },
        { id: "drop", label: "Drop" },
    ];
    const tabs = {};

    tabDefs.forEach((def, i) => {
        const tabBtn = document.createElement("span");
        tabBtn.className = "ffx-tab" + (i === 0 ? " ffx-active" : "");
        tabBtn.textContent = def.label;

        const content = document.createElement("div");
        content.className = "ffx-tab-content" + (i === 0 ? " ffx-active" : "");
        tabs[def.id] = content;

        tabBtn.addEventListener("click", () => {
            tabsBar.querySelectorAll(".ffx-tab").forEach((t) => t.classList.remove("ffx-active"));
            contentWrap.querySelectorAll(".ffx-tab-content").forEach((c) => c.classList.remove("ffx-active"));
            tabBtn.classList.add("ffx-active");
            content.classList.add("ffx-active");
        });

        tabsBar.appendChild(tabBtn);
        contentWrap.appendChild(content);
    });

    header.append(tabsBar, closeBtn);
    popup.append(header, contentWrap);
    overlay.appendChild(popup);

    return { overlay, tabs, closeBtn };
}

function renderNode(container, node, icons, onSelect) {
    const row = document.createElement("div");
    row.className = "ffx-node " + (node.type === "tree" ? "ffx-tree" : "ffx-blob");

    const line = document.createElement("div");
    line.className = "ffx-line";

    const icon = document.createElement("span");
    icon.className = "ffx-icon";
    if (node.type === "blob") {
        const src = icons[node.icon];
        if (src) {
            const img = document.createElement("img");
            img.src = src;
            img.alt = "";
            icon.appendChild(img);
        } else {
            icon.textContent = "\uD83D\uDCC4"; // page
        }
    } else {
        icon.textContent = "\uD83D\uDCC1"; // folder
    }

    const name = document.createElement("span");
    name.textContent = node.name;

    line.append(icon, name);
    row.appendChild(line);

    if (node.type === "tree") {
        row.classList.add("ffx-collapsed");
        const children = document.createElement("div");
        children.className = "ffx-children";
        line.addEventListener("click", () => row.classList.toggle("ffx-collapsed"));
        node.files.forEach((child) => renderNode(children, child, icons, onSelect));
        row.appendChild(children);
    } else {
        line.addEventListener("click", () => onSelect(node.path));
    }

    container.appendChild(row);
}

async function renderGithubTab(container, user, repo, branch, onSelect, { rootPath, extensions } = {}) {
    container.textContent = "";
    const status = document.createElement("div");
    status.className = "ffx-status";
    status.textContent = `Loading ${user}/${repo}@${branch}${rootPath ? "/" + rootPath : ""} \u2026`;
    container.appendChild(status);

    try {
        const root = await fetchForgeTree(user, repo, branch, { rootPath, extensions });

        const iconNames = new Set();
        collectIconNames(root, iconNames);
        const entries = await Promise.all(
            [...iconNames].map(async (name) => [name, await loadSvgIcon(name)])
        );
        const icons = Object.fromEntries(entries);

        status.remove();
        if (root.files.length === 0) {
            const empty = document.createElement("div");
            empty.className = "ffx-status";
            empty.textContent = extensions && extensions.length
                ? `No files matching ${normalizeExtensions(extensions).join(", ")} found.`
                : "Repo tree is empty.";
            container.appendChild(empty);
            return;
        }
        root.files.forEach((child) => renderNode(container, child, icons, onSelect));
    } catch (err) {
        status.textContent = `Failed to load ${user}/${repo}@${branch}:\n${err.message}`;
        status.classList.add("ffx-error");
    }
}

function renderLocalTab(container, paths, onSelect) {
    container.textContent = "";
    if (!paths || paths.length === 0) {
        const empty = document.createElement("div");
        empty.className = "ffx-status";
        empty.textContent = "No local paths provided.";
        container.appendChild(empty);
        return;
    }
    for (const path of paths) {
        const line = document.createElement("div");
        line.className = "ffx-line";
        const icon = document.createElement("span");
        icon.className = "ffx-icon";
        icon.textContent = "\uD83D\uDCBB"; // laptop
        const name = document.createElement("span");
        name.textContent = path;
        line.append(icon, name);
        line.addEventListener("click", () => onSelect(path));
        container.appendChild(line);
    }
}

function renderDropTab(container, onSelect, { extensions } = {}) {
    container.textContent = "";

    const input = document.createElement("input");
    input.type = "file";
    input.className = "ffx-file-input";
    const exts = normalizeExtensions(extensions);
    if (exts) input.accept = exts.join(",");
    input.addEventListener("change", () => {
        const file = input.files[0];
        if (file) onSelect(file);
    });

    const zone = document.createElement("div");
    zone.className = "ffx-dropzone";
    zone.tabIndex = 0;
    zone.innerHTML = `Drop a file here<br><span class="ffx-dropzone-sub">or click to browse</span>`;
    zone.addEventListener("click", () => input.click());
    zone.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            input.click();
        }
    });
    zone.addEventListener("dragover", (e) => {
        e.preventDefault();
        zone.classList.add("ffx-drag-over");
    });
    zone.addEventListener("dragleave", () => zone.classList.remove("ffx-drag-over"));
    zone.addEventListener("drop", (e) => {
        e.preventDefault();
        zone.classList.remove("ffx-drag-over");
        const file = e.dataTransfer.files[0];
        if (file) onSelect(file);
    });

    container.append(zone, input);
}

// ---------------------------------------------------------------------------
// Picker popup wiring (shared by the initial fs_fetch() call and the
// "change artifact" button)
// ---------------------------------------------------------------------------

function showPickerPopup({ user, repo, branch, localPaths, rootPath, extensions, onPick, onCancel }) {
    const ui = buildPopup();
    document.body.appendChild(ui.overlay);

    let settled = false;
    const cleanup = () => ui.overlay.remove();

    const pick = (bufferPromise, selection) => {
        if (settled) return;
        settled = true;
        cleanup();
        onPick(bufferPromise, selection);
    };

    ui.closeBtn.addEventListener("click", () => {
        if (settled) return;
        settled = true;
        cleanup();
        onCancel();
    });

    renderLocalTab(ui.tabs.local, localPaths, (path) =>
        pick(
            fetch(path).then((r) => {
                if (!r.ok) throw new Error(`local fetch failed: ${path} (${r.status})`);
                return r.arrayBuffer();
            }),
            { source: "local", path }
        )
    );

    renderDropTab(ui.tabs.drop, (file) => pick(file.arrayBuffer(), null), { extensions });

    renderGithubTab(
        ui.tabs.github,
        user,
        repo,
        branch,
        (path) =>
            pick(
                fetch(`https://raw.githubusercontent.com/${user}/${repo}/${branch}/${path}`).then((r) => {
                    if (!r.ok) throw new Error(`raw fetch failed: ${path} (${r.status})`);
                    return r.arrayBuffer();
                }),
                { source: "github", path }
            ),
        { rootPath, extensions }
    );
}

function autoFetch(selection, user, repo, branch) {
    if (selection.source === "github") {
        return fetch(`https://raw.githubusercontent.com/${user}/${repo}/${branch}/${selection.path}`).then((r) => {
            if (!r.ok) throw new Error(`raw fetch failed: ${selection.path} (${r.status})`);
            return r.arrayBuffer();
        });
    }
    if (selection.source === "local") {
        return fetch(selection.path).then((r) => {
            if (!r.ok) throw new Error(`local fetch failed: ${selection.path} (${r.status})`);
            return r.arrayBuffer();
        });
    }
    return Promise.reject(new Error(`cannot auto-restore selection from source: ${selection.source}`));
}

// ---------------------------------------------------------------------------
// "Change artifact" floating button
// ---------------------------------------------------------------------------

let changeBtnContainer = null;
function getChangeBtnContainer() {
    if (!changeBtnContainer) {
        changeBtnContainer = document.createElement("div");
        changeBtnContainer.className = "ffx-change-container";
        document.body.appendChild(changeBtnContainer);
    }
    return changeBtnContainer;
}

const changeButtonsShown = new Set();

function escapeHtml(s) {
    return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

function attachChangeButton({ key, user, repo, branch, localPaths, rootPath, extensions }) {
    if (changeButtonsShown.has(key)) return;
    changeButtonsShown.add(key);

    const selection = loadLastSelection(key);
    const label = selection ? selection.path.split("/").pop() : key;

    const btn = document.createElement("button");
    btn.className = "ffx-change-btn";
    btn.type = "button";
    btn.title = `Change artifact (currently: ${label})`;
    btn.innerHTML = `<span class="ffx-change-label">${escapeHtml(label)}</span><span class="ffx-change-icon">\u21C4</span>`;

    btn.addEventListener("click", () => {
        showPickerPopup({
            user, repo, branch, localPaths, rootPath, extensions,
            onPick: (bufferPromise, sel) => {
                bufferPromise
                    .then(() => {
                        if (sel) saveLastSelection(key, sel);
                        else clearLastSelection(key);
                        location.reload();
                    })
                    .catch((err) => console.error("fs_fetch: failed to load new selection", err));
            },
            onCancel: () => {},
        });
    });

    getChangeBtnContainer().appendChild(btn);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Show the forge/local/drop picker (or auto-resolve the remembered
 * selection) and resolve to the selected blob's ArrayBuffer. Rejects if the
 * user closes the popup without selecting.
 *
 * @param {string} user       GitHub user/org that owns the forge repo
 * @param {string} repo       forge repo name
 * @param {string} branch     branch (or ref) to browse
 * @param {string[]} [localPaths]  paths (relative to the current page) shown in the "Local" tab
 * @param {{key?: string, showChangeButton?: boolean, rootPath?: string, extensions?: string[]}} [options]
 * @returns {Promise<ArrayBuffer>}
 */
function fs_fetch(user, repo, branch, localPaths = [], options = {}) {
    const { rootPath, extensions } = options;
    const key = options.key || defaultKey(user, repo, branch, rootPath);
    const showChangeButton = options.showChangeButton !== false;
    ensureStyles();

    return new Promise((resolve, reject) => {
        const openPicker = () => {
            showPickerPopup({
                user, repo, branch, localPaths, rootPath, extensions,
                onPick: (bufferPromise, selection) => {
                    bufferPromise
                        .then((buf) => {
                            if (selection) saveLastSelection(key, selection);
                            resolve(buf);
                            if (showChangeButton) {
                                attachChangeButton({ key, user, repo, branch, localPaths, rootPath, extensions });
                            }
                        })
                        .catch(reject);
                },
                onCancel: () => reject(new Error("fs_fetch: selection cancelled")),
            });
        };

        const remembered = loadLastSelection(key);
        if (!remembered) {
            openPicker();
            return;
        }

        autoFetch(remembered, user, repo, branch)
            .then((buf) => {
                resolve(buf);
                if (showChangeButton) {
                    attachChangeButton({ key, user, repo, branch, localPaths, rootPath, extensions });
                }
            })
            .catch((err) => {
                console.warn("fs_fetch: stored selection is no longer available, showing picker", err);
                clearLastSelection(key);
                openPicker();
            });
    });
}

/**
 * Forget the remembered selection for a given (user, repo, branch[, key]).
 * The next fs_fetch() call for that key will show the picker again.
 */
function fs_reset(user, repo, branch, options = {}) {
    const key = options.key || defaultKey(user, repo, branch, options.rootPath);
    clearLastSelection(key);
}
