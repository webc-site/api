import b36d from "@3-/b36/b36d.js";
import b36e from "@3-/b36/b36e.js";
import u8eq from "@3-/u8/u8eq.js";
import KV from "../../../db/KV.js";

const TTL = 86400,
  LEN = 9;

export const codeByKey = async (key, len = LEN) => {
  const [[, exist], [, ttl]] = await KV.pipeline().getBuffer(key).ttl(key).exec();
  if (exist) {
    if (TTL - ttl >= 59) {
      await KV.expire(key, TTL);
      return b36e(exist);
    }
    return;
  }

  const code = b36e(crypto.getRandomValues(new Uint8Array(len)));
  await KV.set(key, b36d(code), "EX", TTL);
  return code;
};

export const codeVerify = async (key, verify_code) => {
  let bin;
  try {
    bin = b36d(verify_code.toLowerCase());
  } catch {
    return false;
  }

  const saved = await KV.getBuffer(key);
  if (saved && u8eq(saved, bin)) {
    await KV.del(key);
    return true;
  }
  return false;
};
