<script lang="ts">
  import { grantsTemplate, type Grant } from "./store.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let name = $state("编排");
  let grants = $state<Grant[]>(grantsTemplate());

  const granted = $derived(grants.filter((g) => g.on).length);
</script>

<div class="popup">
  <header class="bar" data-tauri-drag-region>
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="2.5" y="3.5" width="11" height="9" rx="2" />
        <path d="M5.5 7h5M5.5 9.5h3" stroke-linecap="round" />
      </svg>
    </span>
    <input class="title" bind:value={name} spellcheck="false" />
    <button class="ghost" onclick={onclose} aria-label="关闭">
      <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6">
        <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
      </svg>
    </button>
  </header>

  <div class="body">
    <div class="grants-head">
      <span class="lb">可以做什么</span>
      <span class="count num">{granted} / {grants.length} 已允许</span>
    </div>

    {#each grants as grant (grant.id)}
      <label class="grant">
        <span class="gtext">
          <span class="gname">{grant.name}</span>
          <span class="gscope">{grant.scope}</span>
        </span>
        <input class="switch" type="checkbox" bind:checked={grant.on} />
      </label>
    {/each}
  </div>

  <footer class="foot">
    <span class="grow"></span>
    <button class="btn" onclick={onclose}>取消</button>
    <button class="btn primary" onclick={onclose}>保存</button>
  </footer>
</div>

<style>
  .popup {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background: var(--bg);
    border: 1px solid var(--line-strong);
    border-radius: var(--r-panel);
    overflow: hidden;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 9px;
    height: 46px;
    padding: 0 10px 0 14px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }

  .mark {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    color: var(--brand);
    background: var(--brand-dim);
    flex: none;
  }

  .title {
    flex: 1;
    min-width: 0;
    height: 26px;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    font-weight: 600;
    outline: none;
  }

  .ghost {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--r-btn);
    background: none;
    color: var(--text-3);
    cursor: pointer;
    flex: none;
  }

  .ghost:hover {
    background: var(--hover);
    color: var(--text);
  }

  .body {
    flex: 1;
    min-height: 0;
    padding: 14px 16px;
    overflow-y: auto;
  }

  .grants-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .lb {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-3);
  }

  .count {
    font-size: 10.5px;
    color: var(--text-3);
  }

  .grant {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 2px;
    border-bottom: 1px solid var(--line);
    cursor: pointer;
  }

  .gtext {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    min-width: 0;
  }

  .gname {
    font-size: 12px;
    color: var(--text);
  }

  .gscope {
    font-size: 10.5px;
    color: var(--text-3);
  }

  .switch {
    appearance: none;
    width: 34px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 10px;
    background: var(--line-strong);
    position: relative;
    cursor: pointer;
    flex: none;
    transition: background 0.15s ease;
  }

  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(31, 35, 41, 0.28);
    transition: transform 0.15s ease;
  }

  .switch:checked {
    background: var(--brand);
  }

  .switch:checked::after {
    transform: translateX(14px);
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 52px;
    padding: 0 14px;
    border-top: 1px solid var(--line);
    background: var(--sunken);
    flex: none;
  }

  .grow {
    flex: 1;
  }

  .btn {
    height: 30px;
    padding: 0 16px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    cursor: pointer;
  }

  .btn:hover {
    background: var(--hover);
  }

  .btn.primary {
    border-color: var(--brand);
    background: var(--brand);
    color: #fff;
    font-weight: 600;
  }
</style>
