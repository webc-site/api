#!/usr/bin/env bun
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { $ } from "bun";

const KVROCKS_DIR = "/tmp/kvrocks",
  KVROCKS_SRC = KVROCKS_DIR + "/src/commands",
  RUST_CLIENT_SRC = join(import.meta.dirname, "src/client"),
  RUST_RESP3_SRC = join(import.meta.dirname, "src/resp3");

if (!existsSync(KVROCKS_SRC)) {
  console.log("正在克隆 Apache Kvrocks 仓库...");
  await $`git clone --depth=1 https://github.com/apache/kvrocks.git ${KVROCKS_DIR}`.quiet();
}

// 终端颜色
const color = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  dim: "\x1b[2m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
  cyan: "\x1b[36m",
  red: "\x1b[31m"
};

// 错误与告警统一收集器
const all_errors = [];
const all_warnings = [];

// =========================================================================
// 1. 解析 Kvrocks C++ 命令元数据（全量 345 指令、分类、选项）
// =========================================================================
const kvrocks_commands = new Map(); // cmd_name -> { category, commanderClass, file, options, returnsNil, returnType }
const kvrocks_categories = new Map(); // category -> Set<cmd_name>

for (const file of readdirSync(KVROCKS_SRC).filter(
  (f) => f.startsWith("cmd_") && f.endsWith(".cc")
)) {
  const category = file.slice(4, -3);
  const content = readFileSync(join(KVROCKS_SRC, file), "utf-8");

  // 提取 MakeCmdAttr 注册 (支持跨行与复杂模板参数)
  const attrRegex = /MakeCmdAttr<([\s\S]*?)>\(\s*"([^"]+)"/g;
  let match;
  while ((match = attrRegex.exec(content)) !== null) {
    const [_, commanderClass, cmdName] = match;
    const upperCmd = cmdName.toUpperCase();

    if (!kvrocks_categories.has(category)) {
      kvrocks_categories.set(category, new Set());
    }
    kvrocks_categories.get(category).add(upperCmd);

    // 查找对应 Commander 的 Parse 和 Execute 代码块
    const baseClassName = commanderClass.split(",")[0].trim();
    const classRegex = new RegExp(`class\\s+${baseClassName}\\s*:[^{]*\\{([\\s\\S]*?)\\};`, "m");
    const classMatch = content.match(classRegex);
    const classBody = classMatch ? classMatch[1] : "";

    // 提取 options / flags (EatEqICase, ParseExpireFlags 等)
    const options = new Set();
    const optRegex = /EatEqICase\(\s*"([^"]+)"/g;
    let optMatch;
    while ((optMatch = optRegex.exec(classBody)) !== null) {
      options.add(optMatch[1].toUpperCase());
    }
    if (classBody.includes("ParseExpireFlags") || classBody.includes("ParseGetExExpireFlags")) {
      for (const f of ["EX", "PX", "EXAT", "PXAT", "KEEPTTL", "PERSIST"]) options.add(f);
    }
    if (classBody.includes("CommandScanBase")) {
      for (const f of ["MATCH", "COUNT", "TYPE", "NOVALUES"]) options.add(f);
    }
    if (classBody.includes("ParseSort")) {
      for (const f of ["BY", "LIMIT", "GET", "ASC", "DESC", "ALPHA", "STORE"]) options.add(f);
    }

    const returnsNil =
      classBody.includes("conn->NilString()") ||
      classBody.includes("conn->NilArray()") ||
      classBody.includes("redis::NilString()") ||
      classBody.includes("redis::NilArray()") ||
      classBody.includes("Null()");

    let returnType = "Value";
    if (classBody.includes("redis::Integer(") || classBody.includes("conn->Integer(")) {
      returnType = "Integer";
    } else if (
      classBody.includes("redis::BulkString(") ||
      classBody.includes("conn->BulkString(")
    ) {
      returnType = "BulkString";
    } else if (classBody.includes("redis::Array(") || classBody.includes("conn->Array(")) {
      returnType = "Array";
    } else if (classBody.includes("redis::Double(") || classBody.includes("conn->Double(")) {
      returnType = "Double";
    } else if (classBody.includes("redis::RESP_OK") || classBody.includes("conn->Ok()")) {
      returnType = "Status_OK";
    }

    kvrocks_commands.set(upperCmd, {
      category,
      commanderClass: baseClassName,
      file,
      options: [...options],
      returnsNil,
      returnType
    });
  }
}

