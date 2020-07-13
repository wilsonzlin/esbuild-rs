const native = require('./native.node');

const requests = new Map<number, [(minCodeLen: number) => void, (error: Error) => void]>();
let nextId = 0;

export const startService = () => {
  native.startService((err: Error | undefined, id: number, minCodeLen: number) => {
    const req = requests.get(id);
    if (!req) {
      throw new Error(`Unknown request ID ${id}`);
    }
    requests.delete(id);
    const [resolve, reject] = req;
    if (err) {
      reject(err);
    } else {
      resolve(minCodeLen);
    }
  });
};

export const stopService = () => {
  native.stopService();
};

export const minify = (codeUtf8: Buffer): Promise<Buffer> => new Promise((resolve, reject) => {
  let id = nextId++;
  let buffer: Buffer;
  // Slicing does not create a new buffer.
  const resolveWithValue = (minCodeLen: number) => resolve(buffer.slice(0, minCodeLen));
  requests.set(id, [resolveWithValue, reject]);
  buffer = native.minify(id, codeUtf8);
});
