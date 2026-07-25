export const GEN_SIDE = 256;
export const BITS_PER_ID = 8; // must be a factor of 32
export const CHUNK_SIDE_SHIFT = 3;
export const DAY_LENGTH_SECONDS = 600;
export const MAX_CHUNK_BATCH_SIZE = 8192;
export const MAX_CHUNKS_LOADED = 256_000;
export const MIP_CAPACITY = Math.ceil(Math.ceil(GEN_SIDE / 2) ** 3 / 32);

export const CHUNK_LEN = Math.ceil(
  ((1 << CHUNK_SIDE_SHIFT) ** 3 * BITS_PER_ID) / 32,
);

export const SHADER_CONFIG = {
  GEN_SIDE,
  MAX_CHUNK_BATCH_SIZE,
  CHUNK_SIDE_SHIFT,
  BITS_PER_ID,
} as const;

export type Config = {
  buffer: GPUBuffer;
  uniformData: ArrayBuffer;
  update: (queue: GPUQueue, values: Partial<ConfigValues>) => void;
};

type ConfigValues = {
  maxTraceDist: number;
  timeOfDay: number;
  pendingAction: 0 | 1 | 2; // none | break | place
};

export function createConfig(device: GPUDevice, initial: ConfigValues): Config {
  const uniformData = new ArrayBuffer(12);
  const floatView = new Float32Array(uniformData);
  const uintView = new Uint32Array(uniformData);

  floatView[0] = initial.maxTraceDist;
  floatView[1] = initial.timeOfDay;
  uintView[2] = initial.pendingAction;

  const buffer = device.createBuffer({
    label: "config buffer",
    size: uniformData.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });

  device.queue.writeBuffer(buffer, 0, uniformData);

  function update(queue: GPUQueue, values: Partial<ConfigValues>) {
    let dirty = false;

    if (values.maxTraceDist !== undefined) {
      floatView[0] = values.maxTraceDist;
      dirty = true;
    }
    if (values.timeOfDay !== undefined) {
      floatView[1] = values.timeOfDay;
      dirty = true;
    }
    if (values.pendingAction !== undefined) {
      uintView[2] = values.pendingAction;
      dirty = true;
    }

    if (dirty) {
      queue.writeBuffer(buffer, 0, uniformData);
    }
  }

  return { buffer, uniformData, update };
}
