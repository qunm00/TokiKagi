const TOTP_PERIOD = 30;

function createTotpStore() {
  let remaining = $state<number>(TOTP_PERIOD);
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

  function tick(): void {
    const secs = TOTP_PERIOD - (Math.floor(Date.now() / 1000) % TOTP_PERIOD);
    if (secs > remaining) {
      refreshCounter++;
    }
    remaining = secs;
  }

  function reset(): void {
    stop();
    remaining = TOTP_PERIOD;
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
