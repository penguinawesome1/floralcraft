import { type Item, createEmptyItem } from "./Item";

export class Hotbar {
  private items = Array.from({ length: 9 }, createEmptyItem);
  private slot = 0;

  constructor() {
    this.items[0] = { kind: 1, id: 1 };
    this.items[1] = { kind: 1, id: 2 };
    this.items[2] = { kind: 1, id: 3 };
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
