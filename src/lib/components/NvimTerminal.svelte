<script lang="ts">
  import { onMount } from 'svelte';
  import { ptyCreate, ptyWrite, ptyResize, ptyClose, ptyCursorLine } from '$lib/api';
  import { appState } from '$lib/store.svelte';
  import { Channel } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  interface Props {
    filePath: string;
    ptyId: number;
  }

  let { filePath, ptyId }: Props = $props();
  let container: HTMLDivElement;
  let terminal: any;
  let id = $derived(`pty-${ptyId}-${filePath}`);
  // Tauri event names only allow alphanumeric, -, /, :, _
  let safeEventId = $derived(id.replace(/[^a-zA-Z0-9_\-\/:]/g, '_'));

  let cleanupFns: Array<() => void> = [];
  let resizeTimers: ReturnType<typeof setTimeout>[] = [];
  let lastW = 0;
  let lastH = 0;
  let cursorPoll: ReturnType<typeof setInterval> | null = null;

  export function blur() {
    terminal?.blur();
  }

  export function focus() {
    terminal?.focus();
  }

  function clearTimers() {
    resizeTimers.forEach((t) => clearTimeout(t));
    resizeTimers = [];
  }

  function measureChar(font: string): { width: number; height: number } {
    const span = document.createElement('span');
    span.style.font = font;
    span.style.position = 'absolute';
    span.style.visibility = 'hidden';
    span.style.whiteSpace = 'pre';
    span.textContent = 'MMMMMMMMMM';
    document.body.appendChild(span);
    const rect = span.getBoundingClientRect();
    document.body.removeChild(span);
    return { width: rect.width / 10, height: rect.height };
  }

  function doResize() {
    if (!terminal || !container) return;
    const rect = container.getBoundingClientRect();
    const w = Math.round(rect.width);
    const h = Math.round(rect.height);
    if (w === lastW && h === lastH) return;
    lastW = w;
    lastH = h;

    const { width: cw, height: ch } = measureChar('13px Geist Mono, monospace');
    const cols = Math.max(1, Math.floor(w / cw));
    const rows = Math.max(1, Math.floor(h / ch));

    terminal.resize(cols, rows);
    ptyResize(id, cols, rows).catch(() => {});
  }

  function scheduleResizes() {
    clearTimers();
    [0, 100, 250, 500].forEach((delay) => {
      resizeTimers.push(setTimeout(doResize, delay));
    });
  }

  onMount(() => {
    (async () => {
      const xterm = await import('@xterm/xterm');
      await import('@xterm/xterm/css/xterm.css');

      terminal = new xterm.Terminal({
        fontFamily: 'Geist Mono, monospace',
        fontSize: 13,
        theme: {
          background: '#000000',
          foreground: '#aaaaaa',
          cursor: '#aaaaaa',
          selectionBackground: '#2a2a2a',
          black: '#000000',
          red: '#cc4444',
          green: '#44cc44',
          yellow: '#cccc44',
          blue: '#4444cc',
          magenta: '#cc44cc',
          cyan: '#44cccc',
          white: '#aaaaaa',
          brightBlack: '#444444',
          brightRed: '#ff4444',
          brightGreen: '#44ff44',
          brightYellow: '#ffff44',
          brightBlue: '#4444ff',
          brightMagenta: '#ff44ff',
          brightCyan: '#44ffff',
          brightWhite: '#ffffff',
        },
        cursorBlink: true,
        allowProposedApi: true,
      });

      terminal.open(container);

      const onDataHandler = terminal.onData((data: string) => {
        const bytes = new TextEncoder().encode(data);
        ptyWrite(id, Array.from(bytes)).catch(() => {});
      });
      cleanupFns.push(() => onDataHandler.dispose());

      async function startPty() {
        const rect = container.getBoundingClientRect();
        const { width: cw, height: ch } = measureChar('13px Geist Mono, monospace');
        const cols = Math.max(1, Math.floor(rect.width / cw));
        const rows = Math.max(1, Math.floor(rect.height / ch));
        terminal.resize(cols, rows);

        const onPtyData = new Channel<Uint8Array>();
        onPtyData.onmessage = (chunk: number[] | Uint8Array) => {
          const bytes = chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
          terminal?.write(bytes);
        };

        await ptyCreate(id, filePath, cols, rows, onPtyData);
        terminal.focus();
      }

      await startPty();

      const eventName = `pty-exit:${safeEventId}`;
      const exitUnlisten = await listen(eventName, () => {
        terminal?.clear();
        terminal?.reset();
        ptyClose(id).catch(() => {});
        startPty();
      });
      cleanupFns.push(() => exitUnlisten());

      const containerRo = new ResizeObserver(scheduleResizes);
      containerRo.observe(container);
      cleanupFns.push(() => containerRo.disconnect());

      const bodyRo = new ResizeObserver(scheduleResizes);
      bodyRo.observe(document.body);
      cleanupFns.push(() => bodyRo.disconnect());

      window.addEventListener('resize', scheduleResizes);
      cleanupFns.push(() => window.removeEventListener('resize', scheduleResizes));

      // Catch any late layout settling
      scheduleResizes();

      // Poll nvim cursor line for preview scroll sync
      cursorPoll = setInterval(async () => {
        try {
          const line = await ptyCursorLine(id);
          if (line !== appState.cursorLine) {
            appState.cursorLine = line;
            appState.cursorLineActive = true;
          }
        } catch {
          // PTY may not be ready yet
        }
      }, 150);
    })();

    return () => {
      clearTimers();
      if (cursorPoll) clearInterval(cursorPoll);
      cleanupFns.forEach((fn) => fn());
      cleanupFns = [];
      terminal?.dispose();
      ptyClose(id).catch(() => {});
    };
  });
</script>

<div class="terminal-container" bind:this={container}></div>

<style>
  .terminal-container {
    box-sizing: border-box;
    height: 100%;
    width: 100%;
    background: #000000;
    padding: 4px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .terminal-container :global(.xterm) {
    flex: 1;
    min-height: 0;
    min-width: 0;
  }
</style>
