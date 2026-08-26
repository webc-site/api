import binU64 from "@3-/intbin/binU64.js";
import KV from "./KV.js";

export default async (k) => {
  const bin = await KV.getBuffer(k);
  if (bin) return binU64(bin);
};
