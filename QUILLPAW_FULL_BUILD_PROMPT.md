# QUILLPAW — AI Coding Agent Master Prompt (Full Build V1–V4)
### Offline Academic Note-Taking App | Tauri + Rust + Svelte

---

> **HOW TO USE THIS PROMPT**
> Paste this entire file into OpenAI Codex (web), Claude Code, Gemini CLI, or any agentic coding tool.
> It is self-contained. The agent has everything it needs to build the complete application.
> Sections marked `[AGENT ACTION]` are explicit instructions to execute.
> Sections marked `[SPEC]` are reference material the agent must follow throughout.
>
> **This prompt builds the entire application in one session — V1 through V4 — as a single
> continuous build sequence. Do not stop after any phase. Complete all 28 steps.**

---

## MISSION

You are building **Quillpaw** — a Windows-first offline desktop academic note-taking application.

It combines:
- The **speed and local-file philosophy** of Obsidian
- The **structural block system** of Notion
- The **drawing/handwriting integration** of Microsoft OneNote

It must be **fully functional without internet access**, privacy-first by default, and optionally
enhanced by local AI that the user explicitly controls.

**Build the complete application: V1 Foundation through V4 Semantic AI. All phases. No stopping.**

---

## COMPLETE FEATURE SCOPE

**V1 — Foundation**
Tauri v2 + Svelte 5 scaffold, three-panel layout, CodeMirror 6 editor, vault folder browser,
note CRUD, file drag-and-drop, tantivy search, warm dark theme, keyboard shortcuts,
command palette, settings panel.

**V2 — Academic Blocks**
LaTeX math (KaTeX), drawing canvas (Skia), block command system (/), callout blocks,
fenced code with syntax highlighting and copy button, PDF inline viewer, table editor.

**V3 — Local AI + Speech**
llama.cpp FFI, Phi-3 Mini model support, AI proposal panel, note summarization,
contextual Q&A, deadline/reminder detection, whisper.cpp STT, live lecture transcription.

**V4 — Semantic Search**
nomic-embed-text embedding pipeline, HNSW vector index (usearch), semantic natural-language
search UI, tag suggestions, NPU scheduling for Intel Core Ultra hardware.

---

## [SPEC] TECHNOLOGY STACK

| Layer             | Technology                    | Notes                                      |
|-------------------|-------------------------------|---------------------------------------------|
| App Framework     | Tauri v2                      | Rust backend + WebView2 on Windows          |
| UI Framework      | Svelte 5 + TailwindCSS v4     | Compiled reactivity, no virtual DOM         |
| Editor            | CodeMirror 6                  | Markdown, syntax highlighting, extensible   |
| Drawing           | rust-skia                     | GPU-accelerated vector canvas               |
| Backend           | Rust (stable)                 | All file IO, search, AI inference           |
| Keyword Search    | tantivy                       | Full-text inverted index                    |
| Semantic Search   | usearch (HNSW)                | Approximate nearest-neighbor vector search  |
| LLM Inference     | llama-cpp-2 crate             | GGUF models, INT4 quantization              |
| Embeddings        | nomic-embed-text-v1.5 GGUF   | Local 768-dim embeddings                    |
| STT               | whisper-rs crate              | Real-time streaming ASR                     |
| Math              | KaTeX (npm)                   | Fast LaTeX rendering in WebView             |
| PDF               | PDF.js (npm)                  | Inline PDF rendering                        |
| Async Runtime     | tokio                         | Non-blocking IO and background tasks        |
| Icons             | Phosphor Icons (npm)          | Consistent minimal academic aesthetic       |

**CRITICAL RULES — NEVER VIOLATE:**
- Do NOT use Electron. Tauri only.
- Do NOT use React or Vue. Svelte only.
- Do NOT use SQLite for notes. Notes are .md plain files only.
- Do NOT make network calls in core code.
- All Rust async code must use tokio. No blocking IO.
- All UI colors must use CSS variables. No hardcoded hex in components.

---

## [SPEC] COMPLETE PROJECT STRUCTURE

