<script lang="ts">
  import { settings } from "./store.svelte";
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
      <p class="d">开启后，胶囊拖到屏幕边缘会收成一枚圆形图标；关掉则始终保持完整</p>
    </div>
    <input class="switch" type="checkbox" bind:checked={settings.floatingIcon} />
  </label>

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
