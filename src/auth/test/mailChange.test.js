import { beforeAll, describe, expect, it } from "bun:test";
import b36e from "@3-/b36/b36e.js";
import u64B255 from "@3-/intbin/u64B255.js";
import KV from "../../db/KV.js";
import orgDb from "../../db/orgDb.js";
import reqCtx from "../../lib/srv/ctx.js";
import { keyVerify } from "../db/mail/key.js";
import { signUpVerifyNew } from "../db/mail/verify.js";
import orgInit from "../db/org/dbInit.js";
import orgUser, { orgUserAccountLi } from "../db/org/orgUser.js";
import { bidUserAccountSet } from "../db/user/bid.js";
import userMail from "../db/mail/userMail.js";
import { MAIL } from "../gen/AuthType.js";
import InfoD from "../gen/InfoD.js";
import ChangeApplyD from "../gen/mail/ChangeApplyD.js";
import ChangeD from "../gen/mail/ChangeD.js";
import {
  ERR_AUTH,
  ERR_MAIL_EXIST,
  ERR_MAIL_INVALID,
  ERR_VERIFY_CODE,
  OK
} from "../gen/mail/ChangeState.js";
import SigninD from "../gen/mail/SigninD.js";
import UserNewD from "../gen/mail/UserNewD.js";
import info from "../url/info.js";
import change from "../url/mail/change.js";
import changeApply from "../url/mail/changeApply.js";
import signin from "../url/mail/signin.js";
import userNew from "../url/mail/userNew.js";

const getVerifyCode = async (org_id, target_val) => {
  const keys = await KV.keysBuffer(Buffer.from("mailChangeVerifyCode:*")),
    prefix_len = 21 + u64B255(org_id).length + 1;

  for (const k of keys) {
    const val = await KV.get(k);
    if (val === target_val) {
      const bin = k.subarray(prefix_len);
      if (bin.length === 16) {
        return [b36e(bin.subarray(0, 8)), b36e(bin.subarray(8, 16))];
      }
      return [b36e(bin)];
    }
  }
  return [];
};

