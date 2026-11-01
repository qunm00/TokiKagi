import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/svelte";
import { afterEach } from "vitest";

afterEach(() => {
  cleanup();
});

// jsdom lacks the Web Animations API used by Svelte transitions.
if (!Element.prototype.animate) {
  Element.prototype.animate = () =>
    ({ cancel() {}, finish() {}, onfinish: null, currentTime: 0 }) as unknown as Animation;
}
