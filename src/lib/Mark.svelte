<script lang="ts">
  /// The app mark, drawn compact: six petals, one colour each, no filters -- the full icon
  /// is an ink-wash render, this is the version that still reads at 16px.
  let { size = 18 }: { size?: number } = $props();

  const petals = [
    { fill: "#7fe3ff", d: "M626.1 474.9C431.1 641.4 578.3 771.4 731.2 837C652 726.5 635.9 589.8 626.1 474.9Z" },
    { fill: "#4fb8ff", d: "M601.2 592.3C359.4 506.6 320.5 699.2 340.2 864.3C396.3 740.5 506.5 658.2 601.2 592.3Z" },
    { fill: "#8b6cff", d: "M487.1 629.4C440.4 377.2 254.2 439.7 121 539.3C256.2 526 382.6 580.4 487.1 629.4Z" },
    { fill: "#ff5fbf", d: "M397.9 549.1C592.9 382.6 445.7 252.6 292.8 187C372 297.5 388.1 434.2 397.9 549.1Z" },
    { fill: "#ff8a6a", d: "M422.8 431.7C664.6 517.4 703.5 324.8 683.8 159.7C627.7 283.5 517.5 365.8 422.8 431.7Z" },
    { fill: "#4fe0a8", d: "M536.9 394.6C583.6 646.8 769.8 584.3 903 484.7C767.8 498 641.4 443.6 536.9 394.6Z" },
  ];
</script>

<svg viewBox="0 0 1024 1024" width={size} height={size} aria-hidden="true">
  <circle cx="512" cy="512" r="150" fill="#bff2ff" opacity="0.85" />
  {#each petals as petal (petal.fill)}
    <path d={petal.d} fill={petal.fill} />
  {/each}
</svg>
