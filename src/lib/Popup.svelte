<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  type Reminder = { id: string; dueAt: number; title: string; level: string };

  let item = $state<Reminder | null>(null);

  // The window is built and the event fires before this webview has a listener, so ask once
  // as well -- the same race the hotkey readout has.
  $effect(() => {
    invoke<Reminder | null>("current_reminder")
      .then((held) => (item = held))
      .catch(() => {});

    let stop: (() => void) | undefined;
    let disposed = false;
    listen<Reminder>("quorum://remind", (event) => {
      item = event.payload;
    })
      .then((un) => (disposed ? un() : (stop = un)))
      .catch(() => {});
    return () => {
      disposed = true;
      stop?.();
    };
  });

  const when = $derived(item ? new Date(item.dueAt).toTimeString().slice(0, 5) : "");

  /// The answer belongs to the main window, which owns the plan list and the file on disk;
  /// this only reports it and gets out of the way.
  function answer(action: "done" | "snooze") {
    if (item) invoke("answer_reminder", { id: item.id, action }).catch(() => {});
  }
</script>

<div class="card">
  <header class="bar" data-tauri-drag-region>
    <span class="tag">强提醒</span>
    <span class="grow"></span>
    <span class="when num">{when}</span>
  </header>

  {#if item}
    <p class="what">{item.title}</p>
    <p class="hint">它会一直停在这儿，直到你回应。</p>

    <div class="acts">
      <button class="btn" onclick={() => answer("snooze")}>10 分钟后再提醒</button>
      <button class="btn primary" onclick={() => answer("done")}>标记已完成</button>
    </div>
  {:else}
    <p class="what">这条计划已经不在了。</p>
  {/if}
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    padding: 12px 14px 14px;
    background: var(--bg);
    border: 1px solid var(--line-strong);
    border-radius: var(--r-panel);
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
  }

  .tag {
    padding: 2px 8px;
    border-radius: var(--r-tag);
    background: var(--orange-dim);
    color: var(--orange);
    font-size: 10px;
    font-weight: 600;
  }

  .grow {
    flex: 1;
  }

  .when {
    font-size: 11px;
    color: var(--text-3);
  }

  .what {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    line-height: 1.4;
    color: var(--text);
  }

  .hint {
    margin: 6px 0 auto;
    font-size: 11px;
    color: var(--text-3);
  }

  .acts {
    display: flex;
    gap: 8px;
    padding-top: 14px;
  }

  .btn {
    flex: 1;
    height: 32px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .btn:hover {
    background: var(--hover);
  }

  .btn.primary {
    border-color: transparent;
    background: var(--brand);
    color: #fff;
    font-weight: 600;
  }

  .btn.primary:hover {
    filter: brightness(1.06);
  }
</style>
