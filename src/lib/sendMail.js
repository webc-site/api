import int from "@3-/int";
import { gray } from "@3-/log/GRAY.js";
import ERR from "@3-/log/ERR.js";
import sendMail from "../../conf/MAIL.js";

export default async (host, to, title, txt, html) => {
  for (let i = 0; i < 3; ++i) {
    const begin = performance.now();
    try {
      console.log(host, "→", to, title);
      const res = await sendMail(host, to, title, txt, html);
      console.log(gray(int(performance.now() - begin) + "ms"), [host, to, title, txt].join("\n"));
      return res;
    } catch (err) {
      ERR(to, err);
    }
  }
};
