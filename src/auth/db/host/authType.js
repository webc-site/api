import { uint32Li as dUint32Li } from "@1-/proto/D.js";
import KV from "../../../db/KV.js";
import { keyAuthType } from "./key.js";

export const authType = async (host) => {
  const bin = await KV.getBuffer(keyAuthType(host));
  if (bin) return dUint32Li(bin);
};