```
quillpaw/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── fs_commands.rs
│   │   │   ├── search_commands.rs
│   │   │   ├── ai_commands.rs
│   │   │   ├── stt_commands.rs
│   │   │   └── drawing_commands.rs
│   │   ├── fs_manager.rs
│   │   ├── search.rs
│   │   ├── watcher.rs
│   │   ├── ai_engine.rs
│   │   ├── embeddings.rs
│   │   ├── stt_engine.rs
│   │   ├── drawing.rs
│   │   └── models.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Editor.svelte
│   │   │   ├── FileTree.svelte
│   │   │   ├── AIPanel.svelte
│   │   │   ├── SearchModal.svelte
│   │   │   ├── CommandPalette.svelte
│   │   │   ├── TabBar.svelte
│   │   │   ├── Settings.svelte
│   │   │   ├── DrawingCanvas.svelte
│   │   │   ├── LectureMode.svelte
│   │   │   └── ProposalCard.svelte
│   │   ├── stores/
│   │   │   ├── vault.ts
│   │   │   ├── editor.ts
│   │   │   ├── ui.ts
│   │   │   ├── ai.ts
│   │   │   └── stt.ts
│   │   └── utils/
│   │       ├── tauri_bridge.ts
│   │       ├── markdown.ts
│   │       └── shortcuts.ts
│   ├── app.css
│   ├── App.svelte
│   └── main.ts
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tailwind.config.js
```

---

## [SPEC] VAULT STORAGE FORMAT

Rule: Notes are plain .md files. Never store note content in a database.

```
Vault/
  ├── Physics/
  │   ├── entropy.md
  │   └── quantum.md
  └── .assets/
      ├── images/
      ├── drawings/        <- vector JSON files
      ├── audio/           <- lecture recordings
      └── files/

Vault/.quillpaw/
  ├── index/               <- tantivy search index
  ├── embeddings/          <- HNSW vector index
  └── config.json
```

Note frontmatter format:
```
---
title: "Entropy and Thermodynamics"
created: 2024-09-01T10:23:00Z
modified: 2024-11-15T16:44:00Z
tags: [physics, thermodynamics]
aliases: ["second law"]
drawing-refs: [sketch_entropy.json]
---
```

File write rules: atomic writes (.md.tmp then rename). Always UTF-8. Never corrupt on crash.

---

## [SPEC] COMPLETE RUST BACKEND — ALL TAURI COMMANDS

All commands async, return Result<T, String>.

### fs_commands.rs
```rust
#[tauri::command] async fn open_vault(app: AppHandle) -> Result<String, String>
#[tauri::command] async fn get_file_tree(vault_path: String) -> Result<Vec<FileNode>, String>
#[tauri::command] async fn read_note(path: String) -> Result<NoteContent, String>
#[tauri::command] async fn save_note(path: String, content: String) -> Result<(), String>
#[tauri::command] async fn create_note(vault_path: String, folder: String, title: String) -> Result<String, String>
#[tauri::command] async fn delete_item(path: String) -> Result<(), String>
#[tauri::command] async fn rename_item(old_path: String, new_name: String) -> Result<String, String>
#[tauri::command] async fn create_folder(vault_path: String, folder_name: String) -> Result<(), String>
#[tauri::command] async fn import_asset(vault_path: String, source_path: String) -> Result<String, String>
#[tauri::command] async fn resolve_asset(vault_path: String, filename: String) -> Result<String, String>
```

### search_commands.rs
```rust
#[tauri::command] async fn build_search_index(vault_path: String) -> Result<(), String>
#[tauri::command] async fn search_notes(vault_path: String, query: String) -> Result<Vec<SearchResult>, String>
#[tauri::command] async fn search_semantic(vault_path: String, query: String) -> Result<Vec<SearchResult>, String>
```

