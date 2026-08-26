import { readdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import read from "@3-/read";
import write from "@3-/write";
import ETA from "../../src/lib/md/ETA.js";

const GEN_HEAD = "// GEN BY tran.js\n",
  compileMd = (md) => {
    const part_li = [];
    for (const item of ETA.parse(md.trim())) {
      if (typeof item === "string") part_li.push("'" + item + "'");
      else if (item.val) part_li.push(item.val.trim());
    }
    return (
      GEN_HEAD + "export default (it) => " + (part_li.length ? part_li.join(" + ") : '""') + ";\n"
    );
  };

export default (i18n_dir) => {
  const dir_li = readdirSync(i18n_dir, { withFileTypes: true })
    .filter((f) => f.isDirectory() && !f.name.startsWith("."))
    .map((f) => join(i18n_dir, f.name));

  for (const dir of dir_li) {
    const file_li = readdirSync(dir);

    for (const file of file_li) {
      if (file.endsWith(".js")) rmSync(join(dir, file), { force: true });
    }

    for (const file of file_li) {
      if (file.endsWith(".md")) {
        write(join(dir, file.slice(0, -3) + ".js"), compileMd(read(join(dir, file))));
      }
    }
  }
};
