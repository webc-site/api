export default {
  entry: [
    "gen.js",
    "tran.js",
    "src/srv.js",
    "src/*/init.js",
    "src/*/url.js",
    "src/*/url/**/*.js",
    "src/*/gen/**/*.js",
    "src/lib/**/*.js",
    "docker/**/*.js",
    "demo/**/*.js",
    "api/js/**/*.js",
    "api/js/**/*.d.ts",
    "src/*/i18n/**/*.js",
    "src/**/test/**/*.js"
  ],
  ignore: ["conf.example/**", "rust/**"],

  rules: {
    unresolved: "off"
  }
};
