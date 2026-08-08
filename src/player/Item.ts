export interface Item {
  kind: number;
  id: number;
}

export function createEmptyItem(): Item {
  return { kind: 0, id: 0 };
}
