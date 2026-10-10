<script lang="ts">
  import { untrack } from "svelte";
  import ConfirmButton from "./ConfirmButton.svelte";
  import Folder from "./Folder.svelte";
  import { addProject, projects, removeProject, ui, updateProject } from "./store.svelte";

  let { id, ondone }: { id?: string; ondone: () => void } = $props();

  const existing = $derived(id ? projects.find((p) => p.id === id) : undefined);

  let name = $state("");
  let note = $state("");
  let paths = $state<string[]>([]);
  let draftPath = $state("");
  let browsing = $state<string[]>([]);

  // A command hands its arguments to this form once; leaving the form unconsumed would
  // refill it the next time the page is opened from the list.
  $effect(() => {
    const source = existing;
    const seeded = source ? null : untrack(() => ui.projectDraft);
    name = source?.name ?? seeded?.name ?? "";
    note = source?.note ?? "";
    paths = source ? [...source.paths] : seeded?.path ? [seeded.path] : [];
    if (seeded) ui.projectDraft = null;
  });

  function browseToggle(path: string) {
    browsing = browsing.includes(path)
      ? browsing.filter((p) => p !== path)
      : [...browsing, path];
  }

  function addPath() {
    const value = draftPath.trim();
    if (!value) return;
    paths = [...paths, value];
    draftPath = "";
  }

  function save() {
    const trimmed = name.trim();
    if (!trimmed) return;
    if (existing) updateProject(existing.id, trimmed, note.trim(), paths);
    else addProject(trimmed, note.trim(), paths);
    ondone();
  }
</script>

<div class="page">
  <label class="field">
    <span class="lb">名称</span>
    <input bind:value={name} spellcheck="false" placeholder="例如：毕业论文" />
  </label>

  <label class="field">
    <span class="lb">说明</span>
    <input bind:value={note} spellcheck="false" placeholder="这个项目是做什么的" />
  </label>

  <div class="field">
    <span class="lb">路径</span>
    <div class="paths">
      {#each paths as path, i (path + i)}
        <span class="path">
          <span class="num">{path}</span>
          <button class="peek" onclick={() => browseToggle(path)}>查看文件夹</button>
          <button class="unpick" aria-label="移除" onclick={() => (paths = paths.filter((_, at) => at !== i))}>
            <svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
            </svg>
          </button>
        </span>
        {#if browsing.includes(path)}
          <div class="tree"><Folder path={path} /></div>
        {/if}
      {/each}
      {#if paths.length === 0}<p class="hint">还没有添加路径</p>{/if}
    </div>
    <div class="addpath">
      <input
        bind:value={draftPath}
        spellcheck="false"
        placeholder="粘贴一个文件夹路径"
        onkeydown={(e) => e.key === "Enter" && addPath()}
      />
      <button class="btn small" onclick={addPath} disabled={!draftPath.trim()}>添加</button>
    </div>
  </div>

  <div class="foot">
    {#if existing}
      <ConfirmButton
        label="删除项目"
        onconfirm={() => {
          removeProject(existing.id);
          ondone();
        }}
      />
    {/if}
    <span class="grow"></span>
    <button class="btn" onclick={ondone}>取消</button>
    <button class="btn primary" onclick={save} disabled={!name.trim()}>
      {existing ? "保存" : "创建项目"}
    </button>
  </div>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px 20px 14px;
    overflow-y: auto;
  }

  .field {
    display: block;
    margin-bottom: 14px;
  }

  .lb {
    display: block;
    margin-bottom: 6px;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--text-3);
  }

  .field input {
    width: 100%;
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    outline: none;
  }

  .field input:focus {
    border-color: var(--brand);
    box-shadow: 0 0 0 3px var(--brand-dim);
  }

  .paths {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-bottom: 6px;
  }

  .path {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 6px 5px 10px;
    border: 1px solid var(--line);
    border-radius: var(--r-btn);
    background: var(--sunken);
    font-size: 11px;
    color: var(--text-2);
  }

  .path .num {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .peek {
    padding: 2px 8px;
    border: 1px solid var(--line-strong);
    border-radius: 5px;
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 10.5px;
    cursor: pointer;
    flex: none;
  }

  .peek:hover {
    border-color: var(--brand);
    color: var(--brand);
  }

  .tree {
    max-height: 220px;
    margin-bottom: 6px;
    padding: 4px 0;
    border: 1px solid var(--line);
    border-radius: var(--r-btn);
    background: var(--bg);
    overflow: auto;
  }

  .unpick {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: none;
    color: var(--text-4);
    cursor: pointer;
    flex: none;
  }

  .unpick:hover {
    background: var(--red-dim);
    color: var(--red);
  }

  .hint {
    margin: 0;
    font-size: 11px;
    color: var(--text-4);
  }

  .addpath {
    display: flex;
    gap: 6px;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    padding-top: 14px;
  }

  .grow {
    flex: 1;
  }

  .btn {
    height: 32px;
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

  .btn.small {
    height: 32px;
    padding: 0 12px;
    font-size: 12px;
    flex: none;
  }

  .btn.primary {
    border-color: var(--brand);
    background: var(--brand);
    color: #fff;
    font-weight: 600;
  }

  .btn.primary:disabled {
    border-color: var(--pressed);
    background: var(--pressed);
    color: var(--text-4);
    cursor: default;
  }

</style>
