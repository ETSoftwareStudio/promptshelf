// One short message at the bottom of a window.

export class Toast {
  text = $state<string | null>(null);
  bad = $state(false);
  #timer: ReturnType<typeof setTimeout> | undefined;

  show(text: string, bad = false, ms = 2600): void {
    this.text = text;
    this.bad = bad;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.text = null), ms);
  }
}
