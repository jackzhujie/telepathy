# 设计规格：高级 Office 文档原生解析插件与引导式开关设计

## 1. 背景与问题描述 (Background & Context)
目前 Telepathy 本地笔记应用在导入 `.docx`、`.xlsx`、`.xls`、`.pptx` 等 Office 文档时，依赖于 `parser-advanced` sidecar。如果用户未安装该 sidecar，系统将抛出错误。这给用户的日常使用（尤其是普通的 Word、Excel、PPT 文件）带来了不便。

我们希望通过引入后端原生解析逻辑（在 Rust 后端完成常见 Office 文档的高性能解析），使得用户在**不依赖外部 sidecar 插件**的情况下也能完美解析和向量索引这些格式。

同时，由于 docx、xlsx、pptx 解析目前被定位为高级功能，我们计划在**“系统设置”**中提供一个开启/关闭“高级 Office 文档解析扩展”的开关：
1. 目前此功能对用户全功能免费。
2. 默认该开关关闭。
3. 如果开关关闭时用户导入或解析这类文件，应用会友好引导用户前往设置页面开启开关。

## 2. 解决方案设计 (Proposed Solution)
本设计包含**后端原生解析器引入**、**设置库开关持久化**以及**前端引导式 UX 交互**三个部分。

### 后端设计

#### 2.1 引入纯 Rust 原生解析方案 (方案 A)
- **Excel (.xlsx/.xls)**：使用 `calamine` 库。它极轻量、极其鲁棒，支持现代 `.xlsx` 和经典的二进制 `.xls` (OLE) 格式。解析后将所有的 Sheet 格式化为标准 **CSV 字符串** (以逗号 `,` 分隔单元格，换行分隔行) 输出。
- **Word (.docx)**：在 Rust 中使用 `zip` 库解包 docx 压缩文件，读取 `word/document.xml`，解析 XML 中的段落 `<w:p>` 标签并拼接为纯文本，保留基本的换行结构。
- **PPT (.pptx)**：解压 pptx，按顺序（如 `slide1.xml`, `slide2.xml` ...）读取 `ppt/slides/slide{n}.xml`，提取其中的文本框文本。为了利于后续向量切片（Chunking）保留幻灯片的物理页面边界，每页之间使用 **`--- Slide N ---`** 分隔线进行隔离输出。

#### 2.2 设置库开关持久化
- 新置种子配置项：`("enable_office_parser", "false")` 在数据库中默认初始化。
- 只有当 `enable_office_parser` 开关在数据库（缓存）中被置为 `"true"` 时，后端解析模块才响应解析请求。
- 否则，将对 `.docx`, `.xlsx`, `.xls`, `.pptx` 抛出特定的引导性错误消息：
  > "该文档为高级 Office 格式（.docx/.xlsx/.pptx/.xls）。当前未开启「高级文档解析扩展」，请前往「系统设置」开启该功能以开始解析和检索。"

### 前端与 UX 设计

#### 2.3 扩充文件选择与导入范围
- 修改 `BatchImportDialog.vue` 的文件选择过滤器和文件夹扫描逻辑，将受支持的格式范围扩大为：`['pdf', 'md', 'txt', 'docx', 'xlsx', 'pptx', 'xls']`。

#### 2.4 系统设置界面 (Settings.vue) 开启开关
在系统设置中加入名为 **“高级功能扩展”** 的独立卡片，包含一个支持流畅微动画的 Toggle 开关按钮，绑定 `enable_office_parser` 键值。

#### 2.5 导入与文档管理界面的主动式引导 (Documents.vue)
- 如果 `settings.enable_office_parser !== 'true'`，会在文档列表上方展示一个亮眼的黄橘色提示横幅 (Banner)：
  > "💡 **提示：高级 Office 文档解析未开启。** 您当前无法解析 Word、Excel 和 PPT 文档。您可以 [前往设置开启此功能](/settings)。"
- 点击链接会自动切换到系统设置页面。
- 即使关闭了横幅，如果在解析文件时由于开关未开启而导致失败，错误信息也会在列表的状态栏中以 `"待开启高级解析扩展..."` 等形式清晰显示。

---

## 3. 详细变更规划 (Detailed Implementation Plan)

### 3.1 后端部分

1. **`Cargo.toml` 依赖引入**：
   ```toml
   [dependencies]
   calamine = "0.24"
   quick-xml = "0.31" # 高性能的 XML 拉流解析器，用于 docx/pptx
   ```

2. **`src/db/settings.rs`**：
   在 `DEFAULT_SETTINGS` 数组中增加：
   ```rust
   ("enable_office_parser", "false"),
   ```

3. **`src/services/parser/core.rs`**：
   实现三个原生提取函数：
   - `pub fn parse_docx(path: &Path) -> Result<String, AppError>`
   - `pub fn parse_excel(path: &Path) -> Result<String, AppError>`
   - `pub fn parse_pptx(path: &Path) -> Result<String, AppError>`

4. **`src/services/parser/mod.rs`**：
   从 `SettingsCache` 中获取 `enable_office_parser` 状态并根据其进行过滤守护。
   ```rust
   let enable_office_parser = cache.get("enable_office_parser").map(|s| s == "true").unwrap_or(false);
   ```
   分发逻辑修改：
   - 如果是 Office 格式且 `!enable_office_parser`，返回统一的报错提示引导开启。
   - 如果已开启，则调用 `core::parse_docx` 等原生解析方法。

### 3.2 前端部分

1. **`src/views/Settings.vue`**：
   增加卡片布局，定义 `toggleOfficeParser` 方法更新持久化状态。
2. **`src/views/Documents.vue`**：
   - 从 `settingsStore` 获取 `enable_office_parser` 状态。
   - 在顶部添加带有跳转按钮的 Banner。
3. **`src/components/batch/BatchImportDialog.vue`**：
   - 更改 `extensions` 范围。
   - 对 PPTX、XLSX、XLS 展示特定的格式表情符号（如 📊、📈、📘）。

---

## 4. 验证方案 (Verification Plan)

### 4.1 自动测试 (Automated Tests)
- 在 `core.rs` 中为 `parse_docx`, `parse_excel`, `parse_pptx` 添加单元测试，验证能够正常读取 mock 测试文件并正确提取其内容，保证解析出的格式正确（如 Excel 解析出逗号分隔的 CSV，PPTX 包含页码标记等）。
- 运行 `cargo test` 检验编译与逻辑正确性。
- 运行 `npm run build` 和 `vue-tsc --noEmit` 校验 TypeScript 语法是否无误。

### 4.2 手动测试路径 (Manual Test Cases)
1. **引导闭环验证**：
   - 在未开启开关时导入一个 `.docx` 文件，点击“解析”，观察是否提示引导开启的高亮错误提示。
   - 点击顶部提示 Banner 中的跳转链接，验证是否能秒级跳转至 `/settings` 界面。
2. **开关状态改变验证**：
   - 切换开关为“开启”状态，再次回到文档界面导入相同 Word 文件并点击“解析”，应当流畅解析完成，并将状态显示为“已就绪”。
   - 分别导入 `.xlsx`、`.pptx`，验证 CSV 输出内容及带 `--- Slide N ---` 页码隔离的幻灯片内容。
