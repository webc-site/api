import captchaVerify from "../../../lib/captchaVerify.js";
import split from "../../../lib/mail/split.js";
import mailOrgUser from "../../db/mail/orgUser.js";
import { passwordResetVerifyNew } from "../../db/mail/verify.js";
import PasswordResetE from "../../gen/mail/PasswordResetE.js";
import { ERR_MAIL_NOT_EXIST, OK } from "../../gen/mail/PasswordResetState.js";

export default captchaVerify(async function (to) {
  to = split(to).join("@");
  const { org_id, host, lang } = this,
    org = await org_id;

  let r;
  if (!(await mailOrgUser(org, to))) {
    r = [ERR_MAIL_NOT_EXIST];
  } else {
    await passwordResetVerifyNew(org, host, lang, to);
    r = [OK];
  }

  return PasswordResetE(r);
});
