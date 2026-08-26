export default (limit = 1024) => {
  const map = new Map();

  return {
    get: (key) => {
      if (!map.has(key)) return;
      const val = map.get(key);
      map.delete(key);
      map.set(key, val);
      return val;
    },
    set: (key, val) => {
      if (map.has(key)) {
        map.delete(key);
      } else if (map.size >= limit) {
        map.delete(map.keys().next().value);
      }
      map.set(key, val);
    },
    has: map.has.bind(map),
    rm: map.delete.bind(map),
    clear: map.clear.bind(map),
    get size() {
      return map.size;
    }
  };
};
