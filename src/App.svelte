<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Capsule from "./lib/Capsule.svelte";
  import Panel from "./lib/Panel.svelte";
  import Popup from "./lib/Popup.svelte";
  import { domainOf, sameRoute, settings, ui, type Domain, type Route } from "./lib/store.svelte";

  type Mode = "capsule" | "edge" | "panel";
  type Hist = { stack: Route[]; cursor: number };

  // Both windows run this same bundle; Tauri tells them apart by window label.
  function currentLabel() {
    try {
      return getCurrentWindow().label;
    } catch {
      return "";
    }
  }

  // Hash flags let a plain browser render one state at a time; the Tauri webview has no hash.
  const flags = new Set(
    (typeof location === "undefined" ? "" : location.hash.slice(1)).split("+").filter(Boolean),
  );
  const isPopup = currentLabel() === "popup" || flags.has("popup");
  const wantsPanel = flags.has("panel") || flags.has("roles");

  let mode = $state<Mode>(wantsPanel ? "panel" : flags.has("edge") ? "edge" : "capsule");
  let collapseTimer: ReturnType<typeof setTimeout> | undefined;

  // Each tab keeps its own history, so back never crosses into the other tab and running
  // the stack dry lands on that tab's own root.
  const initialTab: Domain = flags.has("roles") ? "role" : "plan";
  let tab = $state<Domain>(initialTab);
  let hist = $state<Record<Domain, Hist>>({
    plan: { stack: [{ name: "planner" }], cursor: 0 },
    role: { stack: [{ name: "roles" }], cursor: 0 },
    project: { stack: [{ name: "projects" }], cursor: 0 },
    settings: { stack: [{ name: "settings" }], cursor: 0 },
  });

  const stack = $derived(hist[tab].stack);
  const cursor = $derived(hist[tab].cursor);
  const route = $derived(stack[cursor]);
  const canBack = $derived(cursor > 0);
  const canForward = $derived(cursor < stack.length - 1);

  function setMode(next: Mode) {
    if (mode === next) return;
    mode = next;
    invoke("set_mode", { mode: next }).catch((e) => console.error("set_mode failed", e));
  }

  function open(next: Route) {
    const domain = domainOf(next);
    tab = domain;
    const record = hist[domain];
    if (sameRoute(record.stack[record.cursor], next)) return;
    record.stack = [...record.stack.slice(0, record.cursor + 1), next];
    record.cursor = record.stack.length - 1;
  }

  function back() {
    const record = hist[tab];
    if (record.cursor > 0) record.cursor -= 1;
  }

  function forward() {
    const record = hist[tab];
    if (record.cursor < record.stack.length - 1) record.cursor += 1;
  }

  /// Put the capsule back where it was; the kernel picks capsule or icon from its own state.
  function restore() {
    invoke("set_mode", { mode: "restore" }).catch((e) => console.error("restore failed", e));
  }

  function closeSelf() {
    try {
      getCurrentWindow().close();
    } catch {
      /* browser preview */
    }
  }

  function onEnter() {
    clearTimeout(collapseTimer);
    if (mode === "edge") setMode("capsule");
  }

  function onLeave() {
    if (mode !== "capsule" || !settings.floatingIcon) return;
    clearTimeout(collapseTimer);
    collapseTimer = setTimeout(() => setMode("edge"), 320);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    if (isPopup) closeSelf();
    else if (ui.drawer) ui.drawer = null;
    else restore();
  }

  // The icon can be switched off, and the kernel has to know -- not just the UI.
  $effect(() => {
    if (isPopup) return;
    invoke("set_floating_icon", { enabled: settings.floatingIcon }).catch(() => {});
  });

  // Settings live in a JSON file the kernel owns; the frontend owns the shape. The hotkey
  // is deliberately not read back here: the kernel already registered it, and what it
  // actually holds is the truth (a taken shortcut falls back to another one).
  let restored = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (isPopup) return;
    invoke<Record<string, unknown>>("load_settings")
      .then((saved) => {
        if (typeof saved.capsule === "string") settings.capsule = saved.capsule;
        if (typeof saved.floatingIcon === "boolean") settings.floatingIcon = saved.floatingIcon;
        if (saved.llm && typeof saved.llm === "object") Object.assign(settings.llm, saved.llm);
      })
      .catch(() => {})
      .finally(() => (restored = true));
  });

  $effect(() => {
    const snapshot = {
      hotkey: settings.hotkey,
      capsule: settings.capsule,
      floatingIcon: settings.floatingIcon,
      llm: { ...settings.llm },
    };
    // Never write before the read has landed, or the defaults would overwrite the file.
    if (isPopup || !restored) return;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => invoke("save_settings", { settings: snapshot }).catch(() => {}), 400);
  });

  $effect(() => {
    if (isPopup) return;
    // The startup event races the listener, so ask for it once as well.
    invoke<string>("active_hotkey")
      .then((held) => (settings.hotkey = held))
      .catch(() => {});

    const stop: Array<() => void> = [];
    let disposed = false;

    listen<string>("quorum://mode", (event) => {
      mode = event.payload as Mode;
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch((e) => console.error("mode listener failed", e));

    listen<string>("quorum://hotkey", (event) => {
      settings.hotkey = event.payload ?? "";
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch(() => {});

    // The tray item flips the same preference the settings page owns.
    listen<boolean>("quorum://floating", (event) => {
      settings.floatingIcon = event.payload ?? true;
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch(() => {});

    listen("quorum://focus-lost", () => {
      if (mode === "panel") restore();
    })
      .then((un) => (disposed ? un() : stop.push(un)))
      .catch((e) => console.error("focus listener failed", e));

    return () => {
      disposed = true;
      stop.forEach((un) => un());
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if isPopup}
  <Popup onclose={closeSelf} />
{:else}
  <div
    class="root"
    data-mode={mode}
    style="--capsule:{settings.capsule}"
    onmouseenter={onEnter}
    onmouseleave={onLeave}
    role="presentation"
  >
    {#if mode === "panel"}
      <Panel
        {route}
        {tab}
        {canBack}
        {canForward}
        onopen={open}
        onswitch={(next) => (tab = next)}
        {back}
        {forward}
      />
    {:else}
      <Capsule
        edge={mode === "edge"}
        onopen={(next) => {
          if (next) open(next);
          setMode("panel");
        }}
      />
    {/if}
  </div>
{/if}

<style>
  .root {
    height: 100%;
    display: flex;
  }

  /* Floating: a real capsule -- both ends round, clear of any edge.
     No box-shadow: the window is exactly the size of the pill, so a shadow can only paint
     inside the four corners -- which on a transparent window reads as a grey haze. */
  .root[data-mode="capsule"] {
    align-items: stretch;
    border-radius: var(--r-capsule);
    border: 1px solid var(--line-strong);
    background: var(--capsule);
  }

  /* Parked: the same pill, docked against the edge. A percentage radius would turn a
     148x44 box into an ellipse and spill the contents outside it. */
  .root[data-mode="edge"] {
    align-items: stretch;
    border-radius: var(--r-capsule);
    border: 1px solid var(--line-strong);
    background: var(--capsule);
  }

  /* Same reason as the capsule: the window is the panel, so its shadow can only land in
     the four corner notches. */
  .root[data-mode="panel"] {
    border-radius: var(--r-panel);
    border: 1px solid var(--line-strong);
    background: var(--bg);
    overflow: hidden;
  }
</style>
