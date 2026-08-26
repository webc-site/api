import mailOrgUser from "../db/mail/orgUser.js";
import phoneOrgUser from "../db/phone/orgUser.js";
import split from "../db/phone/split.js";
import { MAIL, PHONE } from "../gen/AccountType.js";
import InfoE from "../gen/InfoE.js";

export default async function (account) {
  const org = await this.org_id,
    is_mail = account.includes("@"),
    type = is_mail ? MAIL : PHONE,
    exist = Boolean(
      await (is_mail ? mailOrgUser(org, account) : phoneOrgUser(org, ...split(account)))
    );

  return InfoE([type, exist]);
}
