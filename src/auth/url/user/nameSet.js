import nameSet from "../../db/user/nameSet.js";
import NameSetE from "../../gen/user/NameSetE.js";

export default async function (user_id, name) {
  if (await this.hasUser(user_id)) {
    await nameSet(await this.org_id, user_id, name);
  }
  return NameSetE([]);
}
