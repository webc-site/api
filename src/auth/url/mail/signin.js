import split from "../../../lib/mail/split.js";
import mailOrgUser from "../../db/mail/orgUser.js";
import { bidUserAccountSet } from "../../db/user/bid.js";
import signinVerify from "../../db/user/signinVerify.js";
import { MAIL } from "../../gen/AuthType.js";
import SigninE from "../../gen/mail/SigninE.js";

export default async function (mail, password) {
  mail = split(mail).join("@");
  const { org_id, bid, ip, ua } = this,
    org = await org_id;

  let user_id = await mailOrgUser(org, mail);

  if (user_id && (await signinVerify(this, org, user_id, password, ip))) {
    await bidUserAccountSet(org, bid, user_id, MAIL, mail, ip, await ua);
  } else {
    user_id = 0;
  }

  return SigninE([user_id]);
}
