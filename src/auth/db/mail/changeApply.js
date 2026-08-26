import b36e from "@3-/b36/b36e.js";
import KV from "../../../db/KV.js";
import codeSend from "../../../lib/mail/codeSend.js";
import { keyMailChange } from "./key.js";

const TTL = 86400,
  LEN = 8,
  send = async (lang, tpl, host, to, code, opt) =>
    codeSend(await import(`../../i18n/${lang}/${tpl}.js`), host, to, code, opt);

export default async (org_id, host, lang, old_mail, new_mail) => {
  const bin = crypto.getRandomValues(new Uint8Array(old_mail ? LEN * 2 : LEN));
  await KV.set(keyMailChange(org_id, bin), (old_mail || "") + "\0" + new_mail, "EX", TTL);

  if (old_mail) {
    const old_code = b36e(bin.subarray(0, LEN)),
      new_code = b36e(bin.subarray(LEN)),
      opt = { from: old_mail, to: new_mail };

    Promise.all(
      [
        ["mailChangeOld", old_mail, old_code],
        ["mailChangeNew", new_mail, new_code]
      ].map(([tpl, mail, code]) => send(lang, tpl, host, mail, code, opt))
    );
    return true;
  }

  send(lang, "mailBind", host, new_mail, b36e(bin), { to: new_mail });
  return false;
};
