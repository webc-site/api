import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import orgDb from "../../../db/orgDb.js";
import { keyOrgMailUser, keyUserMail } from "../org/key.js";
import mailCopy from "./copy.js";
import mailId from "./id.js";
import mailNew from "./new.js";
import userMail from "./userMail.js";

export default async (org_id, user_id, mail) => {
  const old_mail = await userMail(org_id, user_id),
    old_mail_id = old_mail ? await mailId(old_mail) : 0,
    new_mail_id = await mailNew(mail);

  if (old_mail_id && old_mail_id === new_mail_id) return;

  const db = orgDb(org_id),
    uid_buf = u64Buf(user_id),
    user_mail = await mailCopy(db, new_mail_id),
    p = [
      db("UPDATE ONLY type::record('user',$user_id) SET mail=type::record('mail',$mail_id)", {
        user_id,
        mail_id: new_mail_id
      }),
      KV.mset(
        keyUserMail(org_id, uid_buf),
        user_mail,
        keyOrgMailUser(org_id, u64Buf(new_mail_id)),
        uid_buf
      )
    ];

  if (old_mail_id) {
    p.push(KV.del(keyOrgMailUser(org_id, u64Buf(old_mail_id))));
  }

  return Promise.all(p);
};
