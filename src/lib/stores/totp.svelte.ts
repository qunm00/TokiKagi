import * as actions from "$lib/actions/vault";

function createTotpStore() {
  let remaining = $state<number>(30);
  let refreshCounter = $state<number>(0);
  let _interval: ReturnType<typeof setInterval> | null = null;

  function start(): void {
    stop();
    tick();
    _interval = setInterval(tick, 1000);
  }

  function stop(): void {
    if (_interval !== null) {
      clearInterval(_interval);
      _interval = null;
    }
  }

  async function tick(): Promise<void> {
    const secs = await actions.getTimeRemaining();
    if (secs > remaining) {
      refreshCounter++;
    }
    remaining = secs;
  }

  function reset(): void {
    stop();
    remaining = 30;
    refreshCounter = 0;
  }

  return {
    get remaining() {
      return remaining;
    },
    get refreshCounter() {
      return refreshCounter;
    },
    start,
    stop,
    reset,
  };
}

export const totp = createTotpStore();
