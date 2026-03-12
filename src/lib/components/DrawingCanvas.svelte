<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { uiState, closeDrawing } from '$lib/stores/ui';
  import { vaultPath } from '$lib/stores/vault';
  import { tauriInvoke } from '$lib/utils/tauri_bridge';

  type StrokePoint = [number, number, number];
  type Stroke = {
    id: string;
    tool: 'pen' | 'highlighter' | 'eraser';
    color: string;
    width: number;
    opacity: number;
    points: StrokePoint[];
  };

  let canvas: HTMLCanvasElement | null = null;
  let ctx: CanvasRenderingContext2D | null = null;
  let drawing = false;
  let strokes: Stroke[] = [];
  let currentStroke: Stroke | null = null;
  let color = '';
  let width = 2.5;
  let tool: Stroke['tool'] = 'pen';
  let baseColor = '';
  const colorVars = ['--accent', '--accent-bright', '--text-primary', '--accent2', '--success'];
  let colors: string[] = [];

  const resizeCanvas = () => {
    if (!canvas) return;
    const ratio = window.devicePixelRatio || 1;
    canvas.width = canvas.clientWidth * ratio;
    canvas.height = canvas.clientHeight * ratio;
    ctx = canvas.getContext('2d');
    if (ctx) ctx.scale(ratio, ratio);
    redraw();
  };

  const startStroke = (event: PointerEvent) => {
    if (!canvas) return;
    drawing = true;
    const rect = canvas.getBoundingClientRect();
    const point: StrokePoint = [
      event.clientX - rect.left,
      event.clientY - rect.top,
      event.pressure || 0.5
    ];
    currentStroke = {
      id: crypto.randomUUID(),
      tool,
      color,
      width,
      opacity: tool === 'highlighter' ? 0.4 : 1,
      points: [point]
    };
    strokes.push(currentStroke);
  };

  const moveStroke = (event: PointerEvent) => {
    if (!drawing || !canvas || !ctx || !currentStroke) return;
    const rect = canvas.getBoundingClientRect();
    const point: StrokePoint = [
      event.clientX - rect.left,
      event.clientY - rect.top,
      event.pressure || 0.5
    ];
    currentStroke.points.push(point);
    drawStrokeSegment(currentStroke);
  };

  const endStroke = async () => {
    if (!drawing) return;
    drawing = false;
    currentStroke = null;
    await saveDrawing();
  };

  const drawStrokeSegment = (stroke: Stroke) => {
    if (!ctx) return;
    const points = stroke.points;
    if (points.length < 2) return;
    const [p1, p2] = points.slice(-2);
    ctx.globalAlpha = stroke.opacity;
    ctx.strokeStyle = stroke.tool === 'eraser' ? baseColor : stroke.color;
    ctx.lineWidth = stroke.width;
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(p1[0], p1[1]);
    ctx.lineTo(p2[0], p2[1]);
    ctx.stroke();
    ctx.globalAlpha = 1;
  };

  const redraw = () => {
    if (!ctx || !canvas) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    for (const stroke of strokes) {
      for (let i = 1; i < stroke.points.length; i++) {
        const segment: Stroke = {
          ...stroke,
          points: [stroke.points[i - 1], stroke.points[i]]
        };
        drawStrokeSegment(segment);
      }
    }
  };

  const saveDrawing = async () => {
    const vault = $vaultPath;
    if (!vault || !$uiState.drawingFile) return;
    const payload = {
      version: 2,
      canvas: { width: 1920, height: 1080, background: baseColor },
      strokes,
      shapes: [],
      text_layers: []
    };
    await tauriInvoke('save_drawing', {
      vaultPath: vault,
      filename: $uiState.drawingFile,
      drawingJson: JSON.stringify(payload)
    });
  };

  const loadDrawing = async () => {
    const vault = $vaultPath;
    if (!vault || !$uiState.drawingFile) return;
    try {
      const raw = await tauriInvoke<string>('load_drawing', {
        vaultPath: vault,
        filename: $uiState.drawingFile
      });
      const parsed = JSON.parse(raw);
      strokes = parsed.strokes ?? [];
      redraw();
    } catch {
      strokes = [];
      redraw();
    }
  };

  onMount(() => {
    const style = getComputedStyle(document.documentElement);
    colors = colorVars.map((variable) => style.getPropertyValue(variable).trim());
    baseColor = style.getPropertyValue('--bg-base').trim();
    color = colors[0] ?? baseColor;
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);
  });

  onDestroy(() => {
    window.removeEventListener('resize', resizeCanvas);
  });

  $: if ($uiState.drawingOpen) {
    loadDrawing();
  }
</script>

{#if $uiState.drawingOpen}
  <div class="overlay">
    <div class="toolbar">
      <div class="colors">
        {#each colors as swatch, index}
          <button
            type="button"
            class={`swatch ${color === swatch ? 'active' : ''}`}
            style={`background:var(${colorVars[index]})`}
            aria-label={`Select color ${index + 1}`}
            on:click={() => (color = swatch)}
          ></button>
        {/each}
      </div>
      <div class="tools">
        <button class:active={tool === 'pen'} on:click={() => (tool = 'pen')}>Pen</button>
        <button class:active={tool === 'highlighter'} on:click={() => (tool = 'highlighter')}>
          Highlighter
        </button>
        <button class:active={tool === 'eraser'} on:click={() => (tool = 'eraser')}>Eraser</button>
      </div>
      <label class="width">
        Width
        <input type="range" min="1" max="8" step="0.5" bind:value={width} />
      </label>
      <button on:click={closeDrawing}>Close</button>
    </div>
    <div class="canvas-wrap">
      <canvas
        bind:this={canvas}
        on:pointerdown={startStroke}
        on:pointermove={moveStroke}
        on:pointerup={endStroke}
        on:pointerleave={endStroke}
      ></canvas>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay);
    z-index: 45;
    display: grid;
    grid-template-rows: auto 1fr;
  }
  .toolbar {
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
  }
  .colors {
    display: flex;
    gap: var(--space-1);
  }
  .swatch {
    width: 20px;
    height: 20px;
    border: 1px solid var(--border);
    border-radius: 50%;
  }
  .swatch.active {
    border-color: var(--accent);
  }
  .tools button.active {
    border-color: var(--accent);
  }
  .width {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .canvas-wrap {
    background: var(--bg-base);
    display: grid;
    place-items: center;
  }
  canvas {
    width: 90%;
    height: 90%;
    background: var(--bg-base);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }
</style>
