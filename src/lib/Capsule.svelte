<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { plans } from "./store.svelte";

  let { edge = false, onopen }: { edge?: boolean; onopen: () => void } = $props();

  const target = Date.now() + 12 * 60 * 1000 + 4000;
  let remaining = $state(target - Date.now());

  $effect(() => {
    const tick = setInterval(() => {
      remaining = Math.max(0, remaining === 0 ? 0 : target - Date.now());
    }, 500);
    return () => clearInterval(tick);
  });

  const mm = $derived(String(Math.floor(remaining / 60000)).padStart(2, "0"));
  const ss = $derived(String(Math.floor((remaining % 60000) / 1000)).padStart(2, "0"));
  const next = $derived(plans.find((p) => !p.done) ?? plans[0]);
  const start = $derived(
    next
      ? String(Math.floor(next.start)).padStart(2, "0") +
          ":" +
          String(Math.round((next.start % 1) * 60)).padStart(2, "0")
      : "",
  );

  // The whole surface drags, but a press that never moves is a click. Without this split the
  // native drag loop swallows the click and the panel can never be opened by pointer.
  let pressX = 0;
  let pressY = 0;
  let dragging = false;

  function stopTracking() {
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onUp);
  }

  function onDown(event: MouseEvent) {
    if (event.button !== 0) return;
    pressX = event.screenX;
    pressY = event.screenY;
    dragging = false;
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  async function onMove(event: MouseEvent) {
    if (dragging) return;
    if (Math.abs(event.screenX - pressX) + Math.abs(event.screenY - pressY) < 4) return;
    dragging = true;
    stopTracking();
    await invoke("set_mode", { mode: "capsule" }).catch(() => {});
    // No settle call here: startDragging resolves immediately, so it used to fire before
    // the drag even started. The kernel settles once the move events go quiet.
    await getCurrentWindow().startDragging().catch(() => {});
  }

  function onUp() {
    stopTracking();
    if (!dragging) onopen();
  }

  // The menu is native, not in-page: this window is only 44px tall.
  function onMenu(event: MouseEvent) {
    event.preventDefault();
    invoke("capsule_menu").catch(() => {});
  }
</script>

{#if edge}
  <button class="parked" onmousedown={onDown} oncontextmenu={onMenu} aria-label="打开面板">
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6">
        <circle cx="8" cy="8" r="5.6" />
        <path d="M8 5.2v3l2 1.4" stroke-linecap="round" />
      </svg>
    </span>
    <span class="body">
      <span class="num park-time">{mm}:{ss}</span>
      <span class="park-title">{next?.title ?? "暂无计划"}</span>
    </span>
  </button>
{:else}
  <div
    class="capsule"
    onmousedown={onDown}
    oncontextmenu={onMenu}
    role="button"
    tabindex="0"
    aria-label="拖动，或点击打开面板"
    onkeydown={(e) => e.key === "Enter" && onopen()}
  >
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="8" cy="8" r="5.6" />
        <path d="M8 5.2v3l2 1.4" stroke-linecap="round" />
      </svg>
    </span>
    <span class="body">
      <span class="title">{next?.title ?? "暂无计划"}</span>
      <span class="tag">{next ? next.role + " · " + start + " 开始" : "还没有计划"}</span>
    </span>
    <span class="num count">{mm}:{ss}</span>
  </div>
{/if}

<style>
  .parked {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 100%;
    padding: 0 12px 0 10px;
    border: 0;
    border-radius: inherit;
    background: var(--capsule);
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: grab;
  }

  .parked:active {
    cursor: grabbing;
  }

  .parked .mark {
    width: 24px;
    height: 24px;
  }

  .parked .body {
    gap: 0;
  }

  .park-time {
    font-size: 12.5px;
    font-weight: 700;
    line-height: 1.15;
    color: var(--brand);
  }

  .park-title {
    font-size: 10px;
    line-height: 1.2;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .capsule {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 100%;
    padding: 0 16px;
    background: var(--capsule);
    border-radius: inherit;
    cursor: grab;
  }

  .capsule:active {
    cursor: grabbing;
  }

  .mark {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    color: var(--brand);
    background: var(--brand-dim);
    flex: none;
  }

  .body {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }

  .title {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tag {
    font-size: 10px;
    color: var(--text-3);
  }

  .count {
    font-size: 15px;
    font-weight: 600;
    color: var(--brand);
    flex: none;
  }
</style>