describe("mail change & changeApply", () => {
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

  let user_a_id, user_b_id;

  beforeAll(async () => {
    await orgInit(org_id);

    user_a_id = await orgUser(orgDb(org_id), org_id, 1, "UserA", {
      mail: 0,
      password: "pwd"
    });
    user_b_id = await orgUser(orgDb(org_id), org_id, 1, "UserB", {
      mail: 0,
      password: "pwd"
    });

    await bidUserAccountSet(org_id, bid, user_a_id, MAIL, "user_a@test.com", ip, ua);
    await bidUserAccountSet(org_id, bid, user_b_id, MAIL, "", ip, ua);
  });

  const ts = Date.now(),
    mail_a_old = `user_a_old_${ts}@test.com`,
    mail_a_new = `user_a_new_${ts}@test.com`,
    mail_b = `user_b_${ts}@test.com`;

  it("changeApply & change with existing old mail", async () => {
    // 预先给 user_a 绑定邮箱
    const apply_res_buf = await changeApply.call(ctx, user_a_id, mail_a_old),
      [apply_state] = ChangeApplyD(apply_res_buf);
    expect(apply_state).toBe(OK);

    const [bind_code] = await getVerifyCode(org_id, "\0" + mail_a_old);
    expect(bind_code).toBeDefined();

    const change_res_buf = await change.call(ctx, user_a_id, mail_a_old, bind_code, "");
    expect(ChangeD(change_res_buf)[0]).toBe(OK);
    expect(await userMail(org_id, user_a_id)).toBe(mail_a_old);

    const signin_old_before = await signin.call(ctx, mail_a_old, "pwd");
    expect(SigninD(signin_old_before)[0]).toBe(user_a_id);

    // 申请修改为新邮箱
    const apply_new_buf = await changeApply.call(ctx, user_a_id, mail_a_new),
      [new_apply_state, has_old_mail] = ChangeApplyD(apply_new_buf);

    expect(new_apply_state).toBe(OK);
    expect(has_old_mail).toBe(true);

    const [old_c, new_c] = await getVerifyCode(org_id, mail_a_old + "\0" + mail_a_new);
    expect(old_c).toBeDefined();
    expect(new_c).toBeDefined();

    // 错误验证码
    const err_change_buf = await change.call(ctx, user_a_id, mail_a_new, "wrongcode", old_c);
    expect(ChangeD(err_change_buf)[0]).toBe(ERR_VERIFY_CODE);

    // 正确验证码确认修改
    const ok_change_buf = await change.call(ctx, user_a_id, mail_a_new, new_c, old_c);
    expect(ChangeD(ok_change_buf)[0]).toBe(OK);

    // 验证更新
    expect(await userMail(org_id, user_a_id)).toBe(mail_a_new);
    expect(await orgUserAccountLi(org_id, [user_a_id])).toEqual([[MAIL, mail_a_new]]);

    // 修改后用老邮箱登录应当失败返回 0
    const signin_old_after = await signin.call(ctx, mail_a_old, "pwd");
    expect(SigninD(signin_old_after)[0]).toBe(0);

    // 修改后用新邮箱登录应当成功返回 user_a_id
    const signin_new = await signin.call(ctx, mail_a_new, "pwd");
    expect(SigninD(signin_new)[0]).toBe(user_a_id);
  });

  it("changeApply & change without old mail", async () => {
    const apply_buf = await changeApply.call(ctx, user_b_id, mail_b),
      [apply_state, has_old_mail] = ChangeApplyD(apply_buf);

    expect(apply_state).toBe(OK);
    expect(has_old_mail).toBe(false);

    const [new_c] = await getVerifyCode(org_id, "\0" + mail_b);
    expect(new_c).toBeDefined();

    const ok_change_buf = await change.call(ctx, user_b_id, mail_b, new_c, "");
    expect(ChangeD(ok_change_buf)[0]).toBe(OK);
    expect(await userMail(org_id, user_b_id)).toBe(mail_b);
    expect(await orgUserAccountLi(org_id, [user_b_id])).toEqual([[MAIL, mail_b]]);
  });

  it("error cases", async () => {
    // 1. 无效邮箱格式
    const invalid_apply = await changeApply.call(ctx, user_a_id, "invalidmail");
    expect(ChangeApplyD(invalid_apply)[0]).toBe(ERR_MAIL_INVALID);

    const invalid_change = await change.call(ctx, user_a_id, "invalidmail", "code", "code");
    expect(ChangeD(invalid_change)[0]).toBe(ERR_MAIL_INVALID);

    // 2. 邮箱已被其他用户使用 (user_a 已占用 mail_a_new)
    const exist_apply = await changeApply.call(ctx, user_b_id, mail_a_new);
    expect(ChangeApplyD(exist_apply)[0]).toBe(ERR_MAIL_EXIST);

    const exist_change = await change.call(ctx, user_b_id, mail_a_new, "code", "code");
    expect(ChangeD(exist_change)[0]).toBe(ERR_MAIL_EXIST);

    // 3. 未鉴权用户
    const unauth_apply = await changeApply.call(ctx, 99999, `new9999_${ts}@test.com`);
    expect(ChangeApplyD(unauth_apply)[0]).toBe(ERR_AUTH);

    const unauth_change = await change.call(ctx, 99999, `new9999_${ts}@test.com`, "code", "code");
    expect(ChangeD(unauth_change)[0]).toBe(ERR_AUTH);
  });

  it("user signup -> change -> old mail signin fails, new mail signin succeeds", async () => {
    const mail = `xtco3o_${ts}@gmail.com`,
      new_mail = `new_xtco3o_${ts}@gmail.com`,
      pwd = "zspckatf4s",
      name = "Xtco3o";

    await signUpVerifyNew(org_id, "test.com", "zh", mail);
    const code_buf = await KV.getBuffer(keyVerify(1)(org_id, mail));
    expect(code_buf).toBeDefined();

    const signup_buf = await userNew.call(ctx, mail, name, pwd, b36e(code_buf)),
      [uid, state] = UserNewD(signup_buf);
    expect(uid).toBeGreaterThan(0);
    expect(state || 0).toBe(0);

    // 初始邮箱存在，新邮箱不存在
    const info_old_before = await info.call(ctx, mail);
    expect(InfoD(info_old_before)[1]).toBe(true);
    const info_new_before = await info.call(ctx, new_mail);
    expect(InfoD(info_new_before)[1]).toBe(false);

    // 初始邮箱登录成功
    const signin_init = await signin.call(ctx, mail, pwd);
    expect(SigninD(signin_init)[0]).toBe(uid);

    // 修改邮箱
    await changeApply.call(ctx, uid, new_mail);
    const [old_c, new_c] = await getVerifyCode(org_id, mail + "\0" + new_mail);
    expect(old_c).toBeDefined();
    expect(new_c).toBeDefined();

    const change_res = await change.call(ctx, uid, new_mail, new_c, old_c);
    expect(ChangeD(change_res)[0]).toBe(OK);

    // 修改后：老邮箱 info 为不存在，新邮箱 info 为存在
    const info_old_after = await info.call(ctx, mail);
    expect(InfoD(info_old_after)[1]).toBe(false);
    const info_new_after = await info.call(ctx, new_mail);
    expect(InfoD(info_new_after)[1]).toBe(true);

    // 老邮箱登录返回 0（失败）
    const signin_old = await signin.call(ctx, mail, pwd);
    expect(SigninD(signin_old)[0]).toBe(0);

    // 新邮箱登录返回 uid（成功）
    const signin_new = await signin.call(ctx, new_mail, pwd);
    expect(SigninD(signin_new)[0]).toBe(uid);
  });
});
