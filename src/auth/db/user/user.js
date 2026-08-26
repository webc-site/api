import { orgUserName } from "../org/orgUser.js";
import userMail from "../mail/userMail.js";

export default async (org_id, user_id) => {
  if (!user_id) return [0, "", ""];
  const [name, mail] = await Promise.all([orgUserName(org_id, user_id), userMail(org_id, user_id)]);
  if (!name && !mail) return [0, "", ""];
  return [user_id, name || "", mail || ""];
};
