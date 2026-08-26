import sleep from "@3-/sleep";
import { encode, decode, APPLICATION_CBOR } from "./cbor.js";

export default async (rpc_url, user, pass, ns) => {
  const auth = ns ? { user, pass, ns } : { user, pass },
    payload = encode({
      id: "signin",
      method: "signin",
      params: [auth]
    });

  let i = 0;
  for (;;) {
    try {
      const res = await fetch(rpc_url, {
          method: "POST",
          headers: { "Content-Type": APPLICATION_CBOR, Accept: APPLICATION_CBOR },
          body: payload
        }),
        data = decode(new Uint8Array(await res.arrayBuffer()));
      if (data.error) throw new Error(data.error.message);
      return data.result;
    } catch (e) {
      if (++i > 3) throw e;
      await sleep(1000);
    }
  }
};
