# devinit

自动检测项目语言并生成 devenv 开发环境配置。

## 功能

- [x] 核心数据模型
- [x] `devenv.nix` / `devenv.yaml` / `.envrc` 生成
- [x] 5 种语言支持：Rust、Python、Go、Java、JavaScript
- [x] 项目自动检测（语言识别 + 版本推断）
- [x] 单项目多语言支持（如 Go + JavaScript）
- [x] 基础 CLI 工作流
- [x] 非交互式模式（`--yes` 跳过提示，适用于 CI/CD）
- [x] 已有 `devenv` / `direnv` / Nix 环境保护
- [x] Git ignore 处理
  - [x] 支持 `.gitignore`
  - [x] 支持 `.git/info/exclude`
  - [x] 支持父级 Git 仓库检测

## 用法

```bash
# 交互式（自动检测语言，提示确认）
devinit

# 指定语言
devinit --lang go
devinit --lang go --lang javascript

# 非交互式（自动检测 + 默认配置，适用于 CI/CD）
devinit --yes

# 指定目录
devinit /path/to/project
```

## 当前边界

- 目标目录必须已存在
- 不负责 `mkdir`
- 不负责 `git init`

## TODO

- [ ] 更完整的异常处理
- [ ] 更好的交互体验
- [ ] Services 集成（PostgreSQL、Redis 等）
- [ ] 更丰富的项目模板
