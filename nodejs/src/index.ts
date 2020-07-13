const native = require('./native.node');

const requests = new Map<number, [Function, Function]>();
let nextId = 0;

export const startService = () => {
  native.startService((err: Error, id: number, buffer: Buffer) => {
    const req = requests.get(id);
    if (!req) {
      throw new Error(`Unknown request ID ${id} with response ${buffer.toString()}`);
    }
    requests.delete(id);
    const [resolve, reject] = req;
    if (err) {
      reject(err);
    } else {
      resolve(buffer);
    }
  });
};

export const stopService = () => {
  native.stopService();
};

export const minify = (codeUtf8: Buffer): Promise<Buffer> => new Promise((resolve, reject) => {
  let id = nextId++;
  requests.set(id, [resolve, reject]);
  native.minify(id, codeUtf8);
});
