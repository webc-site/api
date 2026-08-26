import captchaVerify from "../../../lib/captchaVerify.js";
import changeApply from "../../db/mail/changeApply.js";
import ChangeApplyE from "../../gen/mail/ChangeApplyE.js";
import { ERR_MAIL_EXIST, OK } from "../../gen/mail/ChangeState.js";
import userVerify from "./_/userVerify.js";

export default captchaVerify(async function (uid, mail) {
  const { host, lang } = this;
  return ChangeApplyE(
    await userVerify.call(this, uid, mail, async (org, old_mail, mail) => {
      if (old_mail === mail) return [ERR_MAIL_EXIST];
      return [OK, await changeApply(org, host, lang, old_mail, mail)];
    })
  );
});
