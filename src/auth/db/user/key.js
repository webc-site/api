import { keyOrg } from "../org/key.js";
import u64Buf from "@3-/intbin/u64Buf.js";

const PREFIX_BID = Buffer.from("bidOrgUser:"),
  PREFIX_BID_NOW = Buffer.from("bidOrgUserNow:"),
  PREFIX_SIGNIN_FAIL = Buffer.from("userSigninFail:");

export const keyBid = (org_id, bid) => keyOrg(PREFIX_BID, org_id, bid),
  keyBidNow = (org_id, bid) => keyOrg(PREFIX_BID_NOW, org_id, bid),
  keySigninFail = (org_id, user_id) => keyOrg(PREFIX_SIGNIN_FAIL, org_id, u64Buf(user_id));
