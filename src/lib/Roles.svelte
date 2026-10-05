<script lang="ts">
  import RoleEditor from "./RoleEditor.svelte";
  import { roles, ui } from "./store.svelte";

  function close() {
    ui.drawer = null;
  }
</script>

<div class="page">
  <div class="list">
    {#each roles as role (role.id)}
      <button class="role" onclick={() => (ui.drawer = { kind: "role", id: role.id })}>
        <span class="avatar" aria-hidden="true">
          <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2.5" y="3.5" width="11" height="9" rx="2" />
            <path d="M5.5 7h5M5.5 9.5h3" stroke-linecap="round" />
          </svg>
        </span>
        <span class="rname">{role.name}</span>
        <span class="rnote num">{role.grants.filter((g) => g.on).length} / {role.grants.length} 项权限</span>
        <svg class="chev" viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.7">
          <path d="M6.5 3.5L11 8l-4.5 4.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    {/each}
  </div>

  <button class="create" onclick={() => (ui.drawer = { kind: "role" })}>
    <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.8">
      <path d="M8 3.4v9.2M3.4 8h9.2" stroke-linecap="round" />
    </svg>
    新建角色
  </button>

  {#if ui.drawer}
    <div class="scrim" role="presentation" onclick={close}></div>
    <aside class="drawer">
      <header>
        <span class="title">{ui.drawer.id ? "编辑角色" : "新建角色"}</span>
        <button class="close" onclick={close} aria-label="关闭">
          <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6">
            <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <RoleEditor id={ui.drawer.id} name={ui.drawer.name} ondone={close} />
    </aside>
  {/if}
</div>

<style>
  .page {
    position: relative;
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

  .role {
    display: grid;
    grid-template-columns: 30px 1fr auto 14px;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 12px;
    margin-bottom: 6px;
    border: 1px solid var(--line);
    border-radius: var(--r-card);
    background: var(--bg);
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .role:hover {
    border-color: var(--line-strong);
    background: var(--sunken);
  }

  .avatar {
    grid-column: 1;
    grid-row: 1 / 3;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    color: var(--brand);
    background: var(--brand-dim);
  }

  .rname {
    grid-column: 2;
    grid-row: 1;
    align-self: end;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }

  .rnote {
    grid-column: 2 / 4;
    grid-row: 2;
    align-self: start;
    font-size: 10.5px;
    color: var(--text-3);
  }

  .chev {
    grid-column: 4;
    grid-row: 1 / 3;
    align-self: center;
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

  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(31, 35, 41, 0.18);
    animation: fade 0.16s ease;
  }

  .drawer {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 66.6667%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-left: 1px solid var(--line-strong);
    box-shadow: -14px 0 34px rgba(31, 35, 41, 0.16);
    animation: slide 0.18s ease;
  }

  .drawer header {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0 10px 0 16px;
    border-bottom: 1px solid var(--line);
    flex: none;
  }

  .drawer .title {
    flex: 1;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }

  .close {
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
  }

  .close:hover {
    background: var(--hover);
    color: var(--text);
  }

  @keyframes fade {
    from { opacity: 0; }
  }

  @keyframes slide {
    from { transform: translateX(24px); opacity: 0.4; }
  }
</style>
