---
name: kv
---

## 存储与持久化

- 所有数据必须完整存储在 SurrealDB 中 (`src/db/SDB.js` / `src/db/orgDb.js`)
- KVrocks 仅用于提升查询性能 (`src/db/KV.js`)
- 写入时：先写入 SurrealDB，然后同步写入 KVrocks

## 查询原则

- 查询时，只查询 KVrocks，严禁回源查询 SurrealDB 兜底
- 禁止在查询逻辑中编写回源回写等防御式编程代码
- 列表与多键查询，优先使用 `pipeline`、`mget` / `mgetBuffer` 批量获取，减少网络往返

## 编码与性能优化

- 尽量用二进制提升读写与存储性能：
  - 数字与二进制互转：
    ```javascript
    import u64Bin from "@3-/intbin/u64Bin.js";
    import binU64 from "@3-/intbin/binU64.js";
    import u64Buf from "@3-/intbin/u64Buf.js";
    ```
  - 键名中间包含数字 ID 时使用 `u64B255`（b255 编码不含 `:`，避免键冲突）：
    ```javascript
    import u64B255 from "@3-/intbin/u64B255.js";
    ```
