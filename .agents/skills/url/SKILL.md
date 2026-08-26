---
name: url
---

## 接口开发流程

1. proto 与接口定义

- 接口与 proto：在 `src/${pkg}/url/${函数名}.proto` 中定义响应消息、状态枚举（按需）及请求消息（仅有入参时定义 `${名称}Req`）
- 业务函数实现：在 `src/${pkg}/url/${函数名}.js` 中编写业务逻辑函数

2. 代码生成

- 运行 `./gen.js` 自动基于 `src/${pkg}/url/*.proto` 生成：
  - `src/${pkg}/gen/` 编解码器
  - `src/${pkg}/url.js` 与 `src/url.js` 路由索引
  - `api/js/${pkg}/` 客户端请求函数与 TypeScript 类型定义
  - `api/js/${pkg}/enum/` 客户端枚举常量

3. 服务端实现

- 在 `src/${pkg}/url/${函数名}.js` 中编写业务逻辑函数（`url.js` 路由映射由 `gen.js` 自动生成，无需手动维护）
- 执行上下文：函数内 `this` 指向请求上下文 Proxy（`reqCtx`），可自动惰性求值并缓存：
  - `this.host`：当前请求的主机域名
  - `this.lang`：请求首选语言
  - `this.org_id`：当前域名所属组织 ID（Promise）
- 无状态数据库操作：
  - 组织数据库完全无状态（基于自研 HTTP 驱动），切换库无任何握手开销
  - 组织级数据库函数接收 `db` 查询函数（如 `(db,org_id,user_id,level,name,conf)`），业务接口中通过 `orgDb(org_id)`（引入 `db/orgDb.js`）传入
- 返回值：
  - 必须使用生成的响应编码器 `${名称}E([...])` 编码为 Protobuf 二进制（`Uint8Array`）返回，严禁返回 `undefined`、未编码的裸对象或遗漏 `return`。
  - 空响应/无数据返回的接口，也必须返回空包编码（如 `ExitE([])` 或 `EmptyE([])`）。
  - 若返回 `undefined`，网关在 `resChunk` 计算长度时会直接抛错导致服务异常。
  - 避免写很多 `return`：多分支业务逻辑中，避免在各分支反复写 `return ${名称}E(...)`，建议先声明 `let r` 记录结果分支，在函数末尾统一调用编码器返回 `return ${名称}E(r)`。
  - 示例：
    - 无数据接口：`return ExitE([])`
    - 单一返回：`return GetE([auth_type_li,user_li])`
    - 多分支统一编码示例：

      ```js
      const [prefix, host_name] = split(mail);
      let r;
      if (!prefix || !host_name) {
        r = [ERR_MAIL_INVALID];
      } else {
        mail = prefix + "@" + host_name;
        const { org_id, host, lang, bid } = this,
          org = await org_id;

        if (!(await bidUserHas(org, bid, uid))) {
          r = [ERR_AUTH];
        } else {
          const exist_uid = await mailOrgUser(org, mail);
          if (exist_uid && exist_uid !== uid) {
            r = [ERR_MAIL_EXIST];
          } else {
            const old_mail = await userMail(org, uid);
            r =
              old_mail === mail
                ? [ERR_MAIL_EXIST]
                : [OK, await changeApply(org, host, lang, old_mail, mail)];
          }
        }
      }
      return MailChangeApplyE(r);
      ```

4. 验证测试

- 编写测试用例并运行 `./test.sh`

## Proto 规范与命名规则

- 文件归属与命名：每个接口对应一个 proto 文件，位于 `src/${pkg}/url/${函数名}.proto`，文件名为小驼峰（如 `userNewByMail.proto`、`get.proto`）
- 基础名称：动词与名词组合，大驼峰风格（如 `UserNewByMail`、`Get`）
- 请求消息：
  - 有入参：定义 `${名称}Req`（如 `UserNewByMailReq`、`InfoReq`）
  - 无入参：业务 proto 中无需声明任何 Req 消息，生成器会自动关联公共 `Empty` 并生成 0 参数的客户端函数
