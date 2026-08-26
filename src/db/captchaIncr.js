import { CAPTCHA } from "@1-/protoapi/STATUS.js";
import captcha from "../lib/captcha.js";
import KV from "./KV.js";

const MAX_FAIL = 3,
  EXPIRE = 86400;

export const captchaVerify = async (key_li, ctx, max_fail = MAX_FAIL) => {
    const fail_li = await KV.mget(key_li);

    if (fail_li.some((f) => +f >= max_fail) && !(await captcha(ctx.req.header("pragma")))) {
      throw CAPTCHA;
    }
    return fail_li.some((f) => +f > 0);
  },
  failIncr = (key_li) =>
    key_li.reduce((p, key) => p.incr(key).expire(key, EXPIRE), KV.pipeline()).exec();
