#!/usr/bin/env bun

import proto2js from "@1-/proto2js";
import merge from "@1-/proto2js/merge.js";
import { parse } from "proto-parser";
import read from "@3-/read";
import write from "@3-/write";
import { $ } from "@3-/zx";
import { cpSync, existsSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import genJs from "./sh/gen/js.js";
import srcPkg from "./sh/srcPkg.js";

const ROOT = import.meta.dirname,
  GEN_HEAD = "// GEN BY gen.js\n",
  toCamel = (name) => name.replace(/_([a-z])/g, (_, c) => c.toUpperCase()),
  toSnake = (name) => name.replace(/[A-Z]/g, (c) => "_" + c.toLowerCase()),
  cap = (name) => name[0].toUpperCase() + name.slice(1),
  walkProto = (dir, prefix = "") => {
    const res = [];
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.name.startsWith("_") || entry.name.startsWith(".")) continue;
      const rel = prefix ? prefix + "/" + entry.name : entry.name,
        full = join(dir, entry.name);
      if (entry.isDirectory()) {
        res.push(...walkProto(full, rel));
      } else if (entry.name.endsWith(".proto")) {
        res.push(rel);
      }
    }
    return res;
  },
  findEnums = (node) => {
    const li = [],
      scan = (obj) => {
        if (!obj) return;
        for (const [k, v] of Object.entries(obj)) {
          if (v.syntaxType === "EnumDefinition") {
            li.push(k);
          } else if (v.nested) {
            scan(v.nested);
          }
        }
      };
    scan(node);
    return li;
  },
  genPkg = (pkg, api_dir, src_dir, gen_tmp_dir) => {
    const url_dir = join(src_dir, pkg, "url");
    if (!existsSync(url_dir)) return false;

    const file_li = walkProto(url_dir).sort();
    if (!file_li.length) return false;

    const imp_li = [],
      field_li = [],
      url_li = [];

    file_li.forEach((file, i) => {
      const url_path = file.slice(0, -6),
        parts = url_path.split("/"),
        base_name = parts.at(-1),
        cap_name = cap(base_name),
        sub_pkg = parts.slice(0, -1),
        snake_name = toSnake(parts.join("_")),
        txt = read(join(url_dir, file)),
        has_req = new RegExp("\\bmessage\\s+" + cap_name + "Req\\b").test(txt),
        req_type = has_req ? [pkg, ...sub_pkg, cap_name + "Req"].join(".") : "Empty",
        tag = i + 1;

      imp_li.push('import "' + file + '";');
      field_li.push("    " + req_type + " " + snake_name + "=" + tag + ";");
      url_li.push(url_path);
    });

    const call_proto =
        'syntax="proto3";\n\npackage ' +
        pkg +
        ";\n\n" +
        imp_li.join("\n") +
        "\n\nmessage Empty {}\n\nmessage Call {\n  oneof req {\n" +
        field_li.join("\n") +
        "\n  }\n}\n",
      tmp_proto_file = join(gen_tmp_dir, pkg + ".proto"),
      pkg_src_gen_dir = join(src_dir, pkg, "gen"),
      pkg_gen_tmp_dir = join(gen_tmp_dir, pkg),
      inc_dir = [url_dir, join(src_dir, pkg)];

    write(tmp_proto_file, call_proto);
    rmSync(pkg_src_gen_dir, { recursive: true, force: true });
    proto2js(tmp_proto_file, gen_tmp_dir, inc_dir);

    const [proto_src] = merge([gen_tmp_dir, ...inc_dir], pkg + ".proto"),
      parsed = parse(proto_src),
      call = parsed.root?.nested?.[pkg]?.nested?.Call || parsed.root?.nested?.Call;

    if (!call?.fields) return false;

    cpSync(pkg_gen_tmp_dir, pkg_src_gen_dir, { recursive: true, force: true });

    const enum_li = findEnums(parsed.root?.nested?.[pkg]?.nested || parsed.root?.nested);

    genJs(api_dir, pkg, pkg_src_gen_dir, call.fields, toCamel, GEN_HEAD, enum_li, url_li);

    const url_code = GEN_HEAD + "export default " + JSON.stringify(url_li) + ";\n";
    write(join(src_dir, pkg, "url.js"), url_code);

    return true;
  },
  main = async () => {
    const src_dir = join(ROOT, "src"),
      gen_tmp_dir = join(src_dir, ".gen"),
      api_dir = join(ROOT, "api", "js");

    rmSync(join(ROOT, "gen"), { recursive: true, force: true });
    rmSync(join(src_dir, "gen"), { recursive: true, force: true });
    rmSync(gen_tmp_dir, { recursive: true, force: true });

    const pkg_li = [];
    for (const pkg of srcPkg(src_dir)) {
      if (genPkg(pkg, api_dir, src_dir, gen_tmp_dir)) {
        pkg_li.push(pkg);
      }
    }

    if (pkg_li.length) {
      const imp_li = [],
        entry_li = [];

      for (const pkg of pkg_li) {
        imp_li.push("import " + pkg + '_url from "./' + pkg + '/url.js";');
        imp_li.push("import " + pkg + 'CallD from "./' + pkg + '/gen/CallD.js";');
        entry_li.push("  " + pkg + ": [" + pkg + "_url," + pkg + "CallD],");
      }

      const src_url_code =
        GEN_HEAD + imp_li.join("\n") + "\n\nexport default {\n" + entry_li.join("\n") + "\n};\n";
      write(join(src_dir, "url.js"), src_url_code);
    }

    rmSync(gen_tmp_dir, { recursive: true, force: true });

    await $({ quiet: true })`bun x oxfmt ${api_dir} ${src_dir}`;
  };

export default main;

if (import.meta.main) {
  await main();
}
