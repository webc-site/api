import { bidUserExit } from "../../db/user/bid.js";
import ExitE from "../../gen/user/ExitE.js";

export default async function (user_id) {
  const { org_id, bid } = this;
  await bidUserExit(await org_id, bid, user_id);
  return ExitE([]);
}
