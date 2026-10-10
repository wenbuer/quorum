<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Mark from "./Mark.svelte";
  import { plans, todayKey, toggleDone, type Route } from "./store.svelte";

  let { edge = false, onopen }: { edge?: boolean; onopen: (route?: Route) => void } = $props();

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
  const badge = $derived(
    !next ? (today.length > 0 ? "全部完成" : "暂无安排") : pending.length + " 项待办",
  );

  // The window is sized by the kernel, so it has to be told how many rows to make room for.
  $effect(() => {
    invoke("capsule_rows", { rows: today.length }).catch(() => {});
  });

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

  const nowLabel = $derived(
    next ? (stage === "before" ? countdown(left) : stage === "running" ? "进行中" : "已错过") : "",
  );

  // Only the title strip drags. A memo list that moved whenever you aimed at a checkbox
  // would be unusable.
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
    <Mark size={14} />
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
  <div class="list">
    <header class="bar" onmousedown={onDown} oncontextmenu={onMenu}>
      {@render mark()}
      <span class="strong">今天</span>
      <span class="badge">{badge}</span>
      <span class="grow"></span>
      {#if nowLabel}<span class="num now">{nowLabel}</span>{/if}
    </header>

    <div class="rows">
      {#each today as plan (plan.id)}
        <div class="row" class:done={plan.done}>
          <button class="tick" class:on={plan.done} onclick={() => toggleDone(plan.id)} aria-label="完成">
            {#if plan.done}
              <svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3.5 8.4l3 3 6-6.6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            {/if}
          </button>
          <span class="clock num">{hhmm(plan.start)}</span>
          <button class="task" onclick={() => onopen({ name: "plan", id: plan.id })}>{plan.title}</button>
          <span class="lvl {plan.level.toLowerCase()} num">{plan.level}</span>
        </div>
      {/each}
      {#if today.length === 0}<p class="empty">今天还没有安排</p>{/if}
    </div>
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
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 0;
    flex: 1;
    min-width: 0;
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

  .list {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: var(--capsule);
    border-radius: inherit;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 13px 0 11px;
    border-bottom: 1px solid var(--line);
    cursor: grab;
    flex: none;
  }

  .bar:active {
    cursor: grabbing;
  }

  .mark {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
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

  .strong {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }

  .badge {
    font-size: 10px;
    color: var(--text-3);
  }

  .grow {
    flex: 1;
  }

  .now {
    font-size: 13px;
    font-weight: 600;
    color: var(--brand);
  }

  .rows {
    flex: 1;
    min-height: 0;
    padding: 2px 6px 6px;
    overflow-y: auto;
  }

  /* Only a long day scrolls, and then it is a hairline, not a browser bar. */
  .rows::-webkit-scrollbar {
    width: 6px;
  }

  .rows::-webkit-scrollbar-thumb {
    border-radius: 3px;
    background: rgba(31, 35, 41, 0.18);
  }

  .rows::-webkit-scrollbar-track {
    background: transparent;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 6px;
    border-radius: var(--r-btn);
  }

  .row:hover {
    background: rgba(255, 255, 255, 0.75);
  }

  .tick {
    display: grid;
    place-items: center;
    width: 15px;
    height: 15px;
    padding: 0;
    border: 1px solid var(--line-strong);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.7);
    color: var(--green);
    cursor: pointer;
    flex: none;
  }

  .tick.on {
    border-color: var(--green);
    background: var(--green-dim);
  }

  .clock {
    font-size: 10.5px;
    color: var(--text-3);
    flex: none;
  }

  .task {
    flex: 1;
    min-width: 0;
    padding: 0;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 12px;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }

  .task:hover {
    color: var(--brand);
  }

  .row.done .task {
    color: var(--text-4);
    text-decoration: line-through;
  }

  .lvl {
    padding: 1px 5px;
    border-radius: var(--r-tag);
    font-size: 9px;
    flex: none;
  }

  .lvl.l1 { color: var(--text-3); background: var(--pressed); }
  .lvl.l2 { color: var(--brand); background: var(--brand-dim); }
  .lvl.l3 { color: var(--orange); background: var(--orange-dim); }

  .empty {
    margin: 16px 0 0;
    text-align: center;
    font-size: 11px;
    color: var(--text-4);
  }
</style>
