import int from "@3-/int";
import u64Buf from "@3-/intbin/u64Buf.js";
import sec from "@3-/time/sec.js";
import kvU64 from "../../../db/kvU64.js";
import KV from "../../../db/KV.js";
import orgDb from "../../../db/orgDb.js";
import AccountE from "../../gen/AccountE.js";
import userAgentId from "../userAgent/id.js";
import uaCopy from "../userAgent/copy.js";
import { keyUserAccount } from "../org/key.js";
import { keyBid, keyBidNow } from "./key.js";
import { SIGNIN, EXIT } from "./ACTION.js";

const WITHSCORES = "WITHSCORES",
  EMPTY_IP = new Uint8Array(),
  liDecode = (li) => {
    const r_li = [];
    for (let i = 0; i < li.length; i += 2) {
      r_li.push([int(li[i]), li[i + 1] > 0]);
    }
    return r_li;
  },
  bidNowSync = async (key_bid, key_now, p) => {
    const [[, li]] = (await p.zrevrange(key_bid, 0, 0, WITHSCORES).exec()).slice(-1),
      [first] = liDecode(li);
    return first && first[1] ? KV.set(key_now, u64Buf(first[0])) : KV.del(key_now);
  },
  bidUserSetPipe = async (org_id, bid, user_id, p = KV.pipeline()) => {
    const key_bid = keyBid(org_id, bid),
      key_now = keyBidNow(org_id, bid),
      li = await KV.zrevrange(key_bid, 0, 0, WITHSCORES),
      score = Math.max(sec(), (li.length ? int(li[1]) : 0) + 1);

    return p.zadd(key_bid, score, user_id).set(key_now, u64Buf(user_id));
  },
  uaId = async (user_agent) => {
    if (!user_agent) return 0;
    if (typeof user_agent === "number") return user_agent;
    return userAgentId(user_agent);
  },
  userBidSave = async (org_id, bid, user_id, ip, user_agent) => {
    const db = orgDb(org_id),
      [[exist]] = await db(
        "SELECT VALUE 1 FROM userBidSignined WHERE user=type::record('user',$user_id) AND bid=$bid LIMIT 1",
        { user_id, bid }
      );
    if (exist) return;

    const ua_id = await uaId(user_agent),
      now = sec();
    if (ua_id) await uaCopy(db, ua_id);
    await Promise.all([
      db(
        "CREATE ONLY userBidSignined SET user=type::record('user',$user_id),bid=$bid,ip=$ip,ua=IF $ua_id { type::record('userAgent',$ua_id) } ELSE { NULL },ts=$ts;",
        {
          user_id,
          bid,
          ip: ip || EMPTY_IP,
          ua_id: ua_id || 0,
          ts: now
        }
      ),
      db(
        "CREATE ONLY userBidLog SET user=type::record('user',$user_id),bid=$bid,ip=$ip,ua=IF $ua_id { type::record('userAgent',$ua_id) } ELSE { NULL },action=$action,ts=$ts;",
        {
          user_id,
          bid,
          ip: ip || EMPTY_IP,
          ua_id: ua_id || 0,
          action: SIGNIN,
          ts: now
        }
      )
    ]);
  },
  bidUserSet = async (org_id, bid, user_id, ip, user_agent, p) => {
    const [pipe] = await Promise.all([
      bidUserSetPipe(org_id, bid, user_id, p),
      userBidSave(org_id, bid, user_id, ip, user_agent)
    ]);
    return pipe.exec();
  };

export const bidUserNow = (org_id, bid) => kvU64(keyBidNow(org_id, bid)),
  bidUserAccountSet = async (org_id, bid, user_id, auth_type, account, ip, user_agent) => {
    const [pipe] = await Promise.all([
      bidUserSetPipe(org_id, bid, user_id),
      userBidSave(org_id, bid, user_id, ip, user_agent)
    ]);
    return pipe
      .set(keyUserAccount(org_id, u64Buf(user_id)), Buffer.from(AccountE([auth_type, account])))
      .exec();
  },
  bidUserExit = async (org_id, bid, user_id) => {
    const key_bid = keyBid(org_id, bid),
      db = orgDb(org_id),
      now = sec();
    await Promise.all([
      bidNowSync(key_bid, keyBidNow(org_id, bid), KV.pipeline().zadd(key_bid, -1, user_id)),
      db(
        "DELETE userBidSignined WHERE user=type::record('user',$user_id) AND bid=$bid;CREATE ONLY userBidLog SET user=type::record('user',$user_id),bid=$bid,ip=<bytes> '',ua=NULL,action=$action,ts=$ts;",
        {
          user_id,
          bid,
          action: EXIT,
          ts: now
        }
      )
    ]);
  },
  bidLi = async (org_id, bid) =>
    liDecode(await KV.zrevrange(keyBid(org_id, bid), 0, -1, WITHSCORES)),
  bidUserHas = async (org_id, bid, user_id) => (await KV.zscore(keyBid(org_id, bid), user_id)) > 0,
  bidUserTouch = async (org_id, bid, user_id, ip, user_agent) => {
    const exist = await bidUserHas(org_id, bid, user_id);
    if (exist) await bidUserSet(org_id, bid, user_id, ip, user_agent);
    return exist;
  },
  bidUserRm = (org_id, bid, user_id) => {
    const key_bid = keyBid(org_id, bid);
    return bidNowSync(key_bid, keyBidNow(org_id, bid), KV.pipeline().zrem(key_bid, user_id));
  };