### ai_commands.rs
```rust
#[tauri::command] async fn load_ai_model(model_path: String) -> Result<(), String>
#[tauri::command] async fn unload_ai_model() -> Result<(), String>
#[tauri::command] async fn summarize_note(note_content: String) -> Result<AiProposal, String>
#[tauri::command] async fn ask_question(vault_path: String, question: String) -> Result<AiProposal, String>
#[tauri::command] async fn detect_reminders(note_content: String) -> Result<Vec<AiProposal>, String>
#[tauri::command] async fn suggest_tags(note_content: String) -> Result<AiProposal, String>
// ONLY this function may write AI output to disk:
#[tauri::command] async fn apply_ai_proposal(proposal_id: String, target_path: String) -> Result<(), String>
```

### stt_commands.rs
```rust
#[tauri::command] async fn start_lecture_mode(app: AppHandle) -> Result<(), String>
#[tauri::command] async fn stop_lecture_mode() -> Result<(), String>
#[tauri::command] async fn list_audio_devices() -> Result<Vec<String>, String>
#[tauri::command] async fn set_audio_device(device_name: String) -> Result<(), String>
```

### drawing_commands.rs
```rust
#[tauri::command] async fn render_drawing_png(drawing_json: String) -> Result<Vec<u8>, String>
#[tauri::command] async fn save_drawing(vault_path: String, filename: String, drawing_json: String) -> Result<String, String>
#[tauri::command] async fn load_drawing(vault_path: String, filename: String) -> Result<String, String>
```

### models.rs
```rust
#[derive(Serialize, Deserialize)]
pub struct FileNode {
    pub name: String, pub path: String,
    pub is_folder: bool, pub children: Option<Vec<FileNode>>,
}

#[derive(Serialize, Deserialize)]
pub struct NoteContent {
    pub path: String, pub title: String, pub body: String,
    pub tags: Vec<String>, pub created: String, pub modified: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String, pub title: String, pub snippet: String,
    pub score: f32, pub result_type: String,  // "keyword" or "semantic"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AiProposal {
    pub id: String,               // UUID
    pub proposal_type: String,    // "summary" | "qa" | "reminder" | "tags" | "edit"
    pub title: String,
    pub content: String,
    pub target_path: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
```

---

## [SPEC] DRAWING STORAGE FORMAT

```json
{
  "version": 2,
  "canvas": { "width": 1920, "height": 1080, "background": "#1C1814" },
  "strokes": [
    { "id": "s_001", "tool": "pen", "color": "#C9843A", "width": 2.5,
      "opacity": 1.0, "points": [[120.3, 204.1, 0.8], [121.0, 205.4, 0.9]] }
  ],
  "shapes": [
    { "id": "r_001", "type": "rect", "x": 50, "y": 50, "w": 200, "h": 100, "stroke": "#EDE0CC" }
  ],
  "text_layers": [
    { "id": "t_001", "x": 100, "y": 300, "text": "annotation", "size": 16, "color": "#EDE0CC" }
  ]
}
```

---

## [SPEC] VISUAL DESIGN — DARK WARM THEME

Quillpaw's aesthetic is dark but warm. A candlelit scholar's den at midnight.
Deep warm browns, amber accents, parchment-toned text. Never cold. Never sterile.

### CSS Variables — implement exactly in app.css

```css
:root {
  --bg-base:        #1C1814;   /* Editor background */
  --bg-panel:       #17140F;   /* Side panels */
  --bg-surface:     #252018;   /* Cards, hover states */
  --bg-elevated:    #2E2820;   /* Modals, dropdowns */

  --border:         #332C22;
  --border-focus:   #C9843A;
  --border-subtle:  #221E16;

  --text-primary:   #EDE0CC;   /* Warm off-white, aged paper */
  --text-secondary: #9E8E78;
  --text-muted:     #5C5040;

  --accent:         #C9843A;   /* Amber/candlelight */
  --accent-bright:  #E8A855;
  --accent-subtle:  rgba(201, 132, 58, 0.12);
  --accent-glow:    rgba(201, 132, 58, 0.06);

  --accent2:        #A0705A;   /* Dusty rose for tags */
  --accent2-subtle: rgba(160, 112, 90, 0.15);

  --success:  #6DAA72;
  --warning:  #C9A03A;
  --danger:   #B05A5A;

  --font-ui:     'Literata', 'Georgia', serif;
  --font-mono:   'JetBrains Mono', 'Fira Code', monospace;
  --font-editor: 'Lora', 'Georgia', serif;

  --space-1: 4px; --space-2: 8px; --space-3: 12px;
  --space-4: 16px; --space-6: 24px; --space-8: 32px;

  --radius-sm: 5px; --radius-md: 9px; --radius-lg: 14px;
  --transition: 140ms ease;

  scrollbar-color: #3D3020 #17140F;
  scrollbar-width: thin;
}
```

