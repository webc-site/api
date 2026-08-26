import binU64 from "@3-/intbin/binU64.js";
import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import InfoD from "../../gen/token/InfoD.js";
import { keyToken, keyUserToken } from "./key.js";

export default async (org_id, rel_id) => {
  const id_buf_li = await KV.zrevrangeBuffer(keyUserToken(org_id, u64Buf(rel_id)), 0, -1);

  if (!id_buf_li.length) return [];

  const token_buf_li = await KV.mgetBuffer(id_buf_li.map(keyToken.bind(null, org_id)));

  return id_buf_li.map((id_buf, i) => [binU64(id_buf), InfoD(token_buf_li[i])]);
};
