import { beforeAll, describe, expect, it } from "bun:test";
import orgDb from "../../db/orgDb.js";
import reqCtx from "../../lib/srv/ctx.js";
import mailNew from "../db/mail/new.js";
import orgInit from "../db/org/dbInit.js";
import orgUser from "../db/org/orgUser.js";
import phoneNew from "../db/phone/new.js";
import phoneOrgUser from "../db/phone/orgUser.js";
import split from "../db/phone/split.js";
import userPhone from "../db/phone/userPhone.js";
import phoneSet from "../db/phone/userPhoneSet.js";
import { MAIL, PHONE } from "../gen/AccountType.js";
import InfoD from "../gen/InfoD.js";
import info from "../url/info.js";

describe("auth info", () => {
  const org_id = 1,
    ctx = reqCtx({
      org_id: Promise.resolve(org_id),
      host: "test.com",
      lang: "zh"
    });

  beforeAll(async () => {
    await orgInit(org_id);
  });

  it("mail exist check", async () => {
    const mail = "info_test_" + Date.now() + "@test.com",
      res_non_exist = await info.call(ctx, mail),
      [type1, exist1] = InfoD(res_non_exist);

    expect(type1).toBe(MAIL);
    expect(exist1).toBe(false);

    const db = orgDb(org_id),
      mail_id = await mailNew(mail);

    await orgUser(db, org_id, 1, "测试邮箱用户", { mail: mail_id });

    const res_exist = await info.call(ctx, mail),
      [type2, exist2] = InfoD(res_exist);

    expect(type2).toBe(MAIL);
    expect(exist2).toBe(true);
  });

  it("phone exist check", async () => {
    const phone_str = "+86 139" + String(Date.now()).slice(-8),
      [area, num] = split(phone_str),
      res_non_exist = await info.call(ctx, phone_str),
      [type1, exist1] = InfoD(res_non_exist);

    expect(type1).toBe(PHONE);
    expect(exist1).toBe(false);

    // 全局录入 phone 记录，但未在 org 绑定用户
    const phone_id = await phoneNew(area, num),
      res_global_only = await info.call(ctx, phone_str),
      [type2, exist2] = InfoD(res_global_only);

    expect(type2).toBe(PHONE);
    expect(exist2).toBe(false);

    // 组织内创建用户绑定该 phone
    const db = orgDb(org_id);
    const user_id = await orgUser(db, org_id, 1, "测试手机用户", { phone: phone_id });

    const res_exist = await info.call(ctx, phone_str),
      [type3, exist3] = InfoD(res_exist);

    expect(type3).toBe(PHONE);
    expect(exist3).toBe(true);

    // 验证 userPhone 读取
    const fetched_phone = await userPhone(org_id, user_id);
    expect(fetched_phone).toBe(phone_str);

    // 换绑手机
    const new_phone_str = "+86 138" + String(Date.now()).slice(-8),
      [new_area, new_num] = split(new_phone_str);
    await phoneSet(org_id, user_id, new_area, new_num);

    // 验证新手机号读取与组织内反查
    expect(await userPhone(org_id, user_id)).toBe(new_phone_str);
    expect(await phoneOrgUser(org_id, new_area, new_num)).toBe(user_id);

    // 验证旧手机号已解绑，反查不到
    expect(await phoneOrgUser(org_id, area, num)).toBeUndefined();

    // 验证重复设置相同手机号不会误删反查索引
    await phoneSet(org_id, user_id, new_area, new_num);
    expect(await userPhone(org_id, user_id)).toBe(new_phone_str);
    expect(await phoneOrgUser(org_id, new_area, new_num)).toBe(user_id);
  });

  it("split phone format check", () => {
    expect(split("+86 13800138000")).toEqual([86, 13800138000]);
    expect(split("86-13800138000")).toEqual([86, 13800138000]);
    expect(split("+1 2025550123")).toEqual([1, 2025550123]);
    expect(split("1-2025550123")).toEqual([1, 2025550123]);
    expect(split("13800138000")).toEqual([86, 13800138000]);
    expect(split("8613800138000")).toEqual([86, 13800138000]);
    expect(split("2025550123")).toEqual([1, 2025550123]);
    expect(split("12025550123")).toEqual([1, 2025550123]);
    expect(split("+86 138 0013 8000")).toEqual([86, 13800138000]);
    expect(split("+44 7911123456")).toEqual([44, 7911123456]);
    expect(split("+44 79 1112 3456")).toEqual([44, 7911123456]);
    expect(split("")).toEqual([0, 0]);
    expect(split("abc")).toEqual([0, 0]);
    expect(split("10086")).toEqual([0, 10086]);
  });
});
