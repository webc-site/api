export const INT = "int",
  STRING = "string",
  BYTES = "bytes",
  BOOL = "bool",
  rec = (t, is_null) => "record<" + t + ">" + (is_null ? " | null" : "");

const idxSql = (table, li, is_unique) => {
  if (!li) return [];
  if (!Array.isArray(li)) li = [li];
  return li.map((f) => {
    const f_li = Array.isArray(f) ? f : [f];
    return (
      "DEFINE INDEX IF NOT EXISTS " +
      f_li.join("_") +
      " ON " +
      table +
      " FIELDS " +
      f_li.join(",") +
      (is_unique ? " UNIQUE;" : ";")
    );
  });
};

export default (table_map) =>
  Object.entries(table_map)
    .flatMap(([table, { auto_id, autoId, field, unique, index }]) => {
      const sql_li = [],
        auto = auto_id || autoId;
      if (auto) {
        const seq_name = typeof auto === "string" ? auto : table;
        sql_li.push(
          "DEFINE SEQUENCE IF NOT EXISTS " + seq_name + " BATCH 1000 START 1;",
          "DEFINE TABLE IF NOT EXISTS " + table + " SCHEMAFULL;",
          "DEFINE FIELD IF NOT EXISTS id ON " +
            table +
            " DEFAULT type::record('" +
            table +
            "',`sequence`::nextval('" +
            seq_name +
            "'));"
        );
      } else {
        sql_li.push("DEFINE TABLE IF NOT EXISTS " + table + " SCHEMAFULL;");
      }
      sql_li.push(
        ...field.map(
          ([k, v]) => "DEFINE FIELD IF NOT EXISTS " + k + " ON " + table + " TYPE " + v + ";"
        ),
        ...idxSql(table, unique, true),
        ...idxSql(table, index, false)
      );
      return sql_li;
    })
    .join("");
