import { KIND_USER } from "../../db/token/KIND.js";
import tokenNameSet from "../../db/token/nameSet.js";
import NameSetE from "../../gen/token/NameSetE.js";

export default async function (uid, id, name) {
  if (await this.hasUser(uid)) {
    await tokenNameSet(await this.org_id, KIND_USER, uid, id, name);
  }
  return NameSetE([]);
}
