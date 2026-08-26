import changeVerify from "../../db/mail/changeVerify.js";
import { bidUserAccountSet } from "../../db/user/bid.js";
import mailSet from "../../db/mail/userMailSet.js";
import { MAIL } from "../../gen/AuthType.js";
import ChangeE from "../../gen/mail/ChangeE.js";
import { ERR_VERIFY_CODE, OK } from "../../gen/mail/ChangeState.js";
import userVerify from "./_/userVerify.js";

export default async function (uid, mail, new_code, old_code = "") {
  const { bid, ip, ua } = this;
  return ChangeE(
    await userVerify.call(this, uid, mail, async (org, old_mail, mail) => {
      if (!(await changeVerify(org, old_mail, mail, new_code, old_code))) {
        return [ERR_VERIFY_CODE];
      }
      await Promise.all([
        mailSet(org, uid, mail),
        bidUserAccountSet(org, bid, uid, MAIL, mail, ip, await ua)
      ]);
      return [OK];
    })
  );
}
