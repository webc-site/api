import binU64 from "@3-/intbin/binU64.js";
import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import { orgUserLevel } from "../org/orgUser.js";

export default (idFunc, keyOrgUser) =>
  async (org_id, ...args) => {
    const id = await idFunc(...args);
    if (!id) return;

    const uid_buf = await KV.getBuffer(keyOrgUser(org_id, u64Buf(id)));
    if (!uid_buf) return;

    const user_id = binU64(uid_buf);
    if (await orgUserLevel(org_id, user_id)) return user_id;
  };
