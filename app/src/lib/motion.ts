// Motion for the app: short, calm, and off when the system asks for less motion.
// Every transition here uses these durations and this curve, so the app moves
// as one thing.

import { cubicOut } from "svelte/easing";
import type { FlipParams } from "svelte/animate";
import type { TransitionConfig } from "svelte/transition";

export const FAST = 140;
export const BASE = 220;

/** True when the system asks for reduced motion. */
export function reduced(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** A duration, or 0 when the system asks for reduced motion. */
export function dur(ms: number): number {
  return reduced() ? 0 : ms;
}

/** For `animate:flip`: rows slide to their new place when the order changes. */
export function moveParams(): FlipParams {
  return { duration: (d) => dur(Math.min(BASE + d / 4, 360)), easing: cubicOut };
}

/** An inline item (such as the pin before a title) grows in from nothing, so
 * the text next to it slides over instead of jumping. */
export function grow(node: Element, { duration = BASE } = {}): TransitionConfig {
  const style = getComputedStyle(node);
  const width = parseFloat(style.width) || node.getBoundingClientRect().width;
  const marginRight = parseFloat(style.marginRight) || 0;
  return {
    duration: dur(duration),
    easing: cubicOut,
    css: (t) =>
      `overflow:hidden;width:${t * width}px;margin-right:${t * marginRight}px;` +
      `opacity:${t};transform:scale(${0.6 + 0.4 * t})`,
  };
}

/** A row that appears or leaves in a list: height and fade together. */
export function row(node: Element, { duration = BASE } = {}): TransitionConfig {
  const style = getComputedStyle(node);
  const height = parseFloat(style.height);
  const pt = parseFloat(style.paddingTop);
  const pb = parseFloat(style.paddingBottom);
  return {
    duration: dur(duration),
    easing: cubicOut,
    css: (t) =>
      `overflow:hidden;opacity:${t};height:${t * height}px;` +
      `padding-top:${t * pt}px;padding-bottom:${t * pb}px`,
  };
}

/** The dimmed backdrop behind a sheet. */
export function dim(_node: Element, { duration = FAST } = {}): TransitionConfig {
  return { duration: dur(duration), css: (t) => `opacity:${t}` };
}

/** A sheet drops in from just above its place. It keeps the sheet's own
 * horizontal centring (`translateX(-50%)`). */
export function sheet(_node: Element, { duration = BASE } = {}): TransitionConfig {
  return {
    duration: dur(duration),
    easing: cubicOut,
    css: (t, u) =>
      `opacity:${t};transform:translateX(-50%) translateY(${-8 * u}px) scale(${0.98 + 0.02 * t})`,
  };
}

/** A small pop-up such as a menu: fade and a slight scale from its corner. */
export function pop(_node: Element, { duration = FAST } = {}): TransitionConfig {
  return {
    duration: dur(duration),
    easing: cubicOut,
    css: (t) => `opacity:${t};transform:scale(${0.96 + 0.04 * t})`,
  };
}

/** The toast rises a little as it appears. Keeps its own centring. */
export function rise(_node: Element, { duration = BASE } = {}): TransitionConfig {
  return {
    duration: dur(duration),
    easing: cubicOut,
    css: (t, u) => `opacity:${t};transform:translateX(-50%) translateY(${6 * u}px)`,
  };
}
