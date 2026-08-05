import { type Item, createEmptyItem } from "./Item";

export class Hotbar {
  private items = Array.from({ length: 9 }, createEmptyItem);
  private slot = 0;

  constructor() {
    for (let i = 0; i < this.items.length; i++) {
      this.items[i] = {
        kind: 1,
        id: i + 1,
      };
    }
  }

  getHeldItem(): Item {
    return this.items[this.slot];
  }

  updateSlot(keys: ReadonlySet<string>) {
    for (let i = 1; i <= this.items.length; i++) {
      if (keys.has(`Digit${i}`)) {
        this.slot = i - 1;
        break;
      }
    }
  }
}
