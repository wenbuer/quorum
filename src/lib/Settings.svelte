<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CAPSULE_TONES, LLM_PRESETS, settings, ui } from "./store.svelte";

  // The registry is the truth, so read it back instead of trusting a UI flag.
  let autostart = $state(false);
  let recording = $state(false);
  let hotkeyNote = $state("");
  let testing = $state(false);
  let testNote = $state("");
  // The row has to answer "is it working" without being opened, and the only honest source
  // for that is the last answer the endpoint actually gave.
  const llmState = $derived(settings.llm.status === "ok" ? "已连接" : "未连接");
  const llmSummary = $derived(
    settings.llm.status === "ok"
      ? "正在用 " + (settings.llm.model || settings.llm.provider)
      : "还没连上，点这里配置；未连上时便签用本地解析",
  );

  $effect(() => {
    invoke<boolean>("autostart_enabled")
      .then((on) => (autostart = on))
      .catch(() => {});
  });

  async function onRecordKey(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      recording = false;
      hotkeyNote = "";
      return;
    }
    const code = event.code;
    if (!code || /^(Control|Alt|Shift|Meta)(Left|Right)$/.test(code)) return;

    const mods: string[] = [];
    if (event.ctrlKey) mods.push("Ctrl");
    if (event.altKey) mods.push("Alt");
    if (event.shiftKey) mods.push("Shift");
    if (event.metaKey) mods.push("Super");
    if (mods.length === 0) {
      hotkeyNote = "至少要配一个 Ctrl / Alt / Shift";
      return;
    }

    try {
      settings.hotkey = await invoke<string>("set_hotkey", { spec: mods.join("+") + "+" + code });
      recording = false;
      hotkeyNote = "";
    } catch {
      hotkeyNote = "这个组合被别的程序占着，换一个";
    }
  }

  function usePreset(id: string) {
    settings.llm.provider = id;
    const preset = LLM_PRESETS.find((item) => item.id === id);
    if (preset && id !== "custom") {
      settings.llm.baseUrl = preset.baseUrl;
      settings.llm.model = preset.model;
    }
  }

  async function testLlm() {
    testing = true;
    testNote = "正在连接…";
    const base = settings.llm.baseUrl.replace(/\/+$/, "");
    try {
      const res = await fetch(base + "/models", {
        headers: settings.llm.apiKey ? { authorization: "Bearer " + settings.llm.apiKey } : {},
      });
      settings.llm.status = res.ok ? "ok" : "failed";
      if (res.ok) testNote = "连接正常";
      else if (res.status === 401 || res.status === 403) testNote = "密钥没有被接受";
      else testNote = "服务端返回 " + res.status;
    } catch {
      settings.llm.status = "failed";
      testNote = "连不上这个地址";
    } finally {
      testing = false;
    }
  }
</script>