### Styling Rules
- Load Google Fonts: Literata, Lora, JetBrains Mono via @import in app.css
- UI: 13px Literata | Editor: 16px Lora, line-height 1.8 | Code: 14px JetBrains Mono
- Active file tree items: 3px amber left border + --accent-subtle background
- Focus rings: outline 2px solid var(--accent), outline-offset 2px
- Modals: --bg-elevated, box-shadow 0 8px 40px rgba(0,0,0,0.6)
- Folder icons: Phosphor FolderSimple in --accent2
- File icons: Phosphor FileText in --text-secondary
- AI Proposal cards: amber left border + --accent-glow background
- Toast errors: amber border, --bg-elevated background

### CodeMirror Custom Theme
Background --bg-base. Cursor --accent 2px wide. Selection --accent-subtle.
Line numbers --text-muted. Active line rgba(201,132,58,0.04). Brackets --accent-subtle.

---

## [SPEC] EDITOR — FULL FEATURE SET

Standard markdown: H1-H6, bold, italic, strikethrough, inline code, lists, blockquotes,
footnotes, tables, fenced code, links [text](url), wiki-links [[note-title]], images ![[file]].

Extended blocks:
- $...$ inline LaTeX and $$...$$ display LaTeX via KaTeX
- Fenced code with per-language syntax highlighting and copy button
- ![[drawing.json]] renders inline Skia drawing canvas
- > [!note], > [!warning], > [!tip], > [!important] callout blocks
- / command palette at line start

Block commands:
  /drawing, /math, /table, /code, /pdf, /file, /callout, /toc, /reminder

