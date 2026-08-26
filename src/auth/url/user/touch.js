import { bidUserTouch } from "../../db/user/bid.js";
import TouchE from "../../gen/user/TouchE.js";

export default async function (user_id) {
  const { org_id, bid, ip, ua } = this,
    exist = await bidUserTouch(await org_id, bid, user_id, ip, await ua);
  return TouchE([exist]);
}
