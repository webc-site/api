import sec from "@3-/time/sec.js";
import { beforeAll, describe, expect, it } from "bun:test";
import orgDb from "../../db/orgDb.js";
import reqCtx from "../../lib/srv/ctx.js";
import orgInit from "../db/org/dbInit.js";
import orgUser from "../db/org/orgUser.js";
import { KIND_TEAM, KIND_USER } from "../db/token/KIND.js";
import tokenLsDb from "../db/token/ls.js";
import tokenNewDb from "../db/token/new.js";
import { bidUserAccountSet } from "../db/user/bid.js";
import TokenLsD from "../gen/token/LsD.js";
import TokenNewD from "../gen/token/NewD.js";
import TokenRefreshD from "../gen/token/RefreshD.js";
import tokenDisable from "../url/token/disable.js";
import tokenEnable from "../url/token/enable.js";
import tokenLs from "../url/token/ls.js";
import tokenNameSet from "../url/token/nameSet.js";
import tokenNewUrl from "../url/token/new.js";
import tokenRefresh from "../url/token/refresh.js";
import tokenRm from "../url/token/rm.js";

describe("token db & tokenLs url", () => {
  const org_id = 1,
    OTHER_UID = 999999,
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
    }),
    ls = async (id = uid) => TokenLsD(await tokenLs.call(ctx, id))[0];

  let uid;

  beforeAll(async () => {
    await orgInit(org_id);
    uid = await orgUser(orgDb(org_id), org_id, 1, "test_user");
    await bidUserAccountSet(org_id, bid, uid, 1, "test@test.com", ip, ua);
  });

  it("tokenLs when empty auto creates token", async () => {
    const token_li = await ls();

    expect(token_li.length).toBe(1);
    const [token_id, [token, name, enable, ts]] = token_li[0];
    expect(typeof token_id).toBe("number");
    expect(token instanceof Uint8Array).toBe(true);
    expect(token.length).toBe(16);
    expect(name).toBe("");
    expect(enable).toBe(true);
    expect(typeof ts).toBe("number");

    // 再次查询应从 KV 查出且列表一致
    const cached_li = await ls();
    expect(cached_li.length).toBe(1);
    expect(cached_li[0][0]).toBe(token_id);
    expect(cached_li[0][1][1]).toBe("");
  });

  it("tokenNew creates new token and updates kv", async () => {
    const [, [new_token, new_name, new_enable]] = await tokenNewDb(
      org_id,
      KIND_USER,
      uid,
      "my_token"
    );

    expect(new_name).toBe("my_token");
    expect(new_enable).toBe(true);
    expect(new_token.length).toBe(16);

    const li = await tokenLsDb(org_id, uid);
    expect(li.length).toBe(2);
  });

  it("tokenNewUrl creates token via url handler with auth", async () => {
    // 越权新建：无权限用户返回空
    const [unauth_id] = TokenNewD(await tokenNewUrl.call(ctx, OTHER_UID, "unauth_token"));
    expect(unauth_id).toBe(0);

    // 正常新建
    const [t_id, [t_bytes, t_name, t_enable, t_ts]] = TokenNewD(
      await tokenNewUrl.call(ctx, uid, "url_token")
    );
    expect(typeof t_id).toBe("number");
    expect(t_id).toBeGreaterThan(0);
    expect(t_name).toBe("url_token");
    expect(t_enable).toBe(true);
    expect(t_bytes.length).toBe(16);
    expect(typeof t_ts).toBe("number");
  });

  it("tokenRefresh regenerates token bytes and updates ts", async () => {
    const [target_token_id, [old_token, , , old_ts]] = (await ls())[0];

    // 越权刷新：无权限用户返回空
    const [unauth_token, unauth_ts] = TokenRefreshD(
      await tokenRefresh.call(ctx, OTHER_UID, target_token_id)
    );
    expect(unauth_token.length).toBe(0);
    expect(unauth_ts).toBe(0);

    // 正常刷新
    const [new_token, new_ts] = TokenRefreshD(await tokenRefresh.call(ctx, uid, target_token_id));
    expect(new_token.length).toBe(16);
    expect(new_token).not.toEqual(old_token);
    expect(new_ts).toBeGreaterThanOrEqual(old_ts);

    // 验证 tokenLs 获取到的 token 已更新
    const after_li = await ls(),
      refreshed_item = after_li.find(([id]) => id === target_token_id);
    expect(refreshed_item).toBeDefined();
    expect(refreshed_item[1][0]).toEqual(new_token);
    expect(refreshed_item[1][3]).toBe(new_ts);
  });

  it("tokenLs returns empty without auth", async () => {
    const token_li = await ls(OTHER_UID);
    expect(token_li.length).toBe(0);
  });

  it("team table shares autoId sequence with user and supports token rel", async () => {
    const db = orgDb(org_id),
      [{ id: team_rec }] = await db(
        "CREATE ONLY team SET name=$name,owner=type::record('user',$owner),ts=$ts;",
        {
          name: "TestTeam",
          owner: uid,
          ts: sec()
        }
      );

    expect(typeof team_rec.id).toBe("number");
    expect(team_rec.id).toBeGreaterThan(uid);

    // 验证通过 tokenNew 为 team 创建 token
    const [token_id, [team_token, team_tname, team_enable]] = await tokenNewDb(
      org_id,
      KIND_TEAM,
      team_rec.id,
      "team_token"
    );

    expect(typeof token_id).toBe("number");
    expect(team_token.length).toBe(16);
    expect(team_tname).toBe("team_token");
    expect(team_enable).toBe(true);

    const [t_rec] = await db("SELECT rel FROM ONLY type::record('token',$id)", {
      id: token_id
    });
    expect(t_rec.rel.tb).toBe("team");
    expect(t_rec.rel.id).toBe(team_rec.id);
  });

  it("tokenDisable and tokenEnable verify auth and update kv & db", async () => {
    const [target_token_id] = (await ls())[0];

    // 禁用该 token
    await tokenDisable.call(ctx, uid, target_token_id);

    // 列表显示 enable 为 false
    const disabled_li = await ls(),
      target_item = disabled_li.find(([id]) => id === target_token_id);
    expect(target_item[1][2]).toBe(false);

    // 重新启用
    await tokenEnable.call(ctx, uid, target_token_id);

    // 列表显示 enable 为 true
    const enabled_li = await ls(),
      enabled_item = enabled_li.find(([id]) => id === target_token_id);
    expect(enabled_item[1][2]).toBe(true);

    // 越权测试：未授权用户尝试禁用无效
    await tokenDisable.call(ctx, OTHER_UID, target_token_id);
    const unauth_li = await ls(),
      unauth_item = unauth_li.find(([id]) => id === target_token_id);
    expect(unauth_item[1][2]).toBe(true);
  });

  it("tokenRm deletes token from db and kv with auth verification", async () => {
    const token_li = await ls(),
      before_count = token_li.length,
      [target_token_id] = token_li[0];

    // 越权删除：无权限用户操作无效
    await tokenRm.call(ctx, OTHER_UID, target_token_id);
    const unauth_li = await ls();
    expect(unauth_li.length).toBe(before_count);

    // 授权用户正常删除
    await tokenRm.call(ctx, uid, target_token_id);
    const rm_li = await ls();
    expect(rm_li.length).toBe(before_count - 1);
    expect(rm_li.find(([id]) => id === target_token_id)).toBeUndefined();
  });

  it("tokenNameSet updates name in db and kv with auth verification", async () => {
    const [target_token_id] = (await ls())[0];

    // 越权修改：无权限用户操作无效
    await tokenNameSet.call(ctx, OTHER_UID, target_token_id, "unauth_rename");
    const unauth_li = await ls(),
      unauth_item = unauth_li.find(([id]) => id === target_token_id);
    expect(unauth_item[1][1]).not.toBe("unauth_rename");

    // 授权用户正常修改名称（超长自动截断为 24 字符）
    await tokenNameSet.call(ctx, uid, target_token_id, "123456789012345678901234567890");
    const renamed_li = await ls(),
      renamed_item = renamed_li.find(([id]) => id === target_token_id);
    expect(renamed_item[1][1]).toBe("123456789012345678901234");
  });
});