Editor shortcuts: Ctrl+B, Ctrl+I, Ctrl+K, Ctrl+`, Ctrl+Shift+C, Ctrl+D, Tab/Shift+Tab

---

## [SPEC] DRAWING SYSTEM

Tools: Pen (pressure-sensitive), Highlighter, Eraser, Shapes (Line/Rect/Ellipse/Arrow),
Lasso Select, Text annotation. 16 color presets + hex. Width slider.
Embedded inline via /drawing. Opens focused overlay on click.
Auto-saves to .assets/drawings/ as JSON. Undo/redo Ctrl+Z/Ctrl+Y.

---

## [SPEC] SEARCH SYSTEM

Keyword (tantivy): Index path/title/body/tags in .quillpaw/index/. Rebuild on vault open,
incremental via notify watcher. Boolean operators, field-specific (tag:physics), fuzzy, regex.

Semantic (usearch + nomic-embed): Background embedding pipeline, CPU idle throttled.
HNSW index in .quillpaw/embeddings/. Natural language query -> nearest-neighbor search.

SearchModal.svelte: Ctrl+Shift+F. Tabs: Keyword | Semantic | Smart (merged).
Live results 150ms debounce. Arrow key navigation. Folder filter toggles.

---

## [SPEC] AI SYSTEM — PROPOSAL CONTRACT

THE RULE:
  1. User explicitly triggers AI (button or command)
  2. AI processes in background
  3. Output appears in AI Panel as a PROPOSAL CARD
  4. Card shows title, content preview, diff if applicable
  5. User clicks [Accept], [Edit], or [Dismiss]
  6. ONLY [Accept] -> apply_ai_proposal() -> write to disk

Rust enforcement: AI commands return only AiProposal structs. apply_ai_proposal is the
SOLE function that calls save_note with AI content. This is a hard architectural rule.

AI Panel sections: Proposals, Summarize, Ask Q&A, Reminders scan, Tag Suggestions, Model Status.
Proposal cards: amber left border, --accent-glow background, Accept/Edit/Dismiss buttons.

AI Modes: OFF (default) / Low-NPU / Balanced-GPU / Full-CPU+GPU
Models: Phi-3 Mini 3.8B INT4 GGUF (~2GB), Qwen2.5 1.5B INT4 GGUF (~800MB), user GGUF path.

---

## [SPEC] LECTURE MODE (STT)

LectureMode.svelte docked bar above editor. Trigger: Ctrl+Shift+L.

Pipeline: cpal (WASAPI on Windows) -> 16kHz resample (rubato) -> silero-vad (ort) ->
whisper-rs -> emit stt-text-chunk Tauri events -> text appends to note at cursor.

Target latency <1.5s. Model: distil-whisper-small. Audio optionally saved to .assets/audio/.
User can freely edit note while recording continues.

---

## [SPEC] SETTINGS PANEL

Slide-in drawer (Ctrl+,):
- General: vault path, theme (Dark Warm/Dark Cool/Light Parchment), font size, editor font
- Editor: auto-save delay, spell check, line numbers, focus mode
- AI: master toggle (default OFF), mode selector, model path/download, RAM indicator
- Speech: microphone selector, STT model selector, auto-save audio toggle
- Privacy: telemetry toggle (default OFF) + notice "Quillpaw stores all data locally."

---

## [SPEC] COMPLETE CARGO.TOML

```toml
[package]
name = "quillpaw"
version = "1.0.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["dialog", "fs", "shell", "notification"] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-shell = "2"
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tantivy = "0.22"
usearch = "2"
notify = "6"
walkdir = "2"
gray_matter = "0.2"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
trash = "3"
uuid = { version = "1", features = ["v4"] }
llama-cpp-2 = "0.1"
whisper-rs = "0.11"
ort = { version = "2", features = ["download-binaries"] }
cpal = "0.15"
rubato = "0.14"
ndarray = "0.15"
```

---

## [SPEC] COMPLETE PACKAGE.JSON

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@tauri-apps/plugin-store": "^2",
    "@codemirror/lang-markdown": "^6",
    "@codemirror/state": "^6",
    "@codemirror/view": "^6",
    "@codemirror/commands": "^6",
    "@codemirror/language": "^6",
    "@codemirror/language-data": "^6",
    "@lezer/highlight": "^1",
    "katex": "^0.16",
    "pdfjs-dist": "^4",
    "svelte": "^5",
    "@sveltejs/vite-plugin-svelte": "^4",
    "@phosphor-icons/svelte": "^2"
  },
  "devDependencies": {
    "tailwindcss": "^4",
    "vite": "^6",
    "typescript": "^5"
  }
}
```

---

## [SPEC] TAURI CONFIGURATION

```json
{
  "app": {
    "windows": [{
      "title": "Quillpaw", "width": 1280, "height": 800,
      "minWidth": 800, "minHeight": 600,
      "decorations": false, "transparent": false, "theme": "Dark"
    }],
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "identifier": "app.quillpaw.desktop",
    "productName": "Quillpaw",
    "version": "1.0.0",
    "targets": ["nsis", "msi"]
  }
}
```

---

## [SPEC] GLOBAL KEYBOARD SHORTCUTS

Register in shortcuts.ts, applied in App.svelte:

  Ctrl+N          -> New note (current folder)
  Ctrl+Shift+N    -> New note (folder picker)
  Ctrl+P          -> Command palette
  Ctrl+Shift+F    -> Search (keyword)
  Ctrl+Shift+S    -> Search (semantic)
  Ctrl+B          -> Toggle left panel
  Ctrl+/          -> Toggle right panel
  Ctrl+W          -> Close active tab
  Ctrl+Tab        -> Next tab
  Ctrl+Shift+Tab  -> Previous tab
  Ctrl+S          -> Force save
  Ctrl+,          -> Settings
  Ctrl+D          -> Drawing canvas
  Ctrl+Shift+L    -> Toggle lecture mode
  F2              -> Rename focused file
  Delete          -> Delete focused file (confirm dialog)
  Escape          -> Close modal / cancel

