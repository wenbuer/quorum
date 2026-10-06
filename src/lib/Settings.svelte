<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CAPSULE_TONES, settings } from "./store.svelte";

  // The registry is the truth, so read it back instead of trusting a UI flag.
  let autostart = $state(false);

  $effect(() => {
    invoke<boolean>("autostart_enabled")
      .then((on) => (autostart = on))
      .catch(() => {});
  });
</script>

<div class="page">
  <div class="item">
    <div>
      <p class="t">全局热键</p>
      <p class="d">在任何窗口按下它，唤起面板</p>
    </div>
    <span class="value num">{settings.hotkey || "未注册"}</span>
  </div>

  <label class="item">
    <div>
      <p class="t">开启悬浮图标</p>
      <p class="d">开启后，胶囊贴到屏幕边缘会收成一小条；关掉则始终保持完整</p>
    </div>
    <input class="switch" type="checkbox" bind:checked={settings.floatingIcon} />
  </label>

  <label class="item">
    <div>
      <p class="t">开机自启</p>
      <p class="d">开机后自动在后台待命，需要时再唤出面板</p>
    </div>
    <input
      class="switch"
      type="checkbox"
      bind:checked={autostart}
      onchange={() => invoke("set_autostart", { enabled: autostart }).catch(() => {})}
    />
  </label>

  <div class="item">
    <div>
      <p class="t">悬浮窗颜色</p>
      <p class="d">胶囊和清单的底色</p>
    </div>
    <div class="tones">
      {#each CAPSULE_TONES as tone (tone.value)}
        <button
          class="tone"
          class:on={settings.capsule === tone.value}
          style="--tone:{tone.value}"
          onclick={() => (settings.capsule = tone.value)}
          aria-label={tone.name}
        ></button>
      {/each}
      <label class="pick" aria-label="自定义颜色">
        <input type="color" bind:value={settings.capsule} />
      </label>
    </div>
  </div>

  <div class="item">
    <div>
      <p class="t">数据位置</p>
      <p class="d">计划、角色、项目都保存在本机</p>
    </div>
    <span class="value">本机</span>
  </div>

  <p class="about num">AAIL 0.1.0</p>
</div>

<style>
  .page {
    height: 100%;
    padding: 8px 20px 16px;
    overflow-y: auto;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 0;
    border-bottom: 1px solid var(--line);
  }

  .item div {
    flex: 1;
    min-width: 0;
  }

  .t {
    margin: 0 0 2px;
    font-size: 12.5px;
    color: var(--text);
  }

  .d {
    margin: 0;
    font-size: 11px;
    line-height: 1.45;
    color: var(--text-3);
  }

  .value {
    font-size: 12px;
    color: var(--text-2);
    flex: none;
  }

  .tones {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: none;
  }

  .tone {
    width: 22px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--line-strong);
    border-radius: 50%;
    background: var(--tone);
    cursor: pointer;
  }

  .tone.on {
    border-color: var(--brand);
    box-shadow: 0 0 0 2px var(--brand-dim);
  }

  /* Native picker, hidden under a rainbow chip. */
  .pick {
    position: relative;
    width: 22px;
    height: 22px;
    border: 1px solid var(--line-strong);
    border-radius: 50%;
    overflow: hidden;
    cursor: pointer;
    background: conic-gradient(#f04a4a, #f59e0b, #22b07d, #2b6cf6, #a855f7, #f04a4a);
  }

  .pick input {
    position: absolute;
    inset: -10px;
    width: 44px;
    height: 44px;
    padding: 0;
    border: 0;
    background: none;
    opacity: 0;
    cursor: pointer;
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

  .about {
    margin: 16px 0 0;
    font-size: 10.5px;
    color: var(--text-4);
  }
</style>
