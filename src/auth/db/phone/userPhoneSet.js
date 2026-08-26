import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import orgDb from "../../../db/orgDb.js";
import { keyOrgPhoneUser, keyUserPhone } from "../org/key.js";
import phoneCopy from "./copy.js";
import phoneId from "./id.js";
import phoneNew from "./new.js";
import split from "./split.js";
import userPhone from "./userPhone.js";

export default async (org_id, user_id, area, num) => {
  const old_phone = await userPhone(org_id, user_id),
    old_phone_id = old_phone ? await phoneId(...split(old_phone)) : 0,
    new_phone_id = await phoneNew(area, num);

  if (old_phone_id && old_phone_id === new_phone_id) return;

  const db = orgDb(org_id),
    uid_buf = u64Buf(user_id),
    user_phone = await phoneCopy(db, new_phone_id),
    p = [
      db("UPDATE ONLY type::record('user',$user_id) SET phone=type::record('phone',$phone_id)", {
        user_id,
        phone_id: new_phone_id
      }),
      KV.mset(
        keyUserPhone(org_id, uid_buf),
        user_phone,
        keyOrgPhoneUser(org_id, u64Buf(new_phone_id)),
        uid_buf
      )
    ];

  if (old_phone_id) {
    p.push(KV.del(keyOrgPhoneUser(org_id, u64Buf(old_phone_id))));
  }

  return Promise.all(p);
};