---

## [AGENT ACTION] COMPLETE BUILD SEQUENCE

Execute every step in order. Do not skip. Do not stop between phases.
Complete all 28 steps before finishing.

---

### PHASE 1 — FOUNDATION (V1)

#### Step 1 — Scaffold
  npm create tauri-app@latest quillpaw -- --template svelte-ts
  cd quillpaw && npm install
Verify app opens with "cargo tauri dev" before continuing.

#### Step 2 — Install all dependencies
Install all packages from Cargo.toml and package.json specs above. Verify cargo build succeeds.

#### Step 3 — Design system
Create src/app.css with all CSS variables. Import Google Fonts (Literata, Lora, JetBrains Mono).
Configure TailwindCSS v4 to reference the CSS variables.

#### Step 4 — Layout shell
Build App.svelte: three-panel layout, custom title bar (draggable, Quillpaw wordmark with
PenNib icon, custom window buttons), placeholder panels, status bar.

#### Step 5 — Rust file system layer
Implement all commands in fs_commands.rs. Register in main.rs. Verify each command works.

#### Step 6 — Vault onboarding
First-launch welcome screen: Quillpaw logo with warm amber glow, tagline "Your notes.
Your machine. Your den." Open Vault / Create Vault buttons. Persist via tauri-plugin-store.

#### Step 7 — File tree
FileTree.svelte: collapsible folders, file icons, amber selected state (3px left border +
--accent-subtle), right-click context menu (New Note, New Folder, Rename, Delete),
file watcher refresh via watcher.rs.

#### Step 8 — Editor (core)
Editor.svelte: CodeMirror 6 with custom warm dark theme, full markdown, [[wiki-link]]
highlighting, inline ![[image]] rendering, 800ms auto-save, unsaved dot indicator.

#### Step 9 — Tab bar
TabBar.svelte: open note tabs, close buttons, amber unsaved dot, Ctrl+Tab navigation.

#### Step 10 — Keyword search
tantivy index in search.rs. Hook to vault open and file watcher. SearchModal.svelte with
live results, snippets, folder filters. Trigger Ctrl+Shift+F.

#### Step 11 — Command palette
CommandPalette.svelte: Ctrl+P, fuzzy search over notes and commands, keyboard navigable.

#### Step 12 — Settings panel
Settings.svelte slide-in drawer. All sections present. AI and STT disabled with "coming
in a later step" state until built.

#### Step 13 — V1 polish
All keyboard shortcuts. Drag-and-drop file import. Status bar word/tag count. F2 inline
rename. Warm confirm dialog before delete. Smooth panel resize.

COMPILE CHECK: Verify all V1 features work before proceeding to Phase 2.

---

### PHASE 2 — ACADEMIC BLOCKS (V2)

#### Step 14 — LaTeX math
CodeMirror extension for $...$ and $$...$$ rendered inline via KaTeX. Updates as user
types, under 10ms latency.

#### Step 15 — Enhanced code blocks
Per-language syntax highlighting via CodeMirror language-data. Copy button on hover
(top-right). Language label (top-left).

#### Step 16 — Drawing canvas
drawing.rs Rust backend via rust-skia. DrawingCanvas.svelte with all drawing tools from
the Drawing spec. Auto-saves to .assets/drawings/ as JSON. Inline rendering via
![[drawing.json]]. Undo/redo Ctrl+Z/Ctrl+Y.

#### Step 17 — Block command system
/ at line start opens floating mini-palette. Implement all block commands from the
Block Commands spec.

#### Step 18 — Callout blocks
Parse > [!note], > [!warning], > [!tip], > [!important]. Render as colored callout boxes
with Phosphor icons and warm-tinted backgrounds per type.

#### Step 19 — PDF inline viewer
PDF.js integration. /pdf command: file picker -> copy to .assets/files/ -> insert
![[filename.pdf]] -> inline viewer with scroll and zoom.

COMPILE CHECK: Drawing saves/loads, LaTeX renders, PDF embeds. Before proceeding.

---

### PHASE 3 — LOCAL AI + SPEECH (V3)

