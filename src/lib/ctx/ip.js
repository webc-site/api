import ipBin from "@3-/ip/ipBin.js";

export default (c) => {
  const h = (k) => c.req.header(k),
    s = h("x-real-ip") || (h("x-forwarded-for") || "").split(",")[0].trim();
  if (s) {
    try {
      return ipBin(s);
    } catch {}
  }
  return new Uint8Array();
};
