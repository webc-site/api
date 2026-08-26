import int from "@3-/int";

// [total_length, prefix_match, number_offset, country_code]
const MATCH_LI = [
  [10, "", 0, 1], // US/CA 10-digit (e.g. 2025550123)
  [11, "12", 1, 1], // US/CA 11-digit with prefix 1 (e.g. 12025550123)
  [11, "1", 0, 86], // CN 11-digit without country code (e.g. 13800138000)
  [13, "86", 2, 86] // CN 13-digit with prefix 86 (e.g. 8613800138000)
];

export default (account) => {
  const li = [];
  let s = "";
  for (const c of account) {
    if (c >= "0" && c <= "9") {
      s += c;
    } else if (s) {
      li.push(s);
      s = "";
    }
  }
  if (s) li.push(s);

  if (li.length === 2) {
    return [int(li[0]), int(li[1])];
  }

  s = li.join("");
  if (!s) return [0, 0];

  for (const [len, prefix, offset, country_code] of MATCH_LI) {
    if (s.length === len && s.startsWith(prefix)) {
      return [country_code, int(s.slice(offset))];
    }
  }

  if (li.length > 1) {
    return [int(li[0]), int(li.slice(1).join(""))];
  }

  return [0, int(s)];
};
