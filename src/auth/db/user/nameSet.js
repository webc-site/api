import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import orgDb from "../../../db/orgDb.js";
import { keyUserName } from "../org/key.js";

export default (org_id, user_id, name) => {
  name = name.slice(0, 32);
  return Promise.all([
    orgDb(org_id)("UPDATE ONLY type::record('user',$user_id) SET name=$name", {
      user_id,
      name
    }),
    KV.set(keyUserName(org_id, u64Buf(user_id)), name)
  ]);
};
