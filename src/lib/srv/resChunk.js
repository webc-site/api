import { uint32 } from "@1-/proto/E.js";
import { ERR } from "@1-/protoapi/STATUS.js";

export default (id, status, res) => {
  const h1 = uint32(id),
    h2 = uint32(status);
  if (status > ERR) {
    const chunk = new Uint8Array(h1.length + h2.length);
    chunk.set(h1, 0);
    chunk.set(h2, h1.length);
    return chunk;
  }
  const h3 = uint32(res.length),
    chunk = new Uint8Array(h1.length + h2.length + h3.length + res.length);
  chunk.set(h1, 0);
  chunk.set(h2, h1.length);
  chunk.set(h3, h1.length + h2.length);
  chunk.set(res, h1.length + h2.length + h3.length);
  return chunk;
};
