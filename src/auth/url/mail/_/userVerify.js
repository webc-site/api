import split from "../../../../lib/mail/split.js";
import mailOrgUser from "../../../db/mail/orgUser.js";
import userMail from "../../../db/mail/userMail.js";
import { ERR_AUTH, ERR_MAIL_EXIST, ERR_MAIL_INVALID } from "../../../gen/mail/ChangeState.js";

export default async function (uid, mail, fn) {
  const [prefix, host_name] = split(mail);
  if (!prefix || !host_name) return [ERR_MAIL_INVALID];

  if (!(await this.hasUser(uid))) return [ERR_AUTH];

  mail = prefix + "@" + host_name;
  const org = await this.org_id,
    exist_uid = await mailOrgUser(org, mail);
  if (exist_uid && exist_uid !== uid) return [ERR_MAIL_EXIST];

  return fn(org, await userMail(org, uid), mail);
}
