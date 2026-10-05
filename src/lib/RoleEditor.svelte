<script lang="ts">
  import { addRole, grantsTemplate, roles, updateRole, type Grant } from "./store.svelte";

  let { id, name: seeded = "", ondone }: { id?: string; name?: string; ondone: () => void } = $props();

  const existing = $derived(id ? roles.find((r) => r.id === id) : undefined);

  let name = $state("");
  let grants = $state<Grant[]>([]);

  $effect(() => {
    const source = existing;
    name = source ? source.name : seeded;
    grants = source ? source.grants.map((g) => ({ ...g })) : grantsTemplate();
  });

  const granted = $derived(grants.filter((g) => g.on).length);

  function save() {
    const trimmed = name.trim();
    if (!trimmed) return;
    if (existing) updateRole(existing.id, trimmed, grants);
    else addRole(trimmed, grants);
    ondone();
  }
</script>

<div class="editor">
  <label class="field">
    <span class="lb">名称</span>
    <input bind:value={name} spellcheck="false" placeholder="例如：周报助手" />
  </label>

  <div class="grants-head">
    <span class="lb">可以做什么</span>
    <span class="count num">{granted} / {grants.length} 已允许</span>
  </div>

  <div class="grants">
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

  <div class="foot">
    <button class="btn" onclick={ondone}>取消</button>
    <button class="btn primary" onclick={save} disabled={!name.trim()}>
      {existing ? "保存" : "创建角色"}
    </button>
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .field {
    display: block;
    padding: 14px 16px 0;
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

  .grants-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 16px 16px 4px;
  }

  .count {
    font-size: 10.5px;
    color: var(--text-3);
  }

  .grants {
    flex: 1;
    min-height: 0;
    padding: 0 16px;
    overflow-y: auto;
  }

  .grant {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 2px;
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
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px 14px;
    border-top: 1px solid var(--line);
    background: var(--sunken);
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
