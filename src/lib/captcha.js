import CAPTCHA_URL from "../../conf/CAPTCHA.js";

export default async (captcha) => {
  if (captcha) {
    try {
      const res = await fetch(CAPTCHA_URL + "/verify/" + captcha);
      return res.ok && (await res.text()) === "1";
    } catch {}
  }
  return false;
};
