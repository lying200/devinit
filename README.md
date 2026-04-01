# devinit

自动检测项目语言并生成 [devenv](https://devenv.sh) 开发环境配置。

## 功能

- 5 种语言支持：Rust、Python、Go、Java、JavaScript
- 项目自动检测：语言识别 + 版本推断
- Monorepo 支持：自动扫描一级子目录
- 多语言支持：单项目同时配置多种语言
- 检测结果可局部修改：保留正确的，修改错误的
- 非交互式模式：`--yes` 跳过提示，适用于 CI/CD
- `--force` 覆盖已有配置
- 已有 devenv/direnv/Nix 环境保护
- Git ignore 处理（`.gitignore` 或 `.git/info/exclude`）

## 安装

### Nix Flake

```bash
# 直接运行
nix run github:lying200/devinit

# 安装到 profile
nix profile install github:lying200/devinit

# 在 flake.nix 中引用
{
  inputs.devinit.url = "github:lying200/devinit";
}
```

### 从源码构建

```bash
cargo install --path .
```

## 用法

```bash
# 交互式（自动检测语言，提示确认）
devinit

# 指定语言
devinit --lang go
devinit --lang go,javascript

# 非交互式（自动检测 + 默认配置，适用于 CI/CD）
devinit --yes

# 非交互式 + 指定语言
devinit --yes --lang go,javascript

# 覆盖已有配置
devinit --force

# 指定目录
devinit /path/to/project
```

## 生成文件

| 文件 | 说明 |
|------|------|
| `devenv.nix` | 语言工具链、包、环境变量配置 |
| `devenv.yaml` | devenv inputs（nixpkgs + 语言 overlay） |
| `.envrc` | direnv 集成，激活 devenv 环境 |

## 许可证

[MIT](LICENSE)
