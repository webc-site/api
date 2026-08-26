import { orgUserPassword } from "../org/orgUser.js";
import { passwordVerify } from "./password.js";

export default async (org_id, user_id, password) =>
  passwordVerify(password, await orgUserPassword(org_id, user_id));
