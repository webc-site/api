#!/usr/bin/env bun

import { setApi, setFetch, setOnCaptcha, setOnErr } from "../api/js/node_modules/@1-/protoapi/_.js";
import userNewByMail from "../api/js/auth/userNewByMail.js";

setApi("http://127.0.0.1:3003");
setFetch((url, opt = {}) =>
  fetch(url, {
    ...opt,
    headers: {
      "accept-language": "zh",
      origin: "http://127.0.0.1",
      ...opt.headers
    }
  })
);
setOnCaptcha(async () => {
  throw new Error("need captcha");
});
setOnErr((err) => {
  console.error("api err:", err);
});

const mail = "demo" + Date.now() + "@test.com";

console.log(await userNewByMail(mail, "DemoUser", "password123"));
