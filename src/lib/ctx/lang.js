import CODE from "@3-/lang/CODE.js";
import split from "@3-/split";

export default (c) => {
  const lang = c.req.header("accept-language");
  if (lang)
    for (const item of lang.split(",")) {
      const tag = split(item, ";")[0].trim(),
        p1 = CODE.indexOf(tag);
      if (p1 >= 0) return CODE[p1];
      const p2 = CODE.indexOf(split(tag, "-")[0]);
      if (p2 >= 0) return CODE[p2];
    }
  return "en";
};
