import { getCookie } from "hono/cookie";
import { uint32 as eUint32 } from "@1-/proto/E.js";
import { uint32 as dUint32 } from "@1-/proto/D.js";
import reqCtx from "./ctx.js";
import cookieStr from "./cookie_str.js";
import BASE64URL from "../../const/BASE64URL.js";
import IS_PROD from "../../const/IS_PROD.js";
import { TODAY } from "../../const/TODAY.js";

const B = "B",
  BID_LEN = 16,
  MAX_AGE = 9e7, // 约 1041 天（2.85 年）
  setB = (c, bid) =>
    c.header(
      "set-cookie",
      cookieStr(
        reqCtx(c).host,
        B,
        Buffer.concat([bid, eUint32(TODAY)]).toString(BASE64URL),
        MAX_AGE,
        IS_PROD
      ),
      { append: true }
    );

export default async (c, next) => {
  const b = getCookie(c, B),
    bin = b && Buffer.from(b, BASE64URL);
  let bid;

  if (bin && bin.length > BID_LEN) {
    bid = bin.subarray(0, BID_LEN);
    if (TODAY > dUint32(bin.subarray(BID_LEN))) setB(c, bid);
  } else {
    setB(c, (bid = crypto.getRandomValues(Buffer.alloc(BID_LEN))));
  }

  c.bid = bid;
  await next();
};
