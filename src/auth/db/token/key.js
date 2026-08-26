import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import InfoE from "../../gen/token/InfoE.js";
import { keyOrg } from "../org/key.js";

const [PREFIX_USER_TOKEN, PREFIX_TOKEN] = ["userToken:", "token:"].map((s) => Buffer.from(s)),
  tokenPipeSet = (p, org_id, id_buf, info) =>
    p.set(keyToken(org_id, id_buf), Buffer.from(InfoE(info)));

export const keyUserToken = (org_id, uid_buf) => keyOrg(PREFIX_USER_TOKEN, org_id, uid_buf),
  keyToken = (org_id, token_id_buf) => keyOrg(PREFIX_TOKEN, org_id, token_id_buf),
  tokenInfoSet = (org_id, id, info) => tokenPipeSet(KV.pipeline(), org_id, u64Buf(id), info).exec(),
  tokenKvSet = (org_id, rel_id, id, info) => {
    const id_buf = u64Buf(id);
    return tokenPipeSet(
      KV.pipeline().zadd(keyUserToken(org_id, u64Buf(rel_id)), info[3], id_buf),
      org_id,
      id_buf,
      info
    ).exec();
  },
  tokenKvRm = (org_id, rel_id, id) => {
    const id_buf = u64Buf(id);
    return KV.pipeline()
      .zrem(keyUserToken(org_id, u64Buf(rel_id)), id_buf)
      .del(keyToken(org_id, id_buf))
      .exec();
  };
