import { toUnicode } from "punycode";
import psl from "@1-/psl";

export default (c) => {
  const origin = c.req.header("origin");
  if (!origin) return;
  return psl(toUnicode(new URL(origin).hostname.toLowerCase().trim()));
};
