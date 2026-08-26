import { KIND_USER } from "../../db/token/KIND.js";
import tokenRm from "../../db/token/rm.js";
import RmE from "../../gen/token/RmE.js";

export default async function (uid, id) {
  if (await this.hasUser(uid)) {
    await tokenRm(await this.org_id, KIND_USER, uid, id);
  }
  return RmE([]);
}
