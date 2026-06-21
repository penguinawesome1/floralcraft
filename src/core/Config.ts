export type Config = {
  buffer: GPUBuffer;
  uniformData: Float32Array;
  update: (queue: GPUQueue, values: Partial<ConfigValues>) => void;
};

type ConfigValues = {
  max_trace_dist: number;
};

export function createConfig(device: GPUDevice, initial: ConfigValues): Config {
  const uniformData = new Float32Array([initial.max_trace_dist]);

  const buffer = device.createBuffer({
    label: "config buffer",
    size: uniformData.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });

  device.queue.writeBuffer(buffer, 0, uniformData);

  function update(queue: GPUQueue, values: Partial<ConfigValues>) {
    if (values.max_trace_dist !== undefined) {
      uniformData[0] = values.max_trace_dist;
      queue.writeBuffer(buffer, 0, uniformData);
    }
  }

  return { buffer, uniformData, update };
}
