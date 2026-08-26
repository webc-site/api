import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import { keyUserMail } from "../org/key.js";

export default (org_id, user_id) => {
  if (user_id) return KV.get(keyUserMail(org_id, u64Buf(user_id)));
};