<div class="page">
  <div class="item">
    <div>
      <p class="t">全局热键</p>
      <p class="d">{hotkeyNote || "在任何窗口按下它，唤起面板"}</p>
    </div>
    <button
      class="hotkey"
      class:recording
      onclick={() => (recording = true)}
      onkeydown={onRecordKey}
      onblur={() => (recording = false)}
    >
      {recording ? "按下组合键…" : settings.hotkey || "未注册"}
    </button>
  </div>

  <label class="item">
    <div>
      <p class="t">显示悬浮胶囊</p>
      <p class="d">关掉后胶囊不再出现在桌面，只用热键或托盘唤出面板；再打开会放回原位</p>
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

  <button class="item row" onclick={() => (ui.llmOpen = true)}>
    <div>
      <p class="t">大模型</p>
      <p class="d">{llmSummary}</p>
    </div>
    <span class="state" class:ok={settings.llm.status === "ok"}>
      <span class="sdot" aria-hidden="true"></span>{llmState}
    </span>
    <span class="chev" aria-hidden="true">
      <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8">
        <path d="M6 3.5L10.5 8 6 12.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </span>
  </button>

  <div class="item">
    <div>
      <p class="t">数据位置</p>
      <p class="d">计划、角色、项目都保存在本机</p>
    </div>
    <span class="value">本机</span>
  </div>

  <p class="about num">智群 QUORUM 0.1.0</p>

  {#if ui.llmOpen}
    <div class="scrim" role="presentation" onclick={() => (ui.llmOpen = false)}></div>
    <aside class="drawer">
      <header>
        <span class="dtitle">大模型</span>
        <span class="state" class:ok={settings.llm.status === "ok"}>
          <span class="sdot" aria-hidden="true"></span>{llmState}
        </span>
        <button class="close" onclick={() => (ui.llmOpen = false)} aria-label="关闭">
          <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6">
            <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" stroke-linecap="round" />
          </svg>
        </button>
      </header>

      <div class="dbody">
        <label class="field">
          <span class="lb">服务商</span>
          <select value={settings.llm.provider} onchange={(e) => usePreset(e.currentTarget.value)}>
            {#each LLM_PRESETS as preset (preset.id)}
              <option value={preset.id}>{preset.name}</option>
            {/each}
          </select>
        </label>

        <label class="field">
          <span class="lb">接口地址</span>
          <input bind:value={settings.llm.baseUrl} spellcheck="false" placeholder="https://api.example.com/v1" />
        </label>

        <label class="field">
          <span class="lb">模型</span>
          <input bind:value={settings.llm.model} spellcheck="false" placeholder="模型名" />
        </label>

        <label class="field">
          <span class="lb">密钥</span>
          <input type="password" bind:value={settings.llm.apiKey} spellcheck="false" placeholder="没有就留空" />
        </label>

        <div class="actions">
          <button class="mini" onclick={testLlm} disabled={testing || !settings.llm.baseUrl}>
            {testing ? "连接中…" : "测试连接"}
          </button>
          <span class="note">{testNote}</span>
        </div>
      </div>
    </aside>
  {/if}
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

  .hotkey {
    padding: 6px 11px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
    flex: none;
  }

  .hotkey:hover {
    border-color: var(--brand);
    color: var(--brand);
  }

  .hotkey.recording {
    border-color: var(--brand);
    background: var(--brand-dim);
    color: var(--brand);
    font-weight: 600;
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

  .field {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 9px;
  }

  .lb {
    width: 52px;
    flex: none;
    font-size: 11px;
    color: var(--text-3);
  }

  .field input,
  .field select {
    flex: 1;
    min-width: 0;
    height: 30px;
    padding: 0 9px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 12px;
    outline: none;
  }

  .field input:focus,
  .field select:focus {
    border-color: var(--brand);
    box-shadow: 0 0 0 3px var(--brand-dim);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 12px 0 0 62px;
  }

  .mini {
    padding: 5px 12px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-btn);
    background: var(--bg);
    color: var(--text-2);
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }

  .mini:hover:not(:disabled) {
    border-color: var(--brand);
    color: var(--brand);
  }

  .mini:disabled {
    color: var(--text-4);
    cursor: default;
  }

  .note {
    font-size: 11px;
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

  .about {
    margin: 16px 0 0;
    font-size: 10.5px;
    color: var(--text-4);
  }

  /* The row is a button but keeps the row's look, so the list still reads as one list. */
  .item.row {
    width: 100%;
    border: 0;
    border-bottom: 1px solid var(--line);
    background: none;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .item.row:hover {
    background: var(--hover);
  }

  .chev {
    display: grid;
    place-items: center;
    color: var(--text-4);
    flex: none;
  }

  .state {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--text-3);
    flex: none;
  }

  .sdot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--orange);
  }

  .state.ok {
    color: var(--green);
  }

  .state.ok .sdot {
    background: var(--green);
  }

  /* Fixed rather than absolute: the settings list scrolls, and a drawer that scrolls away
     with it would be worse than useless. It also covers the tab bar, so there is no
     half-open state to escape from. */
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(31, 35, 41, 0.18);
    animation: fade 0.16s ease;
  }

  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 78%;
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

  .dtitle {
    flex: 1;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }

  .dbody {
    padding: 14px 16px 16px;
    overflow-y: auto;
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
    flex: none;
  }

  .close:hover {
    background: var(--hover);
    color: var(--text);
  }

  @keyframes fade {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slide {
    from { transform: translateX(16px); opacity: 0; }
    to { transform: none; opacity: 1; }
  }
</style>
