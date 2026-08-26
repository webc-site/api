import { beforeAll, describe, expect, it } from "bun:test";
import b36e from "@3-/b36/b36e.js";
import KV from "../../db/KV.js";
import reqCtx from "../../lib/srv/ctx.js";
import { keyVerify } from "../db/mail/key.js";
import { signUpVerifyNew } from "../db/mail/verify.js";
import orgInit from "../db/org/dbInit.js";
import UserNewD from "../gen/mail/UserNewD.js";
import userNew from "../url/mail/userNew.js";

describe("mail userNew", () => {
  const org_id = 1,
    bid = Buffer.from("1234567890123456"),
    ip = Buffer.from([127, 0, 0, 1]),
    ua = 1,
    ctx = reqCtx({
      org_id: Promise.resolve(org_id),
      bid,
      host: "test.com",
      lang: "zh",
      ip,
      ua: Promise.resolve(ua)
    });

  beforeAll(async () => {
    await orgInit(org_id);
  });

  it("user signup by mail succeeds", async () => {
    const mail = "user_new_" + Date.now() + "@test.com",
      name = "测试用户",
      pwd = "password123";

    await signUpVerifyNew(org_id, "test.com", "zh", mail);
    const code_buf = await KV.getBuffer(keyVerify(1)(org_id, mail));
    expect(code_buf).toBeDefined();

    const signup_buf = await userNew.call(ctx, mail, name, pwd, b36e(code_buf)),
      [uid, state] = UserNewD(signup_buf);

    expect(uid).toBeGreaterThan(0);
    expect(state || 0).toBe(0);
  });
});
