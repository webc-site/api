import int from "@3-/int";
import { addExtension, encode, decode } from "cbor-x";
import RecordId from "./RecordId.js";

const geom = (type) => (coordinates) => [type, coordinates];

[
  [0, (v) => new Date(v)],
  [6, () => undefined, () => null],
  [7],
  [
    8,
    (v) => (typeof v === "string" ? RecordId(v) : RecordId(v[0] + ":" + v[1])),
    (v) => [v.tb, v.id]
  ],
  [9],
  [10],
  [
    12,
    ([s, n = 0]) => new Date(int(s) * 1000 + int(n / 1e6)),
    (d) => [int(d.getTime() / 1000), (d.getTime() % 1000) * 1e6],
    Date
  ],
  [13],
  [14, ([s, n = 0]) => int(s) + Number(n) / 1e9],
  [15],
  [37],
  [49],
  [50, (v) => [v, true]],
  [51, (v) => [v, false]],
  [55],
  [56, (v) => new Set(v), (v) => [...v], Set],
  [88, geom("Point")],
  [89, geom("LineString")],
  [90, geom("Polygon")],
  [91, geom("MultiPoint")],
  [92, geom("MultiLineString")],
  [93, geom("MultiPolygon")],
  [94, geom("GeometryCollection")]
].forEach(([tag, decode = (v) => v, encode, Class]) => {
  const ext = { tag, decode };
  if (Class) ext.Class = Class;
  if (encode) ext.encode = (v, enc) => enc(encode(v));
  addExtension(ext);
});

export const APPLICATION_CBOR = "application/cbor";
export { encode, decode };
