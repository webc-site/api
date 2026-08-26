import user from "../db/user/user.js";
import UserE from "../gen/UserE.js";

export default async function (user_id) {
  let r = [];
  if (await this.hasUser(user_id)) {
    r = await user(await this.org_id, user_id);
  }
  return UserE(r);
}
