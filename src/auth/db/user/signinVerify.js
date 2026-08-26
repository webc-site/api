import { captchaVerify, failIncr } from "../../../db/captchaIncr.js";
import KV from "../../../db/KV.js";
import { keyIpFail } from "../ip/key.js";
import { keySigninFail } from "./key.js";
import userPasswordVerify from "./userPasswordVerify.js";

export default async (ctx, org_id, user_id, password, ip) => {
  const key_user = keySigninFail(org_id, user_id),
    key_ip = keyIpFail(ip),
    key_li = [key_user, key_ip],
    fail = await captchaVerify(key_li, ctx),
    res = await userPasswordVerify(org_id, user_id, password);

  if (res) {
    if (fail) await KV.del(...key_li);
    return res;
  }

  await failIncr(key_li);
  return res;
};
