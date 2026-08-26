import { pbkdf2, randomBytes, timingSafeEqual } from "node:crypto";
import { promisify } from "node:util";

const pbkdf2Async = promisify(pbkdf2),
  pbkdf2Hash = (password, salt) => pbkdf2Async(password, salt, 99999, 32, "sha512");

export const passwordHash = async (password) => {
    const salt = randomBytes(16),
      hash = await pbkdf2Hash(password, salt);
    return Buffer.concat([salt, hash]);
  },
  passwordVerify = async (password, hashed) => {
    if (!hashed) return false;
    const u8 = new Uint8Array(hashed);
    if (u8.length !== 48) return false;

    const salt = u8.subarray(0, 16),
      expected_hash = u8.subarray(16),
      hash = await pbkdf2Hash(password, salt);
    return timingSafeEqual(hash, expected_hash);
  };
