import { UAParser } from "ua-parser-js";

const EMPTY = "",
  HEADER_LI = ["user-agent", "sec-ch-ua", "sec-ch-ua-platform", "sec-ch-ua-platform-version"],
  uaHeaders = (req) => {
    const r = {};
    for (const k of HEADER_LI) r[k] = req.header(k) || EMPTY;
    return r;
  };

export default async (req) => {
  const headers = uaHeaders(req),
    res = await new UAParser(headers).getResult().withClientHints(),
    { browser, os } = res;

  return [browser.name || EMPTY, browser.version || EMPTY, os.name || EMPTY, os.version || EMPTY];
};
