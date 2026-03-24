# devinit

- [x] 核心数据模型
- [x] `devenv.nix` / `devenv.yaml` / `.envrc` 生成
- [x] Rust 最小初始化支持
- [x] Python 最小初始化支持
- [x] Go 最小初始化支持
- [x] Java 最小初始化支持
- [x] JavaScript 最小初始化支持
- [x] 基础 CLI 工作流
- [x] 已有 `devenv` / `direnv` / Nix 环境保护
- [x] Git ignore 处理
- [x] 支持 `.gitignore`
- [x] 支持 `.git/info/exclude`
- [x] 支持父级 Git 仓库检测

## 当前边界

- [x] 目标目录必须已存在
- [x] 不负责 `mkdir`
- [x] 不负责 `git init`
- [x] 主要面向交互式使用

## TODO

- [ ] 非交互式 CLI 参数
- [ ] 常见语言 / 项目结构自动检测
- [ ] 更完整的异常处理
- [ ] 更好的交互体验
- [ ] 更丰富的项目模板
- [ ] 服务 / 工具集成
