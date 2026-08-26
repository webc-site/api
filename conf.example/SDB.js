// SurrealDB HTTP 连接地址
// HTTP 协议完全无状态，通过在请求头中指定 Surreal-DB 即可零开销无缝切库
export const URI = "http://127.0.0.1:9050",
  DB_CONF = {
    username: "i",
    password: "TODO",
    namespace: process.env.NODE_ENV || "dev",
    database: "i"
  };