// =========================================================================
// 2. 解析 Rust Client 代码（Methods, Cmd calls, Enums, Return Types）
// =========================================================================
const rust_cmd_set = new Set();
const rust_methods = new Map(); // method_name -> { file, returnType }
const rust_enums = new Map(); // enum_name -> Set<variant_name>
const rust_hardcoded_args = []; // { file, line, arg }

// 提取 Rust Result<...> 内部类型
function extractReturnType(sigAfterResult) {
  let depth = 0;
  let startIdx = -1;
  for (let i = 0; i < sigAfterResult.length; i++) {
    if (sigAfterResult[i] === "<") {
      if (depth === 0) startIdx = i + 1;
      depth++;
    } else if (sigAfterResult[i] === ">") {
      depth--;
      if (depth === 0) {
        return sigAfterResult.slice(startIdx, i).trim();
      }
    }
  }
  return sigAfterResult.trim();
}

// 检查 constants.rs 常量定义
const constants_content = readFileSync(join(RUST_RESP3_SRC, "constants.rs"), "utf-8");
const defined_constants = new Set();
const const_map = new Map();

// 1. 匹配 pub const ...
for (const match of constants_content.matchAll(/pub const ([A-Z0-9_]+): &str = "([^"]+)";/g)) {
  defined_constants.add(match[2]);
  const_map.set(match[1], match[2]);
}

// 2. 匹配 const_str! 宏
const macroMatch = constants_content.match(/const_str!\s*\{([\s\S]*?)\}/);
if (macroMatch) {
  const body = macroMatch[1];
  for (const line of body.split("\n")) {
    const trimmed = line.trim().replace(/,$/, "");
    if (!trimmed || trimmed.startsWith("//")) continue;
    if (trimmed.includes("=")) {
      const parts = trimmed.split("=");
      const key = parts[0].trim();
      const val = parts[1].trim().replace(/^"|"$/g, "");
      defined_constants.add(val);
      const_map.set(key, val);
    } else {
      defined_constants.add(trimmed);
      const_map.set(trimmed, trimmed);
    }
  }
}

