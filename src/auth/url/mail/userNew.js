import orgDb from "../../../db/orgDb.js";
import split from "../../../lib/mail/split.js";
import mailNew from "../../db/mail/new.js";
import mailOrgUser from "../../db/mail/orgUser.js";
import { signUpVerify } from "../../db/mail/verify.js";
import { USER } from "../../db/org/LEVEL.js";
import orgUser from "../../db/org/orgUser.js";
import { bidUserAccountSet } from "../../db/user/bid.js";
import { MAIL } from "../../gen/AuthType.js";
import UserNewE from "../../gen/mail/UserNewE.js";
import { ERR_MAIL_EXIST, ERR_VERIFY_CODE } from "../../gen/mail/UserNewState.js";

export default async function (mail, name, password, verify_code) {
  mail = split(mail).join("@");
  const { org_id, bid, ip, ua } = this,
    org = await org_id;
  let r;

  if (!(await signUpVerify(org, mail, verify_code))) {
    r = [0, ERR_VERIFY_CODE];
  } else if (await mailOrgUser(org, mail)) {
    r = [0, ERR_MAIL_EXIST];
  } else {
    const mail_id = await mailNew(mail),
      user_id = await orgUser(orgDb(org), org, USER, name, { password, mail: mail_id });
    await bidUserAccountSet(org, bid, user_id, MAIL, mail, ip, await ua);
    r = [user_id];
  }

  return UserNewE(r);
}
