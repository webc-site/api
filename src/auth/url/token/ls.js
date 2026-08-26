import { KIND_USER } from "../../db/token/KIND.js";
import tokenLs from "../../db/token/ls.js";
import tokenNew from "../../db/token/new.js";
import LsE from "../../gen/token/LsE.js";

export default async function (uid) {
  if (await this.hasUser(uid)) {
    const org_id = await this.org_id,
      li = await tokenLs(org_id, uid);
    return LsE([li.length ? li : [await tokenNew(org_id, KIND_USER, uid, "")]]);
  }
  return LsE([]);
}