- 状态枚举：`${名称}State`（如 `UserNewByMailState`，首项为 `OK=0`，错误项如 `ERR_MAIL_EXIST=1`）；若无需校验参数或无业务状态分支，可省略状态枚举
- 响应消息：`${名称}`（如 `UserNewByMail`、`Get`，包含数据字段与可选的 `optional ${名称}State state`；无状态枚举时无需包含 `state` 字段）
- **避免使用 `optional`**：返回值消息字段避免写 `optional`，未查到或无数据时直接使用对应类型的零值（如 `id` 没有用 `0`、字符串用 `""`、布尔用 `false`）

### 示例

`src/auth/url/get.proto`（无入参示例）：

```proto
syntax="proto3";

package auth;

enum AuthType {
  PHONE=1;
  GOOGLE=2;
  APPLE=3;
  MICROSOFT=4;
  WECHAT=5;
  GITHUB=6;
}

message User {
  uint64 id=1;
  string name=2;
}

message Get {
  repeated AuthType auth_type_li=1;
  repeated User user_li=2;
}
```

`src/auth/url/userNewByMail.proto`（有入参示例）：

```proto
syntax="proto3";

package auth;

message UserNewByMailReq {
  string mail=1;
  string name=2;
  string password=3;
  string verify_code=4;
}

enum UserNewByMailState {
  OK=0;
  ERR_MAIL_EXIST=1;
  ERR_VERIFY_CODE=2;
}

message UserNewByMail {
  uint64 uid=1;
  optional UserNewByMailState state=2;
}
```

## 映射对应关系

- tag 序号与 url.js：`gen.js` 自动根据 `src/${pkg}/url/*.proto` 字母序分配 tag 并映射到 `src/${pkg}/url.js` 数组下标（`tag - 1`）
- 命名映射：
  - proto 文件名 `camelCase`（如 `userNewByMail.proto`、`get.proto`）
  - 函数实现文件名 `camelCase`（如 `src/${pkg}/url/userNewByMail.js`、`src/${pkg}/url/get.js`）
  - 客户端请求函数 `camelCase`（如 `api/js/${pkg}/userNewByMail.js`、`api/js/${pkg}/get.js`）
- 代码生成产物：
  - `src/${pkg}/gen/${名称}ReqE.js` / `src/${pkg}/gen/EmptyE.js`（请求编码）
  - `src/${pkg}/gen/${名称}D.js`（响应解码）
  - `api/js/${pkg}/${函数名}.js`（客户端请求函数，如 `req("${pkg}")(tag,ReqE,ResD)`）
  - `api/js/${pkg}/${函数名}.d.ts`（客户端类型定义）
  - `api/js/${pkg}/enum/${枚举名}.js`（客户端枚举定义）

## 数据库建表与表结构更新（SurrealDB）

项目数据库采用 SurrealDB。

### 1. 配置与建表定义

在 `docker/sdb/sdb.surql` 中编写 SurrealQL 语句（按字母序维护）：

- 自增序列：`DEFINE SEQUENCE ${表名} BATCH 1000 START 1;`
- 严格表模式：`DEFINE TABLE ${表名} SCHEMAFULL;`
- 字段与默认主键：
  - 主键：`DEFINE FIELD id ON ${表名} DEFAULT type::record('${表名}',\`sequence\`::nextval('${表名}'));`
  - 记录/实体引用：类型为 `record<目标表>` 时直接以实体/表名命名，不加 `_id` 后缀（如 `DEFINE FIELD host ON ${表名} TYPE record<host>;`、`DEFINE FIELD org ON host TYPE record<org>;`、`DEFINE FIELD user ON orgUser TYPE record<user>;`），保持图导航语法自然、极简
  - 普通类型：`DEFINE FIELD ${字段名} ON ${表名} TYPE int | string | bytes | bool;`
- 索引：`DEFINE INDEX ${索引名} ON ${表名} FIELDS ${字段1},${字段2} UNIQUE;`

### 2. 更新表结构

修改 `docker/sdb/sdb.surql` 后，运行更新脚本增量应用变更：

```bash
bun ./docker/sdb/updateSchema.js
```

- 该脚本会逐条执行 `sdb.surql` 中的语句，自动忽略已存在的表与索引（`already exists`）。
- 全量初始化数据库使用 `bun ./docker/sdb/init.js`（仅负责连接并执行 `sdb.surql` 初始化）。
- 重置数据库与表结构：运行 `./docker/reset.sh` 彻底重置 Docker 容器、清空数据并重新初始化。
