import u64B255 from "@3-/intbin/u64B255.js";
import u64B64 from "@3-/intbin/u64B64.js";
import { keyOrg } from "../org/key.js";

const COLON = Buffer.from(":"),
  PREFIX_RESET_FAIL = Buffer.from("mailResetFail:"),
  PREFIX_VERIFY = Buffer.from("mailVerifyCode:"),
  PREFIX_MAIL_CHANGE = Buffer.from("mailChangeVerifyCode:");

export const keyHost = (host) => "mailHost:" + host,
  keyMail = (host_id, prefix) => "mail:" + u64B64(host_id) + ":" + prefix,
  keyResetFail = (org_id, mail) => keyOrg(PREFIX_RESET_FAIL, org_id, Buffer.from(mail)),
  keyMailChange = (org_id, code_bin) => keyOrg(PREFIX_MAIL_CHANGE, org_id, code_bin),
  keyVerify = (kind) => {
    const prefix = Buffer.concat([PREFIX_VERIFY, u64B255(kind), COLON]);
    return (org_id, mail) => keyOrg(prefix, org_id, Buffer.from(mail));
  };