for (const file of readdirSync(RUST_CLIENT_SRC).filter(
  (f) => f.endsWith(".rs") && f !== "mod.rs"
)) {
  const content = readFileSync(join(RUST_CLIENT_SRC, file), "utf-8");
  const lines = content.split("\n");

  // 1. 匹配所有构造的命令名称 (支持常量标识符和字符串字面量)
  const cmdMatches = content.matchAll(
    /(?:Cmd::new|build_\w+|eval_generic|json_key_path_cmd)(?:::<[^>]+>)?\(\s*([A-Za-z0-9_"]+)/g
  );
  for (const [_, rawCmd] of cmdMatches) {
    if (rawCmd.startsWith('"')) {
      rust_cmd_set.add(rawCmd.replace(/^"|"$/g, "").toUpperCase());
    } else if (const_map.has(rawCmd)) {
      rust_cmd_set.add(const_map.get(rawCmd).toUpperCase());
    } else if (rawCmd !== "name") {
      rust_cmd_set.add(rawCmd.toUpperCase());
    }
  }

  // 2. 匹配 Enum 定义及其 Variants
  const enumRegex = /pub\s+enum\s+([A-Za-z0-9_]+)(?:<[^>]+>)?\s*\{([\s\S]*?)\}/g;
  let enumMatch;
  while ((enumMatch = enumRegex.exec(content)) !== null) {
    const [_, enumName, body] = enumMatch;
    const variants = new Set(
      [...body.matchAll(/^\s*([A-Za-z0-9_]+)(?:\(.*\))?,?/gm)]
        .map((m) => m[1].toUpperCase())
        .filter((v) => v !== "PUB" && v !== "USE")
    );
    rust_enums.set(enumName, variants);
  }

  // 3. 稳健提取 pub async fn 签名与返回值类型
  const fnHeadRegex = /pub\s+async\s+fn\s+([a-z0-9_]+)/g;
  let fnHeadMatch;
  while ((fnHeadMatch = fnHeadRegex.exec(content)) !== null) {
    const fnName = fnHeadMatch[1];
    const fnOffset = fnHeadMatch.index;
    const block = content.slice(fnOffset, fnOffset + 300);
    const resIdx = block.indexOf("-> Result<");
    if (resIdx !== -1) {
      const afterRes = block.slice(resIdx + "-> Result".length);
      const retType = extractReturnType(afterRes);
      rust_methods.set(fnName, {
        file,
        returnType: retType.replace(/\s+/g, " ")
      });
    }
  }

  // 4. 检查残留硬编码 .arg("...")
  for (let idx = 0; idx < lines.length; idx++) {
    const line = lines[idx];
    const directArgMatch = line.match(/\.arg\(\s*"([^"]+)"\s*\)/);
    if (directArgMatch) {
      rust_hardcoded_args.push({ file, line: idx + 1, arg: directArgMatch[1] });
    }
  }
}

// =========================================================================
// 3. 报告输出：维度 1 - 命令全量覆盖度检查
// =========================================================================
console.log(
  `${color.bold}${color.cyan}=================================================================${color.reset}`
);
console.log(
  `${color.bold}${color.cyan}    Apache Kvrocks Rust 客户端接口与契约综合质量检查报告    ${color.reset}`
);
console.log(
  `${color.bold}${color.cyan}=================================================================${color.reset}\n`
);

console.log(`${color.bold}[ 维度 1: 指令全量覆盖度检查 (Kvrocks C++ 对比) ]${color.reset}`);
console.log(
  `${"类别".padEnd(16)} ${"Kvrocks".padEnd(10)} ${"已实现".padEnd(8)} ${"覆盖率".padEnd(8)} 状态`
);
console.log("-".repeat(56));

let total_kvrocks = 0;
let total_implemented = 0;

for (const [category, cmd_set] of kvrocks_categories) {
  const missing_li = [];
  let implemented = 0;

  for (const cmd of cmd_set) {
    ++total_kvrocks;
    if (rust_cmd_set.has(cmd)) {
      ++total_implemented;
      ++implemented;
    } else {
      missing_li.push(cmd);
    }
  }

  if (missing_li.length) {
    all_errors.push(`[指令覆盖] ${category} 缺少指令: ${missing_li.join(", ")}`);
  }

  const rate = `${((implemented / cmd_set.size) * 100).toFixed(1)}%`;
  const status =
    implemented === cmd_set.size
      ? `${color.green}✅ 100%${color.reset}`
      : implemented > 0
        ? `${color.yellow}🟡 部分实现${color.reset}`
        : `${color.red}❌ 未实现${color.reset}`;

  console.log(
    `${category.padEnd(16)} ${String(cmd_set.size).padEnd(10)} ${String(implemented).padEnd(8)} ${rate.padEnd(8)} ${status}`
  );
}

console.log("-".repeat(56));
const overall_rate = `${((total_implemented / total_kvrocks) * 100).toFixed(1)}%`;
console.log(
  `${color.bold}${"总计".padEnd(16)} ${String(total_kvrocks).padEnd(10)} ${String(total_implemented).padEnd(8)} ${overall_rate.padEnd(8)}${color.reset}`
);

// =========================================================================
// 4. 报告输出：维度 2 - 指令参数与选项 (Flags/Options) 覆盖检查
// =========================================================================
console.log(`\n${color.bold}[ 维度 2: 关键指令参数与选项 (Flags/Options) 覆盖检查 ]${color.reset}`);

const important_option_commands = [
  {
    cmd: "SET",
    enum: "Set",
    expected: [
      "EX",
      "PX",
      "EXAT",
      "PXAT",
      "KEEPTTL",
      "NX",
      "XX",
      "GET",
      "IFEQ",
      "IFNE",
      "IFDEQ",
      "IFDNE"
    ]
  },
  { cmd: "SCAN", enum: "Scan", expected: ["MATCH", "COUNT", "TYPE", "NOVALUES"] },
  { cmd: "HSCAN", enum: "HScan", expected: ["MATCH", "COUNT", "NOVALUES"] },
  { cmd: "SSCAN", enum: "SScan", expected: ["MATCH", "COUNT", "NOVALUES"] },
  { cmd: "ZSCAN", enum: "ZScan", expected: ["MATCH", "COUNT", "NOVALUES"] },
  { cmd: "ZADD", enum: "ZAddOption", expected: ["NX", "XX", "GT", "LT", "CH", "INCR"] },
  { cmd: "XADD", enum: "XAddOption", expected: ["NOMKSTREAM", "MAXLEN", "MINID", "LIMIT"] },
  {
    cmd: "HSETEX",
    enum: "HSetExOption",
    expected: ["EX", "PX", "EXAT", "PXAT", "KEEPTTL", "FNX", "FXX"]
  },
  { cmd: "HGETEX", enum: "HGetExOption", expected: ["EX", "PX", "EXAT", "PXAT", "PERSIST"] },
  {
    cmd: "GEOSEARCH",
    enum: "GeoSearch",
    expected: [
      "FROMMEMBER",
      "FROMLONLAT",
      "BYRADIUS",
      "BYBOX",
      "ASC",
      "DESC",
      "COUNT",
      "WITHCOORD",
      "WITHDIST",
      "WITHHASH"
    ]
  },
  {
    cmd: "TS.CREATE",
    enum: "TsCreate",
    expected: ["RETENTION", "CHUNKSIZE", "DUPLICATEPOLICY", "LABELS"]
  },
  { cmd: "BF.RESERVE", enum: "BfReserve", expected: ["EXPANSION", "NONSCALING"] },
  {
    cmd: "BF.INSERT",
    enum: "BfInsert",
    expected: ["CAPACITY", "ERROR", "EXPANSION", "NOCREATE", "NONSCALING"]
  },
  { cmd: "TDIGEST.MERGE", enum: "TDigestMerge", expected: ["OVERRIDE"] },
  { cmd: "FT.SEARCH", enum: "FtSearch", expected: ["LIMIT", "NOCONTENT"] }
];

for (const item of important_option_commands) {
  const enumVariants = rust_enums.get(item.enum);
  if (!enumVariants) {
    all_errors.push(`[选项缺失] 未找到选项 Enum: ${item.enum} (${item.cmd})`);
    continue;
  }
  const missingOpts = [];
  for (const exp of item.expected) {
    const matched = [...enumVariants].some((v) => v.replace(/_/g, "") === exp.replace(/_/g, ""));
    if (!matched) missingOpts.push(exp);
  }

  if (missingOpts.length === 0) {
    console.log(
      `  ${color.green}✔${color.reset} [${item.cmd.padEnd(10)}] Enum ${item.enum.padEnd(14)}: 支持全部 ${item.expected.length} 项选项 (${item.expected.join(", ")})`
    );
  } else {
    all_warnings.push(
      `[选项缺失] [${item.cmd}] Enum ${item.enum} 缺少变体: ${missingOpts.join(", ")}`
    );
  }
}

// =========================================================================
// 5. 报告输出：维度 3 - 返回值类型契约与 Null 安全检查
// =========================================================================
console.log(
  `\n${color.bold}[ 维度 3: 返回值类型契约与 Null 安全检查 (去冗余 Result< 前缀) ]${color.reset}`
);

const null_sensitive_methods = [
  { method: "get", cmd: "GET", expectOpt: true },
  { method: "hget", cmd: "HGET", expectOpt: true },
  { method: "lindex", cmd: "LINDEX", expectOpt: true },
  { method: "zscore", cmd: "ZSCORE", expectOpt: true },
  { method: "lpop", cmd: "LPOP", expectOpt: true },
  { method: "rpop", cmd: "RPOP", expectOpt: true },
  { method: "blpop", cmd: "BLPOP", expectOpt: true },
  { method: "brpop", cmd: "BRPOP", expectOpt: true },
  { method: "bzpopmin", cmd: "BZPOPMIN", expectOpt: true },
  { method: "bzpopmax", cmd: "BZPOPMAX", expectOpt: true },
  { method: "geodist", cmd: "GEODIST", expectOpt: true },
  { method: "tdigest_max", cmd: "TDIGEST.MAX", expectOpt: true },
  { method: "tdigest_min", cmd: "TDIGEST.MIN", expectOpt: true },
  { method: "ts_get", cmd: "TS.GET", expectOpt: true },
  { method: "set", cmd: "SET", expectOpt: true },
  { method: "set_typed", cmd: "SET", expectOpt: true },
  { method: "set_get", cmd: "SET", expectOpt: true },
  { method: "xadd", cmd: "XADD", expectOpt: true },
  { method: "xadd_opt", cmd: "XADD", expectOpt: true },
  { method: "json_get", cmd: "JSON.GET", expectOpt: true },
  { method: "digest", cmd: "DIGEST", expectOpt: true },
  { method: "dump", cmd: "DUMP", expectOpt: true },
  { method: "randomkey", cmd: "RANDOMKEY", expectOpt: true },
  { method: "client_getname", cmd: "CLIENT GETNAME", expectOpt: true }
];

for (const check of null_sensitive_methods) {
  const method = rust_methods.get(check.method);
  if (!method) {
    all_errors.push(`[方法缺失] 未找到方法: fn ${check.method}`);
    continue;
  }

  const isOpt = method.returnType.startsWith("Option<");
  if (check.expectOpt && isOpt) {
    console.log(
      `  ${color.green}✔${color.reset} fn ${check.method.padEnd(16)} -> ${method.returnType.padEnd(24)} (可空安全)`
    );
  } else {
    all_errors.push(`[契约异常] fn ${check.method} -> ${method.returnType} (预期返回 Option<T>)`);
  }
}

// =========================================================================
// 6. 报告输出：维度 4 - 常量化规范与硬编码审计
// =========================================================================
console.log(`\n${color.bold}[ 维度 4: 常量化规范与硬编码审计 ]${color.reset}`);
console.log(
  `  • 已在 constants.rs 中统一定义常量: ${color.green}${defined_constants.size} 项${color.reset}`
);

if (rust_hardcoded_args.length === 0) {
  console.log(
    `  • 硬编码参数/子命令字面量扫描: ${color.green}0 处硬编码，已 100% 全部常量化！${color.reset}`
  );
} else {
  for (const item of rust_hardcoded_args) {
    all_warnings.push(
      `[硬编码] ${item.file}:${item.line} -> 直接使用 .arg("${item.arg}")，建议使用常量`
    );
  }
}

// =========================================================================
// 7. 总结、评分与统一错误/警告输出
// =========================================================================
console.log(
  `\n${color.bold}=================================================================${color.reset}`
);
const totalChecks =
  total_kvrocks +
  important_option_commands.length +
  null_sensitive_methods.length +
  (rust_hardcoded_args.length === 0 ? 1 : 0);
const passedChecks =
  total_implemented +
  (important_option_commands.length - all_warnings.filter((w) => w.includes("[选项缺失]")).length) +
  (null_sensitive_methods.length -
    all_errors.filter((e) => e.includes("[契约异常]") || e.includes("[方法缺失]")).length) +
  (rust_hardcoded_args.length === 0 ? 1 : 0);
const healthScore = ((passedChecks / totalChecks) * 100).toFixed(1);

console.log(
  `${color.bold}综合质量评分: ${all_errors.length === 0 && all_warnings.length === 0 ? color.green : color.yellow}${healthScore} / 100${color.reset}`
);
console.log(`  • 指令覆盖率: ${overall_rate} (${total_implemented}/${total_kvrocks})`);
console.log(
  `  • 选项覆盖率: ${(((important_option_commands.length - all_warnings.filter((w) => w.includes("[选项缺失]")).length) / important_option_commands.length) * 100).toFixed(1)}%`
);
console.log(
  `  • 返回值契约: ${(((null_sensitive_methods.length - all_errors.filter((e) => e.includes("[契约异常]")).length) / null_sensitive_methods.length) * 100).toFixed(1)}%`
);
console.log(
  `  • 常量化完整度: ${rust_hardcoded_args.length === 0 ? "100.0% (0 处硬编码)" : "需收敛常量"}`
);

if (all_errors.length > 0 || all_warnings.length > 0) {
  console.log(
    `\n${color.bold}${color.red}-----------------------------------------------------------------${color.reset}`
  );
  console.log(
    `${color.bold}${color.red}                   统一异常与待改进清单                          ${color.reset}`
  );
  console.log(
    `${color.bold}${color.red}-----------------------------------------------------------------${color.reset}`
  );
  if (all_errors.length > 0) {
    console.log(`\n${color.red}❌ 严重错误 (${all_errors.length} 项):${color.reset}`);
    for (const err of all_errors) {
      console.log(`  • ${err}`);
    }
  }
  if (all_warnings.length > 0) {
    console.log(`\n${color.yellow}⚠ 警告建议 (${all_warnings.length} 项):${color.reset}`);
    for (const warn of all_warnings) {
      console.log(`  • ${warn}`);
    }
  }
} else {
  console.log(
    `\n${color.green}🎉 0 错误 0 警告，全部 345 个指令、选项、返回值契约与常量化 100% 达标！${color.reset}`
  );
}

console.log(
  `${color.bold}${color.cyan}=================================================================${color.reset}\n`
);
