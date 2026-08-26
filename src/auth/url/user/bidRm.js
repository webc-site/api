import { bidUserRm } from "../../db/user/bid.js";
import BidRmE from "../../gen/user/BidRmE.js";

export default async function (user_id) {
  const { org_id, bid } = this;
  await bidUserRm(await org_id, bid, user_id);
  return BidRmE([]);
}
