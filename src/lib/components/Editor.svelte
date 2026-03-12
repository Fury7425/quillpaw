<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { EditorState, RangeSetBuilder } from '@codemirror/state';
  import {
    EditorView,
    Decoration,
    DecorationSet,
    ViewPlugin,
    WidgetType,
    lineNumbers,
    highlightActiveLine,
    highlightActiveLineGutter,
    drawSelection,
    keymap
  } from '@codemirror/view';
  import { markdown } from '@codemirror/lang-markdown';
  import { languages } from '@codemirror/language-data';
  import { defaultKeymap } from '@codemirror/commands';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import katex from 'katex';
  import 'katex/dist/katex.min.css';
  import { getDocument, GlobalWorkerOptions } from 'pdfjs-dist';
  import pdfWorker from 'pdfjs-dist/build/pdf.worker.min.mjs?url';

  import { noteBody, activeNote, updateBody, saveActiveNote, insertRequest } from '$lib/stores/editor';
  import { vaultPath } from '$lib/stores/vault';
  import { openDrawing } from '$lib/stores/ui';
  import { tauriInvoke } from '$lib/utils/tauri_bridge';

  GlobalWorkerOptions.workerSrc = pdfWorker;

  let editorRoot: HTMLDivElement | null = null;
  let view: EditorView | null = null;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let currentVault = '';
  let commandOpen = false;
  let commandQuery = '';
  let commandX = 0;
  let commandY = 0;
  let commandLineFrom = 0;
  let commandLineTo = 0;
  let baseColor = '';
  let accentColor = '';

  const unsubscribe = vaultPath.subscribe((value) => {
    currentVault = value;
  });

  const commandList = [
    { id: 'drawing', label: 'Drawing', insert: async () => await insertDrawing() },
    { id: 'math', label: 'Math Block', insert: async () => insertBlock('$$\n\n$$') },
    { id: 'table', label: 'Table', insert: async () => insertBlock('| Col | Col |\n| --- | --- |\n|     |     |\n') },
    { id: 'code', label: 'Code Block', insert: async () => insertBlock('```\n\n```\n') },
    { id: 'pdf', label: 'PDF', insert: async () => await insertPdf() },
    { id: 'file', label: 'File', insert: async () => await insertFile() },
    { id: 'callout', label: 'Callout', insert: async () => insertBlock('> [!note] Title\n> Details\n') },
    { id: 'toc', label: 'Table of Contents', insert: async () => insertBlock('[[toc]]\n') },
    { id: 'reminder', label: 'Reminder', insert: async () => insertBlock('- [ ] Reminder: \n') }
  ];

  class AssetWidget extends WidgetType {
    constructor(private filename: string) {
      super();
    }
    toDOM(): HTMLElement {
      const wrapper = document.createElement('span');
      wrapper.className = 'inline-asset';
      const lower = this.filename.toLowerCase();
      if (lower.endsWith('.json')) {
        const canvas = document.createElement('canvas');
        canvas.className = 'drawing-preview';
        wrapper.appendChild(canvas);
        if (currentVault) {
          tauriInvoke<string>('load_drawing', { vaultPath: currentVault, filename: this.filename })
            .then((raw) => renderDrawingPreview(canvas, raw))
            .catch(() => {
              wrapper.textContent = `Drawing: ${this.filename}`;
            });
        }
        return wrapper;
      }
      if (lower.endsWith('.pdf')) {
        const container = document.createElement('div');
        container.className = 'pdf-preview';
        const canvas = document.createElement('canvas');
        const controls = document.createElement('div');
        const zoomIn = document.createElement('button');
        const zoomOut = document.createElement('button');
        zoomIn.textContent = '+';
        zoomOut.textContent = '-';
        controls.appendChild(zoomOut);
        controls.appendChild(zoomIn);
        container.appendChild(controls);
        container.appendChild(canvas);
        wrapper.appendChild(container);
        let scale = 0.9;
        if (currentVault) {
          tauriInvoke<string>('resolve_asset', { vaultPath: currentVault, filename: this.filename })
            .then((path) => {
              const src = convertFileSrc(path);
              const render = async () => {
                const pdf = await getDocument(src).promise;
                const page = await pdf.getPage(1);
                const viewport = page.getViewport({ scale });
                canvas.width = viewport.width;
                canvas.height = viewport.height;
                const context = canvas.getContext('2d');
                if (!context) return;
                await page.render({ canvasContext: context, viewport }).promise;
              };
              zoomIn.onclick = () => {
                scale = Math.min(scale + 0.2, 2);
                render();
              };
              zoomOut.onclick = () => {
                scale = Math.max(scale - 0.2, 0.6);
                render();
              };
              render();
            })
            .catch(() => {
              wrapper.textContent = `PDF: ${this.filename}`;
            });
        }
        return wrapper;
      }
      const img = document.createElement('img');
      img.alt = this.filename;
      img.loading = 'lazy';
      wrapper.appendChild(img);
      if (currentVault) {
        tauriInvoke<string>('resolve_asset', { vaultPath: currentVault, filename: this.filename })
          .then((path) => {
            img.src = convertFileSrc(path);
          })
          .catch(() => {
            wrapper.textContent = `Missing asset: ${this.filename}`;
          });
      }
      return wrapper;
    }
  }

  class MathWidget extends WidgetType {
    constructor(private expression: string, private display: boolean) {
      super();
    }
    toDOM(): HTMLElement {
      const container = document.createElement(this.display ? 'div' : 'span');
      container.className = this.display ? 'math-block' : 'math-inline';
      try {
        katex.render(this.expression, container, {
          displayMode: this.display,
          throwOnError: false
        });
      } catch {
        container.textContent = this.expression;
      }
      return container;
    }
  }

  class CodeBlockHeader extends WidgetType {
    constructor(private lang: string, private code: string) {
      super();
    }
    toDOM(): HTMLElement {
      const header = document.createElement('div');
      header.className = 'code-header';
      const label = document.createElement('span');
      label.textContent = this.lang || 'code';
      const button = document.createElement('button');
      button.textContent = 'Copy';
      button.onclick = () => navigator.clipboard.writeText(this.code);
      header.appendChild(label);
      header.appendChild(button);
      return header;
    }
  }

  const wikiLinkPlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = this.build(view);
      }
      update(update: { docChanged: boolean; viewportChanged: boolean; view: EditorView }) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = this.build(update.view);
        }
      }
      build(view: EditorView): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>();
        for (const range of view.visibleRanges) {
          const text = view.state.doc.sliceString(range.from, range.to);
          const regex = /!?\[\[[^\]]+\]\]/g;
          let match: RegExpExecArray | null;
          while ((match = regex.exec(text))) {
            const start = range.from + match.index;
            const end = start + match[0].length;
            const token = match[0];
            builder.add(start, end, Decoration.mark({ class: 'wiki-link' }));
            if (token.startsWith('!')) {
              const filename = token.replace('![[', '').replace(']]', '');
              builder.add(
                end,
                end,
                Decoration.widget({ widget: new AssetWidget(filename), side: 1 })
              );
            }
          }
        }
        return builder.finish();
      }
    },
    { decorations: (v) => v.decorations }
  );

  const mathPlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = this.build(view);
      }
      update(update: { docChanged: boolean; viewportChanged: boolean; view: EditorView }) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = this.build(update.view);
        }
      }
      build(view: EditorView): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>();
        const codeBlocks = collectCodeBlocks(view.state.doc);
        for (const range of view.visibleRanges) {
          const text = view.state.doc.sliceString(range.from, range.to);
          const blockRegex = /\$\$([\s\S]+?)\$\$/g;
          const inlineRegex = /\$([^$\n]+?)\$/g;
          let match: RegExpExecArray | null;
          while ((match = blockRegex.exec(text))) {
            const start = range.from + match.index;
            const end = start + match[0].length;
            if (isInCodeBlock(start, codeBlocks)) continue;
            builder.add(end, end, Decoration.widget({ widget: new MathWidget(match[1], true) }));
          }
          while ((match = inlineRegex.exec(text))) {
            const start = range.from + match.index;
            const end = start + match[0].length;
            if (isInCodeBlock(start, codeBlocks)) continue;
            builder.add(end, end, Decoration.widget({ widget: new MathWidget(match[1], false) }));
          }
        }
        return builder.finish();
      }
    },
    { decorations: (v) => v.decorations }
  );

  const codeBlockPlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = this.build(view);
      }
      update(update: { docChanged: boolean; viewportChanged: boolean; view: EditorView }) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = this.build(update.view);
        }
      }
      build(view: EditorView): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>();
        const doc = view.state.doc;
        let lineNumber = 1;
        while (lineNumber <= doc.lines) {
          const line = doc.line(lineNumber);
          const trimmed = line.text.trim();
          if (trimmed.startsWith('```')) {
            const lang = trimmed.replace('```', '').trim();
            const { code, endLine } = readCodeBlock(doc, lineNumber);
            builder.add(
              line.from,
              line.from,
              Decoration.widget({ widget: new CodeBlockHeader(lang, code), side: -1 })
            );
            lineNumber = endLine + 1;
            continue;
          }
          lineNumber += 1;
        }
        return builder.finish();
      }
    },
    { decorations: (v) => v.decorations }
  );

  const calloutPlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = this.build(view);
      }
      update(update: { docChanged: boolean; viewportChanged: boolean; view: EditorView }) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = this.build(update.view);
        }
      }
      build(view: EditorView): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>();
        const doc = view.state.doc;
        let activeType: string | null = null;
        for (let i = 1; i <= doc.lines; i++) {
          const line = doc.line(i);
          const trimmed = line.text.trim();
          if (trimmed.startsWith('> [!')) {
            const type = trimmed
              .replace('> [!', '')
              .split(']')[0]
              .trim()
              .toLowerCase();
            activeType = type;
            builder.add(line.from, line.from, Decoration.line({ class: `callout callout-${type}` }));
            continue;
          }
          if (activeType && trimmed.startsWith('>')) {
            builder.add(line.from, line.from, Decoration.line({ class: `callout callout-${activeType}` }));
          } else {
            activeType = null;
          }
        }
        return builder.finish();
      }
    },
    { decorations: (v) => v.decorations }
  );

  const editorTheme = EditorView.theme({
    '&': {
      backgroundColor: 'var(--bg-base)',
      color: 'var(--text-primary)',
      height: '100%'
    },
    '.cm-content': {
      fontFamily: 'var(--font-editor)',
      fontSize: '16px',
      lineHeight: '1.8'
    },
    '.cm-line': { padding: '0 16px' },
    '.cm-gutters': {
      backgroundColor: 'var(--bg-base)',
      color: 'var(--text-muted)',
      border: 'none'
    },
    '.cm-activeLine': { backgroundColor: 'var(--accent-glow)' },
    '.cm-selectionBackground': { backgroundColor: 'var(--accent-subtle)' },
    '.cm-cursor': { borderLeft: '2px solid var(--accent)' }
  });

  const buildState = (doc: string) =>
    EditorState.create({
      doc,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        drawSelection(),
        keymap.of(defaultKeymap),
        markdown({ codeLanguages: languages }),
        wikiLinkPlugin,
        mathPlugin,
        codeBlockPlugin,
        calloutPlugin,
        editorTheme,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const content = update.state.doc.toString();
            updateBody(content);
            if (saveTimer) clearTimeout(saveTimer);
            saveTimer = setTimeout(() => saveActiveNote(), 800);
          }
          updateCommandPalette(update.view);
        })
      ]
    });

  const applyInsert = (text: string) => {
    if (!view) return;
    const selection = view.state.selection.main;
    view.dispatch({
      changes: { from: selection.from, to: selection.to, insert: text },
      selection: { anchor: selection.from + text.length }
    });
    view.focus();
  };

  const insertBlock = (text: string) => {
    if (!view) return;
    view.dispatch({
      changes: { from: commandLineFrom, to: commandLineTo, insert: text }
    });
    commandOpen = false;
    view.focus();
  };

  const insertDrawing = async () => {
    const vault = currentVault;
    if (!vault) return;
    const filename = `drawing-${Date.now()}.json`;
    await tauriInvoke('save_drawing', {
      vaultPath: vault,
      filename,
      drawingJson: JSON.stringify({
        version: 2,
        canvas: { width: 1920, height: 1080, background: baseColor },
        strokes: [],
        shapes: [],
        text_layers: []
      })
    });
    insertBlock(`![[${filename}]]`);
    openDrawing(filename);
  };

  const insertFile = async () => {
    const vault = currentVault;
    if (!vault) return;
    const selected = await open({ multiple: false });
    if (!selected || Array.isArray(selected)) return;
    const stored = await tauriInvoke<string>('import_asset', {
      vaultPath: vault,
      sourcePath: selected
    });
    const filename = stored.split(/[/\\]/).pop() ?? 'file';
    insertBlock(`![[${filename}]]`);
  };

  const insertPdf = async () => {
    const vault = currentVault;
    if (!vault) return;
    const selected = await open({ multiple: false, filters: [{ name: 'PDF', extensions: ['pdf'] }] });
    if (!selected || Array.isArray(selected)) return;
    const stored = await tauriInvoke<string>('import_asset', {
      vaultPath: vault,
      sourcePath: selected
    });
    const filename = stored.split(/[/\\]/).pop() ?? 'document.pdf';
    insertBlock(`![[${filename}]]`);
  };

  const handleDrop = async (event: DragEvent) => {
    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;
    const file = files[0] as File & { path?: string };
    const sourcePath = file.path;
    if (!sourcePath || !currentVault) return;
    try {
      const storedPath = await tauriInvoke<string>('import_asset', {
        vaultPath: currentVault,
        sourcePath
      });
      const parts = storedPath.split(/[/\\]/);
      const filename = parts[parts.length - 1] || file.name;
      applyInsert(`![[${filename}]]`);
    } catch {
      // ignore import errors, toasts handled by bridge callers
    }
  };

  const updateCommandPalette = (view: EditorView) => {
    const selection = view.state.selection.main;
    const line = view.state.doc.lineAt(selection.head);
    if (line.text.startsWith('/')) {
      commandOpen = true;
      commandQuery = line.text.slice(1).trim();
      commandLineFrom = line.from;
      commandLineTo = line.to;
      const coords = view.coordsAtPos(selection.head);
      if (coords) {
        commandX = coords.left;
        commandY = coords.bottom + 6;
      }
    } else {
      commandOpen = false;
    }
  };

  const filteredCommands = () =>
    commandList.filter((command) =>
      command.label.toLowerCase().includes(commandQuery.toLowerCase())
    );

  const runCommand = async (command: { insert: () => Promise<void> }) => {
    await command.insert();
    commandOpen = false;
  };

  onMount(() => {
    const style = getComputedStyle(document.documentElement);
    baseColor = style.getPropertyValue('--bg-base').trim();
    accentColor = style.getPropertyValue('--accent').trim();
    if (!editorRoot) return;
    view = new EditorView({
      state: buildState($noteBody),
      parent: editorRoot
    });
  });

  onDestroy(() => {
    unsubscribe();
    if (view) view.destroy();
  });

  $: if (view && $noteBody !== view.state.doc.toString()) {
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: $noteBody }
    });
  }

  $: if ($insertRequest) {
    applyInsert($insertRequest);
  }

  const collectCodeBlocks = (doc: EditorState['doc']) => {
    const ranges: Array<{ from: number; to: number }> = [];
    let inBlock = false;
    let start = 0;
    for (let i = 1; i <= doc.lines; i++) {
      const line = doc.line(i);
      if (line.text.trim().startsWith('```')) {
        if (!inBlock) {
          inBlock = true;
          start = line.from;
        } else {
          inBlock = false;
          ranges.push({ from: start, to: line.to });
        }
      }
    }
    return ranges;
  };

  const isInCodeBlock = (pos: number, blocks: Array<{ from: number; to: number }>) =>
    blocks.some((block) => pos >= block.from && pos <= block.to);

  const readCodeBlock = (doc: EditorState['doc'], startLine: number) => {
    let code = '';
    let endLine = startLine;
    for (let i = startLine + 1; i <= doc.lines; i++) {
      const line = doc.line(i);
      if (line.text.trim().startsWith('```')) {
        endLine = i;
        break;
      }
      code += `${line.text}\n`;
    }
    return { code, endLine };
  };

  const renderDrawingPreview = (canvas: HTMLCanvasElement, raw: string) => {
    try {
      const data = JSON.parse(raw);
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      canvas.width = 180;
      canvas.height = 100;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const scaleX = canvas.width / (data.canvas?.width || 1920);
      const scaleY = canvas.height / (data.canvas?.height || 1080);
      (data.strokes || []).forEach((stroke: { color: string; width: number; points: StrokePoint[] }) => {
        ctx.strokeStyle = stroke.color || accentColor || baseColor;
        ctx.lineWidth = Math.max(1, stroke.width * 0.5);
        ctx.lineCap = 'round';
        for (let i = 1; i < stroke.points.length; i++) {
          const [x1, y1] = stroke.points[i - 1];
          const [x2, y2] = stroke.points[i];
          ctx.beginPath();
          ctx.moveTo(x1 * scaleX, y1 * scaleY);
          ctx.lineTo(x2 * scaleX, y2 * scaleY);
          ctx.stroke();
        }
      });
    } catch {
      // ignore parse errors
    }
  };
