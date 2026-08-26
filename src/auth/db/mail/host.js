import hostDecode from "../../../lib/hostDecode.js";
import kvU64 from "../../../db/kvU64.js";
import { MAIL_HOST_ID } from "./hostNew.js";
import { keyHost } from "./key.js";

export default async (host) => {
  host = hostDecode(host);
  const k = keyHost(host);
  let id = MAIL_HOST_ID.get(k);
  if (id) return id;

  id = await kvU64(k);
  if (id) MAIL_HOST_ID.set(k, id);
  return id;
};
