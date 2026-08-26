import { dUint32 } from "@1-/proto/D.js";
import utf8d from "@3-/utf8/utf8d.js";

export default function* (body) {
  let pos = 0;
  while (pos < body.length) {
    const p1 = body.indexOf(0, pos),
      mod = utf8d(body.subarray(pos, p1)),
      [id, p2] = dUint32(body, p1 + 1),
      [call_len, p3] = dUint32(body, p2),
      call_bin = body.subarray(p3, p3 + call_len),
      [tag] = dUint32(call_bin, 0);

    yield [id, mod, (tag >>> 3) - 1, call_bin];
    pos = p3 + call_len;
  }
}