</script>

<div class="editor-shell">
  {#if $activeNote}
    <div class="meta">
      <span class="title">{$activeNote.title}</span>
      <span class="date">{$activeNote.modified}</span>
    </div>
  {/if}
  {#if !$activeNote}
    <div class="empty">Open a note to start writing.</div>
  {/if}
  <div
    class="editor"
    bind:this={editorRoot}
    on:dragover|preventDefault
    on:drop|preventDefault={handleDrop}
  />
  {#if commandOpen}
    <div class="command-palette" style={`top:${commandY}px; left:${commandX}px;`}>
      {#each filteredCommands() as command}
        <button on:click={() => runCommand(command)}>{command.label}</button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .editor-shell {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--bg-base);
    position: relative;
  }
  .meta {
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    justify-content: space-between;
    color: var(--text-secondary);
  }
  .editor {
    flex: 1;
  }
  .empty {
    padding: var(--space-4);
    color: var(--text-muted);
  }
  .command-palette {
    position: fixed;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-strong);
    padding: var(--space-2);
    display: grid;
    gap: var(--space-1);
    z-index: 30;
  }
  .command-palette button {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    text-align: left;
    padding: var(--space-2);
  }
  :global(.wiki-link) {
    color: var(--accent-bright);
  }
  :global(.inline-asset) {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: var(--space-2);
    color: var(--text-secondary);
    font-size: 12px;
  }
  :global(.inline-asset img) {
    max-height: 180px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }
  :global(.drawing-preview) {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }
  :global(.pdf-preview) {
    display: grid;
    gap: var(--space-1);
  }
  :global(.pdf-preview canvas) {
    max-width: 240px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
  }
  :global(.pdf-preview button) {
    width: 24px;
    height: 24px;
  }
  :global(.code-header) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    margin: 4px 0;
    color: var(--text-secondary);
  }
  :global(.code-header button) {
    background: transparent;
    border: 1px solid var(--border-subtle);
    padding: 2px 6px;
  }
  :global(.math-block) {
    margin: 8px 0;
  }
  :global(.callout) {
    border-left: 3px solid var(--accent);
    background: var(--accent-subtle);
    padding: 4px 8px;
  }
  :global(.callout-warning) {
    border-left-color: var(--warning);
    background: var(--accent-glow);
  }
  :global(.callout-tip) {
    border-left-color: var(--success);
    background: var(--accent-glow);
  }
  :global(.callout-important) {
    border-left-color: var(--danger);
    background: var(--accent-glow);
  }
</style>
