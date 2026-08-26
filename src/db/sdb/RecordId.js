const toString = function () {
    return this.tb + ":" + this.id;
  },
  proto = {
    toString,
    toJSON: toString,
    [Symbol.toPrimitive]: toString
  };

export default (str) => {
  const idx = str.indexOf(":"),
    raw = str.slice(idx + 1);
  return {
    __proto__: proto,
    tb: str.slice(0, idx),
    id: Number(raw) || raw
  };
};
