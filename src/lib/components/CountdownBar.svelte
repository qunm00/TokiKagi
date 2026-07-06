<script lang="ts">
import { Progress } from "melt/builders";

let {
  remaining,
  total = 30,
}: {
  remaining: number;
  total?: number;
} = $props();

const progress = new Progress({
  get value() {
    return remaining;
  },
  get max() {
    return total;
  },
});

let percent = $derived(total > 0 ? (remaining / total) * 100 : 0);

let barColor = $derived(
  percent > 50
    ? "var(--color-success)"
    : percent > 20
      ? "var(--color-warning)"
      : "var(--color-danger)",
);
</script>

<div {...progress.root} class="countdown-root">
  <div class="countdown-track">
    <div
      class="countdown-bar"
      style="width: {percent}%; background: {barColor};"
    >
      <span class="sr-only">{remaining} of {total} seconds remaining</span>
    </div>
  </div>
</div>

<style>
.countdown-root {
  width: 100%;
}

.countdown-track {
  width: 100%;
  height: 4px;
  background: var(--color-bg-tertiary);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.countdown-bar {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width var(--transition-base) linear, background var(--transition-base);
  min-width: 0;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px;
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
</style>
