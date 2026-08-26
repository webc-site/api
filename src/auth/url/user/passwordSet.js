import { captchaVerify, failIncr } from "../../../db/captchaIncr.js";
import KV from "../../../db/KV.js";
import orgDb from "../../../db/orgDb.js";
import split from "../../../lib/mail/split.js";
import { keyResetFail } from "../../db/mail/key.js";
import mailOrgUser from "../../db/mail/orgUser.js";
import { passwordResetVerify } from "../../db/mail/verify.js";
import { bidUserAccountSet } from "../../db/user/bid.js";
import passwordSet from "../../db/user/passwordSet.js";
import { MAIL } from "../../gen/AuthType.js";
import PasswordSetE from "../../gen/user/PasswordSetE.js";
import { ERR_MAIL_NOT_EXIST, ERR_VERIFY_CODE, OK } from "../../gen/user/PasswordSetResult.js";

export default async function (mail, password, verify_code) {
  mail = split(mail).join("@");
  const { org_id, bid, ip, ua } = this,
    org = await org_id,
    key_li = [keyResetFail(org, mail)],
    fail = await captchaVerify(key_li, this);

  let r;
  if (!(await passwordResetVerify(org, mail, verify_code))) {
    await failIncr(key_li);
    r = [ERR_VERIFY_CODE];
  } else {
    if (fail) await KV.del(...key_li);

    const user_id = await mailOrgUser(org, mail);
    if (!user_id) {
      r = [ERR_MAIL_NOT_EXIST];
    } else {
      await Promise.all([
        passwordSet(orgDb(org), org, user_id, password),
        bidUserAccountSet(org, bid, user_id, MAIL, mail, ip, await ua)
      ]);
      r = [OK];
    }
  }

  return PasswordSetE(r);
}
