import codeSend from "../../../lib/mail/codeSend.js";
import { codeByKey, codeVerify } from "../_/verify.js";
import { keyVerify } from "./key.js";

const verify = (kind, tpl) => {
  const key = keyVerify(kind);
  return [
    // verifyNew
    async (org_id, host, lang, mail) => {
      const code = await codeByKey(key(org_id, mail));
      if (code) {
        codeSend(await import(`../../i18n/${lang}/${tpl}.js`), host, mail, code);
      }
    },
    // verify
    (org_id, mail, verify_code) => codeVerify(key(org_id, mail), verify_code)
  ];
};

export const [signUpVerifyNew, signUpVerify] = verify(1, "signUpMail"),
  [passwordResetVerifyNew, passwordResetVerify] = verify(2, "passwordResetMail");
