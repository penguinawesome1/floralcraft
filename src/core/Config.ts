export const GEN_SIDE = 256;
export const BITS_PER_ID = 8; // must be a factor of 32
export const CHUNK_SIDE_SHIFT = 3;
export const DAY_LENGTH_SECONDS = 600;
export const MAX_CHUNK_BATCH_SIZE = 8192;
export const MAX_CHUNKS_LOADED = 64_000;

export type Config = {
  buffer: GPUBuffer;
  uniformData: Float32Array;
  update: (queue: GPUQueue, values: Partial<ConfigValues>) => void;
};

type ConfigValues = {
  maxTraceDist: number;
  timeOfDay: number;
};

export function createConfig(device: GPUDevice, initial: ConfigValues): Config {
  const uniformData = new Float32Array([
    initial.maxTraceDist,
    initial.timeOfDay,
  ]);

  const buffer = device.createBuffer({
    label: "config buffer",
    size: uniformData.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });

  device.queue.writeBuffer(buffer, 0, uniformData);

  function update(queue: GPUQueue, values: Partial<ConfigValues>) {
    let dirty = false;

    if (values.maxTraceDist !== undefined) {
      uniformData[0] = values.maxTraceDist;
      dirty = true;
    }
    if (values.timeOfDay !== undefined) {
      uniformData[1] = values.timeOfDay;
      dirty = true;
    }

    if (dirty) {
      queue.writeBuffer(buffer, 0, uniformData);
    }
  }

  return { buffer, uniformData, update };
}
