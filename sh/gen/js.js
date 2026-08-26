import importLi from "@1-/jsparser/importLi.js";
import read from "@3-/read";
import write from "@3-/write";
import { existsSync, readdirSync, rmSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { parse as parseJs } from "yuku-parser";

const findEnumFile = (dir, target) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) {
        const found = findEnumFile(full, target);
        if (found) return found;
      } else if (entry.name === target) {
        return full;
      }
    }
  },
  NUMBER_SET = new Set([
    "int32",
    "uint32",
    "sint32",
    "fixed32",
    "sfixed32",
    "int64",
    "uint64",
    "sint64",
    "fixed64",
    "sfixed64",
    "float",
    "double"
  ]),
  cap = (name) => name[0].toUpperCase() + name.slice(1),
  toTsType = (raw, sub_map = {}) => {
    if (NUMBER_SET.has(raw)) return "number";
    if (raw.endsWith("Li")) {
      const base = raw.slice(0, -2);
      return (NUMBER_SET.has(base) ? "number" : base === "bool" ? "boolean" : base) + "[]";
    }
    if (raw === "bool") return "boolean";
    if (raw === "bytes") return "Uint8Array";
    if (raw.endsWith("[]")) {
      const base = raw.slice(0, -2),
        sub = sub_map[base];
      return (sub || "any") + "[]";
    }
    const sub = sub_map[raw];
    return sub || raw || "any";
  },
  parseComments = (comments, toCamel) => {
    const name_li = [];
    for (const { value } of comments) {
      for (const line of value.split("\n")) {
        const m = line.trim().match(/^\d+\s+([a-zA-Z0-9_]+)/);
        if (m && m[1] !== "_") name_li.push(toCamel(m[1]));
      }
    }
    return name_li;
  },
  parseProtoJs = (file_path, toCamel) => {
    const src = read(file_path),
      ast = parseJs(src, { sourceType: "module" }),
      name_li = parseComments(ast.comments, toCamel);

    let cur_path = file_path,
      cur_src = src,
      cur_ast = ast,
      reexport_path;

    do {
      reexport_path = undefined;
      for (const { type, source } of cur_ast.program.body) {
        if (type === "ExportNamedDeclaration" && source?.value) {
          reexport_path = join(dirname(cur_path), source.value);
          break;
        }
      }
      if (reexport_path) {
        cur_path = reexport_path;
        cur_src = read(cur_path);
        cur_ast = parseJs(cur_src, { sourceType: "module" });
      }
    } while (reexport_path);

    let array_code = "[]",
      elements = [];
    const import_map = new Map(),
      field_li = [],
      sub_map = {},
      resolveSub = (sub_name) => {
        if (sub_map[sub_name]) return;
        for (const [src_pkg, entry] of import_map) {
          if (entry.default === sub_name && src_pkg.startsWith(".")) {
            const sub_file = join(
                dirname(cur_path),
                src_pkg.endsWith(".js") ? src_pkg : src_pkg + ".js"
              ),
              sub_info = parseProtoJs(sub_file, toCamel);
            sub_map[sub_name] =
              "[" + sub_info.field_li.map((f) => f.name + "?: " + f.type).join(",") + "]";
          }
        }
      };

    for (const node of cur_ast.program.body) {
      const { type } = node;
      if (type === "ImportDeclaration") {
        const src_pkg = node.source.value;
        if (!import_map.has(src_pkg)) import_map.set(src_pkg, { named: new Set(), default: null });
        const entry = import_map.get(src_pkg);
        for (const spec of node.specifiers) {
          if (spec.type === "ImportDefaultSpecifier") {
            entry.default = spec.local.name;
          } else {
            const { name } = spec.local;
            if (name !== "$E" && name !== "$D" && spec.imported?.name !== "$") {
              entry.named.add(name);
            }
          }
        }
      } else if (type === "ExportDefaultDeclaration") {
        const { declaration } = node;
        if (declaration.type === "CallExpression" && declaration.arguments.length) {
          const [arg] = declaration.arguments;
          if (arg.type === "ArrayExpression") {
            elements = arg.elements.map((el) => {
              if (el.type === "Identifier") {
                resolveSub(el.name);
                return el.name;
              }
              if (
                el.type === "ArrayExpression" &&
                el.elements.length === 1 &&
                el.elements[0].type === "Identifier"
              ) {
                const sub_name = el.elements[0].name;
                resolveSub(sub_name);
                return sub_name + "[]";
              }
              return "any";
            });
          }
          array_code = cur_src
            .slice(arg.start, arg.end)
            .replace(/\/\*.*?\*\//g, "")
            .replace(/\s+/g, " ")
            .trim();
        }
      }
    }

    name_li.forEach((name, i) => {
      field_li.push({ name, type: toTsType(elements[i], sub_map) });
    });

    return { cur_path, import_map, array_code, field_li };
  };

export default (
  api_dir,
  pkg,
  pkg_src_gen_dir,
  call_fields,
  toCamel,
  GEN_HEAD,
  enum_li = [],
  url_li = []
) => {
  const pkg_api_dir = join(api_dir, pkg);
  rmSync(pkg_api_dir, { recursive: true, force: true });

  const enum_dir = join(pkg_api_dir, "enum");
  for (const enum_name of enum_li) {
    const src_file = findEnumFile(pkg_src_gen_dir, enum_name + ".js");
    if (src_file) {
      write(join(enum_dir, enum_name + ".js"), read(src_file));
    }
  }

  write(
    join(pkg_api_dir, "_req.js"),
    GEN_HEAD +
      'import { req } from "@1-/protoapi";\n\n' +
      "export default req(" +
      JSON.stringify(pkg) +
      ");\n"
  );

  const copied_proto_set = new Set(),
    copyProto = (rel_file) => {
      if (copied_proto_set.has(rel_file)) return;
      copied_proto_set.add(rel_file);

      const src_file = join(pkg_src_gen_dir, rel_file);
      if (!existsSync(src_file)) return;

      const content = read(src_file),
        [static_li] = importLi(content);
      write(join(pkg_api_dir, "proto", rel_file), content);

      for (const item of static_li) {
        if (item.startsWith(".")) {
          let dep = item;
          if (!dep.endsWith(".js")) dep += ".js";
          const dep_abs = resolve(dirname(src_file), dep),
            dep_rel = relative(pkg_src_gen_dir, dep_abs);
          copyProto(dep_rel);
        }
      }
    };

  for (const field of Object.values(call_fields)) {
    const { id, type } = field,
      url_path = url_li[id - 1],
      parts = url_path.split("/"),
      base_name = parts.at(-1),
      cap_base = cap(base_name),
      sub_parts = parts.slice(0, -1),
      raw_req_type = type.value,
      req_type = raw_req_type.startsWith(pkg + ".")
        ? raw_req_type.slice(pkg.length + 1)
        : raw_req_type,
      res_type = req_type.endsWith("Req")
        ? req_type.slice(0, -3)
        : sub_parts.length
          ? sub_parts.join(".") + "." + cap_base
          : cap_base,
      req_e_file =
        req_type === "Empty"
          ? join(pkg_src_gen_dir, "EmptyE.js")
          : join(pkg_src_gen_dir, ...req_type.split(".")) + "E.js",
      res_d_file = join(pkg_src_gen_dir, ...res_type.split(".")) + "D.js",
      api_js_file = join(pkg_api_dir, url_path + ".js"),
      api_dts_file = join(pkg_api_dir, url_path + ".d.ts"),
      api_file_dir = dirname(api_js_file),
      req_info = parseProtoJs(req_e_file, toCamel),
      res_info = parseProtoJs(res_d_file, toCamel),
      req_named_set = new Set();

    for (const [, entry] of req_info.import_map) {
      if (entry.default) req_named_set.add(entry.default);
      for (const n of entry.named) req_named_set.add(n);
    }

    let res_array_code = res_info.array_code;
    for (const [, entry] of res_info.import_map) {
      const new_named = new Set();
      for (const n of entry.named) {
        if (req_named_set.has(n)) {
          new_named.add(n + " as D_" + n);
          res_array_code = res_array_code.replace(new RegExp(`\\b${n}\\b`, "g"), "D_" + n);
        } else {
          new_named.add(n);
        }
      }
      entry.named = new_named;
    }

    const merged_import = new Map(),
      imp_li = [];

    for (const info of [req_info, res_info]) {
      for (const [pkg_src, entry] of info.import_map) {
        let import_key = pkg_src;
        if (pkg_src.startsWith(".")) {
          const abs_file = resolve(
              dirname(info.cur_path),
              pkg_src.endsWith(".js") ? pkg_src : pkg_src + ".js"
            ),
            rel_to_gen = relative(pkg_src_gen_dir, abs_file);
          copyProto(rel_to_gen);

          const target_proto = join(pkg_api_dir, "proto", rel_to_gen),
            rel_import = relative(api_file_dir, target_proto);
          import_key = rel_import.startsWith(".") ? rel_import : "./" + rel_import;
        }

        if (!merged_import.has(import_key)) {
          merged_import.set(import_key, { named: new Set(), default: null });
        }
        const merged_entry = merged_import.get(import_key);
        if (entry.default) merged_entry.default = entry.default;
        for (const n of entry.named) merged_entry.named.add(n);
      }
    }

    for (const [import_path, entry] of merged_import) {
      if (entry.default) {
        imp_li.push("import " + entry.default + ' from "' + import_path + '";');
      }
      if (entry.named.size) {
        imp_li.push("import { " + [...entry.named].join(",") + ' } from "' + import_path + '";');
      }
    }

    const rel_req = relative(api_file_dir, join(pkg_api_dir, "_req.js")),
      req_import = rel_req.startsWith(".") ? rel_req : "./" + rel_req;

    imp_li.push('import req from "' + req_import + '";');

    const param_names = req_info.field_li.map((f) => f.name),
      params_str = param_names.join(","),
      call_args_str = params_str ? "," + params_str : "",
      code =
        GEN_HEAD +
        imp_li.join("\n") +
        "\n\nexport default (" +
        params_str +
        ") => req(" +
        id +
        "," +
        req_info.array_code +
        "," +
        res_array_code +
        call_args_str +
        ");\n";

    write(api_js_file, code);

    const dts_params = req_info.field_li.map((f) => f.name + "?: " + f.type).join(","),
      dts_returns = res_info.field_li.map((f) => f.name + "?: " + f.type).join(","),
      dts_code =
        GEN_HEAD +
        "declare const _default: (" +
        dts_params +
        ") => Promise<[" +
        dts_returns +
        "]>;\nexport default _default;\n";
    write(api_dts_file, dts_code);
  }
};
