# Research: Image Base64 Upload

**Feature Branch**: `002-image-base64-upload`  
**Date**: 2026-02-27

## 研究领域

### 1. DashScope API 图片传入方式

**决策**: 本地文件使用 Base64 编码传入，远程 URL 保持原样

**理由**:
- DashScope API 支持两种图片传入方式：公网 URL 和 Base64 编码
- 公网 URL 必须是 HTTP/HTTPS 协议，不支持 `file://` 协议
- 本地文件只能通过 Base64 编码传入，格式为 `data:{mime_type};base64,{base64_data}`
- 官方文档明确说明此格式（参见 Qwen Image Edit Guide）

**Base64 格式详解**:
```
data:{mime_type};base64,{base64_data}
```
- `{mime_type}`: 图像的媒体类型，如 `image/jpeg`、`image/png`、`image/webp`
- `{base64_data}`: 文件内容经 Base64 编码后的字符串
- 示例: `data:image/jpeg;base64,/9j/4AAQSkZJRgABAQ...`

**受影响的 API**:

| API | 模型 | 图片字段 | 当前实现 | 修改后 |
|-----|------|---------|---------|--------|
| 图生图编辑 | qwen-image-edit-max | `content[].image` | `file://{path}` | `data:{mime};base64,{data}` |
| 图片翻译 | qwen-mt-image | `input.image_url` | `file://{path}` | `data:{mime};base64,{data}` |

**文生图不受影响**: 文生图 API 不接收图片输入，仅接收文字 Prompt。

---

### 2. Rust Base64 编码方案

**决策**: 使用 `base64` crate v0.22

**理由**:
- Rust 标准库不包含 Base64 编码功能
- `base64` crate 是 Rust 生态中最主流的 Base64 库
- 已被 reqwest、serde 等核心生态 crate 间接依赖，不会增加实际二进制体积
- API 简洁：`BASE64_STANDARD.encode(&bytes)` 即可完成编码

**考虑的替代方案**:

| 方案 | 拒绝原因 |
|------|---------|
| data-encoding crate | 功能更广（hex、base32 等），但本项目只需 base64，过于重量 |
| 手动实现 | 不必要的复杂性，容易引入 bug |

**关键 API**:
```rust
use base64::prelude::*;

let bytes: Vec<u8> = std::fs::read("image.png")?;
let encoded: String = BASE64_STANDARD.encode(&bytes);
let data_uri = format!("data:image/png;base64,{}", encoded);
```

---

### 3. MIME 类型映射

**决策**: 基于文件扩展名确定 MIME 类型，与现有代码保持一致

**映射表**:

| 扩展名 | MIME 类型 |
|--------|----------|
| .jpg, .jpeg | image/jpeg |
| .png | image/png |
| .webp | image/webp |
| .gif | image/gif |
| .bmp | image/bmp |
| .tiff, .tif | image/tiff |
| 其他/未知 | image/png (安全默认值) |

**理由**: 现有代码（`edit_image` 和 `translate_image` 命令）已实现基于扩展名的 MIME 检测逻辑。新的 `encode_image_to_data_uri()` 函数复用相同逻辑，确保一致性。

---

### 4. 性能与大小考量

**决策**: 无需特殊优化，直接读取文件 + 编码

**分析**:
- 10MB 图片文件的 Base64 编码约需 10-50ms（现代 CPU）
- 编码后大小增加约 33%（10MB → ~13.3MB）
- DashScope API 对单张图片限制 10MB（原始文件大小），前端已有校验
- 编码后的 data URI 作为 JSON 字符串发送，reqwest 会自动处理序列化

**无需担忧的点**:
- 内存占用：最大约 23MB（10MB 原始 + 13MB 编码），对桌面应用完全可接受
- 网络传输：Base64 增加 33% 大小，但图片生成 API 本身响应时间远大于传输时间差
