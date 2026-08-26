import { CAPTCHA } from "@1-/protoapi/STATUS.js";
import NODE_ENV from "../const/NODE_ENV.js";
import captcha from "./captcha.js";

export default (fn) =>
  NODE_ENV === "test"
    ? fn
    : async function (...args) {
        if (await captcha(this.req.header("pragma"))) {
          return fn.apply(this, args);
        }
        throw CAPTCHA;
      };
