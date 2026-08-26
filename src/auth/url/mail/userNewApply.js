import captchaVerify from "../../../lib/captchaVerify.js";
import split from "../../../lib/mail/split.js";
import mailOrgUser from "../../db/mail/orgUser.js";
import { signUpVerifyNew } from "../../db/mail/verify.js";
import UserNewApplyE from "../../gen/mail/UserNewApplyE.js";
import { ERR_MAIL_EXIST, OK } from "../../gen/mail/UserNewApplyState.js";

export default captchaVerify(async function (to) {
  to = split(to).join("@");
  const { org_id, host, lang } = this,
    org = await org_id;

  let r;
  if (await mailOrgUser(org, to)) {
    r = [ERR_MAIL_EXIST];
  } else {
    await signUpVerifyNew(org, host, lang, to);
    r = [OK];
  }

  return UserNewApplyE(r);
});
