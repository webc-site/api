import u64Buf from "@3-/intbin/u64Buf.js";
import sec from "@3-/time/sec.js";
import KV from "../../../db/KV.js";
import kvU64 from "../../../db/kvU64.js";
import AccountD from "../../gen/AccountD.js";
import mailCopy from "../mail/copy.js";
import phoneCopy from "../phone/copy.js";
import { passwordHash } from "../user/password.js";
import {
  keyOrgMailUser,
  keyOrgPhoneUser,
  keyUserAccount,
  keyUserLevel,
  keyUserMail,
  keyUserName,
  keyUserPassword,
  keyUserPhone
} from "./key.js";

export const orgUserName = (org_id, user_id) => KV.get(keyUserName(org_id, u64Buf(user_id))),
  orgUserNameLi = (org_id, user_id_li) =>
    user_id_li.length ? KV.mget(user_id_li.map((id) => keyUserName(org_id, u64Buf(id)))) : [],
  orgUserAccountLi = async (org_id, user_id_li) =>
    user_id_li.length
      ? (await KV.mgetBuffer(user_id_li.map((id) => keyUserAccount(org_id, u64Buf(id))))).map(
          (buf) => (buf ? AccountD(buf) : [0, ""])
        )
      : [],
  orgUserLevel = (org_id, user_id) => kvU64(keyUserLevel(org_id, u64Buf(user_id))),
  orgUserPassword = (org_id, user_id) => KV.getBuffer(keyUserPassword(org_id, u64Buf(user_id)));

export default async (db, org_id, level, name, conf = {}) => {
  name = name.slice(0, 32);
  const { password, mail, phone } = conf;

  let user_mail, user_phone;
  if (mail) user_mail = await mailCopy(db, mail);
  if (phone) user_phone = await phoneCopy(db, phone);

  const password_hash = password ? await passwordHash(password) : new Uint8Array(),
    sql =
      "CREATE ONLY user SET level=$level,ts=$ts,name=$name,password=$password_hash,mail=IF $mail { type::record('mail',$mail) } ELSE { NULL },phone=IF $phone { type::record('phone',$phone) } ELSE { NULL }",
    ts = sec(),
    [{ id }] = await db(sql, {
      level,
      ts,
      name,
      password_hash,
      mail: mail || null,
      phone: phone || null
    }),
    user_id = id.id,
    uid_buf = u64Buf(user_id),
    kv_args = [
      keyUserName(org_id, uid_buf),
      name,
      keyUserLevel(org_id, uid_buf),
      u64Buf(level),
      keyUserPassword(org_id, uid_buf),
      password_hash
    ];

  if (user_mail) {
    kv_args.push(
      keyUserMail(org_id, uid_buf),
      user_mail,
      keyOrgMailUser(org_id, u64Buf(mail)),
      uid_buf
    );
  }

  if (user_phone) {
    kv_args.push(
      keyUserPhone(org_id, uid_buf),
      user_phone,
      keyOrgPhoneUser(org_id, u64Buf(phone)),
      uid_buf
    );
  }

  await KV.mset(...kv_args);

  return user_id;
};
