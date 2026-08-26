import signin from "./signin.js";
import query from "./query.js";

const EXPIRY_MARGIN = 60000,
  tokenExp = (token) => {
    const part = token.split(".")[1];
    return part ? JSON.parse(Buffer.from(part, "base64url")).exp * 1000 : 0;
  };

export default async (url, user, pass, namespace) => {
  const rpc_url = url.endsWith("/rpc") ? url : url.replace(/\/+$/, "") + "/rpc",
    ns = user === "root" ? undefined : namespace;

  let token,
    expire_at = 0,
    signing;

  const auth = async (force) => {
    if (!force && token && Date.now() < expire_at) return token;
    if (!signing) {
      signing = (async () => {
        try {
          token = await signin(rpc_url, user, pass, ns);
          expire_at = tokenExp(token) - EXPIRY_MARGIN;
          return token;
        } finally {
          signing = null;
        }
      })();
    }
    return signing;
  };

  await auth();

  return (database) => (sql, params) => query(rpc_url, auth, namespace, database, sql, params);
};
