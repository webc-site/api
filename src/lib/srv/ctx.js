import hasUser from "../ctx/hasUser.js";
import host from "../ctx/host.js";
import ip from "../ctx/ip.js";
import lang from "../ctx/lang.js";
import org_id from "../ctx/org_id.js";
import ua from "../ctx/ua.js";
import uid from "../ctx/uid.js";

const GET = {
  hasUser,
  host,
  ip,
  lang,
  org_id,
  ua,
  uid
};

export default (c) => {
  const cache = {},
    ctx = new Proxy(c, {
      get: (target, prop) => {
        if (prop in target) return target[prop];
        const fn = GET[prop];
        if (fn) return cache[prop] || (cache[prop] = fn(ctx));
      }
    });
  return ctx;
};
