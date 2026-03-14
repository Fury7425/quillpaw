# Quillpaw ??Offline Academic Note-Taking

**Quillpaw** is a Windows-first, offline desktop academic note-taking application designed for scholars, researchers, and students. 

It combines:
- The **speed and local-file philosophy** of Obsidian
- The **structural block system** of Notion
- The **drawing and handwriting integration** of Microsoft OneNote

Quillpaw is **fully functional without internet access**, privacy-first by default, and optionally enhanced by local AI that the user explicitly controls.

---

## ? Core Features

### ??Editor & Academic Blocks
- **Rich Markdown Editor:** Built on CodeMirror 6 with a custom aesthetic.
- **LaTeX Math:** Fast inline `$math$` and block `$$math$$` rendering via KaTeX.
- **Enhanced Code Blocks:** Syntax highlighting and quick-copy buttons.
- **Inline PDF Viewer:** Embed and read PDFs directly within your notes.
- **Callout Blocks:** Draw attention with stylized `> [!note]`, `> [!warning]`, etc.
- **Block Commands:** Type `/` to open a quick-insert palette for math, tables, code, and more.

### ??Local Storage & Organization
- **Your Vault, Your Rules:** Notes are saved locally as plain `.md` files. No proprietary databases.
- **File Explorer:** Drag-and-drop assets, create folders, and navigate notes quickly.
- **Embedded Drawing Canvas:** A GPU-accelerated vector canvas powered by `rust-skia`. Create and embed drawings inline using the `/drawing` command.

### ??Lightning-Fast Search
- **Keyword Search:** Instant full-text indexing and fuzzy matching using `tantivy`.
- **Semantic Search:** Natural language search powered by local `nomic-embed-text` embeddings and `usearch` HNSW indexing. Finds the *meaning* of your query, not just the exact words.

### ??Local AI Integration (Opt-in)
Quillpaw leverages `llama-cpp-2` for fast, local inference (supporting INT4 GGUF models like Phi-3 Mini and Qwen2.5 1.5B) directly on your device.
- **Summarization:** Generate concise summaries of long academic texts.
- **Contextual Q&A:** Ask questions and get answers grounded in your specific note's context.
- **Tag Suggestions:** Automatically scan notes and suggest relevant structural tags.
- **Reminder Detection:** Extract deadlines and tasks directly into actionable Windows notifications.
*Note: AI outputs are treated as "Proposals". They are never written to your disk unless you explicitly click [Accept].*

### ??? Live Lecture Transcription
- **Lecture Mode:** Real-time STT (Speech-To-Text) using `whisper-rs` and `cpal`. Transcribe lectures directly into your notes while you continue typing and editing without interruption.

### ?? Visual Design
Quillpaw features three meticulously crafted themes:
- **Dark Warm:** A candlelit scholar's den. Deep browns, amber accents, and parchment-toned text.
- **Dark Cool:** A sleek, modern coding aesthetic with cool blue accents.
- **Light Parchment:** A traditional, easy-to-read paper experience.

---

## ??? Technology Stack

- **Framework:** Tauri v2 (Rust backend + WebView2 frontend)
- **UI:** Svelte 5 + TailwindCSS v4
- **Editor:** CodeMirror 6
- **Drawing:** `rust-skia`
- **Search (Keyword):** `tantivy`
- **Search (Semantic):** `usearch` (HNSW) + `nomic-embed-text`
- **AI Inference:** `llama-cpp-2` (GGUF)
- **STT Transcription:** `whisper-rs`
- **Async Runtime:** `tokio`

---

## ?? Getting Started

### Prerequisites
- Node.js (v18+)
- Rust (latest stable)
- Build tools (C++ build tools / Windows SDK for compiling `rust-skia` and `llama.cpp`)

### Installation & Build

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/quillpaw.git
   cd quillpaw
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

4. **Build for production:**
   ```bash
   npm run tauri build
   ```

---

## ?? Privacy Guarantee

Quillpaw is entirely local.
- **No cloud syncing** (unless you manually put your vault in a cloud drive like Dropbox/OneDrive).
- **No telemetry.**
- **No API keys required.** All AI and transcription happens directly on your CPU/GPU/NPU.

---

## ??? License

This project is licensed under the MIT License.