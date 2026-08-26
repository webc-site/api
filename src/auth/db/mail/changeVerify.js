import b36d from "@3-/b36/b36d.js";
import split from "@3-/split";
import KV from "../../../db/KV.js";
import { keyMailChange } from "./key.js";

export default async (org_id, old_mail, new_mail, new_code, old_code) => {
  new_code = new_code.trim().toLowerCase();
  if (!new_code || (old_mail && !old_code)) return false;

  let bin;
  try {
    bin = old_mail
      ? Buffer.concat([b36d(old_code.trim().toLowerCase()), b36d(new_code)])
      : b36d(new_code);
  } catch {
    return false;
  }

  const key = keyMailChange(org_id, bin),
    saved = await KV.get(key);

  if (!saved) return false;

  const [saved_old, saved_new] = split(saved, "\0");

  if (saved_new !== new_mail || (saved_old || "") !== (old_mail || "")) {
    return false;
  }

  await KV.del(key);
  return true;
};
