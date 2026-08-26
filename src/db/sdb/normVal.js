const normVal = (val) => {
  if (val == null) return val;
  if (val instanceof Uint8Array) {
    return Buffer.isBuffer(val) ? val : Buffer.from(val.buffer, val.byteOffset, val.byteLength);
  }
  if (Array.isArray(val)) return val.map(normVal);
  if (val.constructor === Object) {
    const out = {};
    for (const [k, v] of Object.entries(val)) out[k] = normVal(v);
    return out;
  }
  return val;
};

export default normVal;
