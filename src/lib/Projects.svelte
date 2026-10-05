<script lang="ts">
  import { projects } from "./store.svelte";

  let { onopen, oncreate }: { onopen: (id: string) => void; oncreate: () => void } = $props();
</script>

<div class="page">
  <div class="list">
    {#each projects as project (project.id)}
      <button class="row" onclick={() => onopen(project.id)}>
        <span class="avatar" aria-hidden="true">
          <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2.5 5.5a1.5 1.5 0 011.5-1.5h2l1.2 1.4H12a1.5 1.5 0 011.5 1.5v4.6A1.5 1.5 0 0112 13H4a1.5 1.5 0 01-1.5-1.5z" />
          </svg>
        </span>
        <span class="name">{project.name}</span>
        <span class="meta num">{project.paths.length} 个路径</span>
        <svg class="chev" viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.7">
          <path d="M6.5 3.5L11 8l-4.5 4.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    {/each}
    {#if projects.length === 0}
      <p class="empty">还没有项目。</p>
    {/if}
  </div>

  <button class="create" onclick={oncreate}>
    <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.8">
      <path d="M8 3.4v9.2M3.4 8h9.2" stroke-linecap="round" />
    </svg>
    新建项目
  </button>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 12px 14px 14px;
    min-height: 0;
  }

  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .row {
    display: grid;
    grid-template-columns: 30px 1fr auto 14px;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 11px 12px;
    margin-bottom: 6px;
    border: 1px solid var(--line);
    border-radius: var(--r-card);
    background: var(--bg);
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .row:hover {
    border-color: var(--line-strong);
    background: var(--sunken);
  }

  .avatar {
    grid-row: 1 / 3;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    color: var(--brand);
    background: var(--brand-dim);
  }

  .name {
    grid-column: 2;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }

  .meta {
    grid-column: 3;
    font-size: 10.5px;
    color: var(--text-3);
  }

  .chev {
    grid-column: 4;
    grid-row: 1 / 3;
    color: var(--text-4);
  }

  .empty {
    margin: 28px 0 0;
    text-align: center;
    font-size: 11.5px;
    color: var(--text-4);
  }

  .create {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    margin-top: 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    flex: none;
  }

  .create:hover {
    border-color: var(--brand);
    color: var(--brand);
    background: var(--brand-dim);
  }
</style>