#### Step 20 — AI engine backend
ai_engine.rs with llama-cpp-2. Load/unload GGUF models. Implement all ai_commands.rs
commands. Enforce proposal contract: AI commands return only AiProposal structs.
apply_ai_proposal is the only function writing AI output to disk.

#### Step 21 — AI panel
Full AIPanel.svelte: Proposals, Summarize, Ask, Reminders, Tag Suggestions, Model Status.
Proposal cards: amber left border, --accent-glow background, Accept/Edit/Dismiss buttons.

#### Step 22 — AI features
Wire all four capabilities:
- Summarize button -> summarize_note -> proposal card
- Q&A text input -> ask_question -> proposal with source note links
- Reminders scan button -> detect_reminders -> proposal list; accepted ones create
  Windows notifications via tauri-plugin-notification
- Tag suggestions button -> suggest_tags -> proposal card

#### Step 23 — Model downloader
Settings: Download Phi-3 Mini and Qwen2.5 1.5B buttons. Background download with progress
bar. User must click to start. Models in {vault}/.quillpaw/models/. Auto-populate path.

#### Step 24 — STT engine and lecture mode
stt_engine.rs: cpal audio capture -> 16kHz resample (rubato) -> silero-vad (ort) ->
whisper-rs -> emit stt-text-chunk Tauri events. LectureMode.svelte docked bar above editor.
Text appends to note at cursor. Trigger Ctrl+Shift+L. Audio saved to .assets/audio/.

COMPILE CHECK: AI loads, full proposal flow works, STT transcribes in real time.

---

### PHASE 4 — SEMANTIC SEARCH (V4)

#### Step 25 — Embedding pipeline
embeddings.rs: load nomic-embed-text-v1.5 GGUF via llama-cpp-2 embedding mode.
Batch-compute embeddings for all vault notes during CPU idle (tokio, <30% CPU throttle).
Cache keyed by note path + modification time. Store in .quillpaw/embeddings/.

#### Step 26 — HNSW vector index
usearch HNSW index over stored embeddings. Persist to .quillpaw/embeddings/hnsw.bin.
search_semantic: embed query -> nearest-neighbor -> return top-10 with cosine similarity.

#### Step 27 — Semantic search UI
Semantic tab in SearchModal.svelte. Natural language input. Results with similarity score.
Smart Search mode merging keyword and semantic results. Trigger Ctrl+Shift+S.

#### Step 28 — NPU scheduling and tag suggestions
Detect Intel Core Ultra NPU via Windows WMI. If present, prefer NPU execution provider
in llama-cpp-2. AI mode "Low (NPU)" routes all inference to NPU only.
Wire suggest_tags: use embedding similarity to find related notes, extract common tags,
propose as AiProposal.

---

## [SPEC] NON-NEGOTIABLE CONSTRAINTS

Verify every item before finishing:

  [ ] No internet calls in core code (model download only, user-triggered)
  [ ] No database for note content — plain .md files only
  [ ] AI NEVER writes to disk without user clicking Accept
  [ ] All Rust file operations async — no blocking IO
  [ ] App fully functional with AI disabled
  [ ] All user data inside user-chosen vault folder
  [ ] Telemetry OFF by default, never silently enabled
  [ ] All UI colors use CSS variables — no hardcoded hex in components
  [ ] apply_ai_proposal is the only path writing AI output to disk

---

## FINAL NOTES FOR THE AGENT

- Ask no clarifying questions. This document is complete.
- Write production-quality code throughout. No TODO placeholders in core logic.
- Comment all public Rust functions with /// doc comments.
- TypeScript strict mode on in tsconfig.json.
- Error handling: Rust returns Result<T, String>. Frontend shows warm amber toast on error.
- Compile and verify after steps 13, 19, 24, and 28. Fix all errors before moving on.
- Git commit after each numbered step if using git.
- "Quillpaw" everywhere: window title, bundle ID app.quillpaw.desktop, config folder
  .quillpaw/, all user-facing strings.

Begin with Step 1. Do not stop until Step 28 is complete.
