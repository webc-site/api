import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import { keyUserPassword } from "../org/key.js";
import { passwordHash } from "./password.js";

export default async (db, org_id, user_id, password) => {
  const password_hash = await passwordHash(password);
  await Promise.all([
    db("UPDATE ONLY type::record('user',$user_id) SET password=$password_hash", {
      user_id,
      password_hash
    }),
    KV.set(keyUserPassword(org_id, u64Buf(user_id)), password_hash)
  ]);
};
