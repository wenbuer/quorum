<script lang="ts">
  /// Deleting takes two clicks: the first arms it, the second does it, and walking away or
  /// waiting four seconds disarms it. A modal would be heavier than the mistake it prevents.
  let { label, onconfirm, confirm = "确认删除" }: {
    label: string;
    onconfirm: () => void;
    confirm?: string;
  } = $props();

  let armed = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function press() {
    if (!armed) {
      armed = true;
      clearTimeout(timer);
      timer = setTimeout(() => (armed = false), 4000);
      return;
    }
    clearTimeout(timer);
    armed = false;
    onconfirm();
  }

  $effect(() => () => clearTimeout(timer));
</script>

<button class="cb" class:danger={!armed} class:arm={armed} onclick={press}>
  {armed ? confirm : label}
</button>

<style>
  .cb {
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

  .cb:hover {
    background: var(--hover);
  }

  .cb.danger {
    border-color: rgba(240, 74, 74, 0.35);
    color: var(--red);
  }

  .cb.danger:hover {
    background: var(--red-dim);
  }

  .cb.arm {
    border-color: transparent;
    background: var(--red);
    color: #fff;
    font-weight: 600;
  }
</style>
