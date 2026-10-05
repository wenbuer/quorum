<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Folder from "./Folder.svelte";

  let { path, depth = 0 }: { path: string; depth?: number } = $props();

  type Entry = { name: string; path: string; is_dir: boolean; size: number };

  let entries = $state<Entry[] | null>(null);
  let failed = $state(false);
  let openPaths = $state<string[]>([]);

  $effect(() => {
    const target = path;
    let alive = true;
    entries = null;
    failed = false;
    invoke<Entry[]>("list_dir", { path: target })
      .then((list) => alive && (entries = list))
      .catch(() => {
        if (!alive) return;
        entries = [];
        failed = true;
      });
    return () => {
      alive = false;
    };
  });

  function toggle(child: string) {
    openPaths = openPaths.includes(child)
      ? openPaths.filter((p) => p !== child)
      : [...openPaths, child];
  }
</script>

{#if entries === null}
  <p class="hint" style="padding-left:{depth * 14 + 8}px">读取中…</p>
{:else if failed}
  <p class="hint" style="padding-left:{depth * 14 + 8}px">无法读取这个位置</p>
{:else if entries.length === 0}
  <p class="hint" style="padding-left:{depth * 14 + 8}px">空文件夹</p>
{:else}
  {#each entries as entry (entry.path)}
    <button
      class="entry"
      class:folder={entry.is_dir}
      style="padding-left:{depth * 14 + 8}px"
      onclick={() => entry.is_dir && toggle(entry.path)}
    >
      <span class="ico" aria-hidden="true">
        {#if entry.is_dir}
          <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2.5 5.5a1.5 1.5 0 011.5-1.5h2l1.2 1.4H12a1.5 1.5 0 011.5 1.5v4.6A1.5 1.5 0 0112 13H4a1.5 1.5 0 01-1.5-1.5z" />
          </svg>
        {:else}
          <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4">
            <path d="M4 2.5h5l3 3v8H4z" stroke-linejoin="round" />
          </svg>
        {/if}
      </span>
      <span class="name">{entry.name}</span>
      {#if !entry.is_dir && entry.size > 0}
        <span class="size num">{entry.size < 1024 ? entry.size + " B" : Math.round(entry.size / 1024) + " KB"}</span>
      {/if}
    </button>
    {#if entry.is_dir && openPaths.includes(entry.path)}
      <Folder path={entry.path} depth={depth + 1} />
    {/if}
  {/each}
{/if}

<style>
  .entry {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 100%;
    height: 26px;
    padding-right: 8px;
    border: 0;
    background: none;
    color: var(--text-2);
    font: inherit;
    font-size: 11.5px;
    text-align: left;
    cursor: default;
  }

  .entry.folder {
    cursor: pointer;
    color: var(--text);
  }

  .entry.folder:hover {
    background: var(--sunken);
  }

  .ico {
    display: grid;
    place-items: center;
    color: var(--text-4);
    flex: none;
  }

  .entry.folder .ico {
    color: var(--brand);
  }

  .name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .size {
    font-size: 10px;
    color: var(--text-4);
    flex: none;
  }

  .hint {
    margin: 4px 0;
    font-size: 11px;
    color: var(--text-4);
  }
</style>
