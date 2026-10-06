<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { plans, todayKey } from "./store.svelte";

  let { edge = false, onopen }: { edge?: boolean; onopen: () => void } = $props();

  let at = $state(Date.now());

  $effect(() => {
    const tick = setInterval(() => (at = Date.now()), 1000);
    return () => clearInterval(tick);
  });

  const today = $derived(plans.filter((p) => p.day === todayKey).sort((a, b) => a.start - b.start));
  const pending = $derived(today.filter((p) => !p.done));
  // Prefer one that has not run out yet; if the day is behind us, still talk about the first.
  const upcoming = $derived(pending.filter((p) => hourAt(p.end) > at));
  const next = $derived(upcoming[0] ?? pending[0]);
  const rest = $derived(Math.max(0, (upcoming.length || pending.length) - 1));
  const left = $derived(next ? hourAt(next.start) - at : 0);
  const stage = $derived(
    !next ? "idle" : at < hourAt(next.start) ? "before" : at <= hourAt(next.end) ? "running" : "missed",
  );

  function hourAt(hour: number) {
    const date = new Date();
    date.setHours(Math.floor(hour), Math.round((hour % 1) * 60), 0, 0);
    return date.getTime();
  }

  function hhmm(hour: number) {
    const h = String(Math.floor(hour)).padStart(2, "0");
    const m = String(Math.round((hour % 1) * 60)).padStart(2, "0");
    return h + ":" + m;
  }

  /// Hours only show up once they matter; under an hour it is a plain minute:second clock.
  function countdown(ms: number) {
    const total = Math.max(0, Math.floor(ms / 1000));
    const h = Math.floor(total / 3600);
    const m = String(Math.floor((total % 3600) / 60)).padStart(2, "0");
    const s = String(total % 60).padStart(2, "0");
    return h > 0 ? h + ":" + m + ":" + s : m + ":" + s;
  }

  const idleTitle = $derived(today.length > 0 ? "今天的安排都做完了" : "今天没有安排");
  const idleNote = $derived(today.length > 0 ? "共 " + today.length + " 项" : "写下今天的打算");
  const todayLabel = $derived(
    new Date().getMonth() + 1 + "/" + String(new Date().getDate()).padStart(2, "0"),
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

{#snippet mark()}
  <span class="mark" class:dim={!next} class:ok={!next && today.length > 0} aria-hidden="true">
    {#if !next && today.length > 0}
      <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M3.5 8.4l3 3 6-6.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    {:else}
      <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6">
        <circle cx="8" cy="8" r="5.6" />
        <path d="M8 5.2v3l2 1.4" stroke-linecap="round" />
      </svg>
    {/if}
  </span>
{/snippet}

{#if edge}
  <button class="parked" onmousedown={onDown} oncontextmenu={onMenu} aria-label="打开面板">
    {@render mark()}
    <span class="body">
      {#if next}
        <span class="num park-time">{stage === "before" ? countdown(left) : stage === "running" ? "进行中" : "已错过"}</span>
        <span class="park-title">{next.title}{rest ? " +" + rest : ""}</span>
      {:else}
        <span class="num park-time">{today.length > 0 ? "完成" : "待命"}</span>
        <span class="park-title">{today.length > 0 ? "共 " + today.length + " 项" : "今天没有安排"}</span>
      {/if}
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
    {@render mark()}
    <span class="body">
      <span class="title">{next ? next.title : idleTitle}</span>
      <span class="tag"
        >{next
          ? next.role + " · " + hhmm(next.start) + " 开始" + (rest ? " · 还有 " + rest + " 项" : "")
          : idleNote}</span
      >
    </span>
    {#if next}
      <span class="num count">{stage === "before" ? countdown(left) : stage === "running" ? "进行中" : "已错过"}</span>
    {:else}
      <span class="num date">{todayLabel}</span>
    {/if}
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

  /* Nothing scheduled: the mark steps back instead of shouting. */
  .mark.dim {
    color: var(--text-3);
    background: var(--pressed);
  }

  .mark.ok {
    color: var(--green);
    background: var(--green-dim);
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
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    font-size: 15px;
    font-weight: 600;
    color: var(--brand);
    flex: none;
  }

  .date {
    font-size: 12px;
    color: var(--text-4);
    flex: none;
  }
</style>
