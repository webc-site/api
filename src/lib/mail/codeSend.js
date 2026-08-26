import split from "@3-/split";
import sendMail from "../sendMail.js";
import mdHtm from "./mdHtm.js";

export default (render_mod, host, to, code, opt) => {
  const md = render_mod.default({ ...opt, code, token_str: "**" + code + "**\n" }),
    [head, body] = split(md, "\n"),
    trim_head = head.trim(),
    title = host + " - " + trim_head + (trim_head.includes(code) ? "" : " : " + code),
    trim_body = body.trimStart();

  return sendMail(host, to, title, trim_body.replaceAll("*", ""), mdHtm(trim_body));
};
