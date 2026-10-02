# 设计规格：聊天输入框视觉识别（图片分析）独立开关（修订版）

## 1. 背景与问题描述 (Background & Context)
对于没有 GPU 加速或者 Mac 芯片的普通电脑，运行 7B 级别的多模态视觉模型依靠 CPU 推理会非常缓慢。在当前应用逻辑中，只要用户配置了视觉模型，那么聊天中上传的任何图片都会强制送入视觉模型进行理解，导致回复耗时巨大。
用户的核心需求是：希望能够有选择性地禁用单次对话的图片识别功能。
根据用户的进一步反馈：**如果在本次对话中关闭了视觉识别，那么连带“选择图片”和“粘贴图片”的功能也应该被禁用，以避免混淆。**

## 2. 解决方案设计 (Proposed Solution)
在 `ChatInput.vue` 的输入框旁增加一个**“图片分析 (Vision Mode)”**开关（Toggle）。该开关的作用是彻底掌控当前是否允许输入图片。

### 功能逻辑
1. **状态管理**：
   - 新增 `const visionMode = ref(true)` 状态，默认开启（允许识别图片和上传图片）。
2. **开关关闭 (`visionMode === false`) 时的行为**：
   - 隐藏或禁用现有的图片上传 📎/🖼️ 按钮（置灰并提示“已关闭图片分析”）。
   - 全局的 `handlePaste` (粘贴图片) 监听器将直接忽略粘贴的图片文件。
   - 如果用户在选中了一些图片后，手动将开关关闭，此时会自动清空已选中的图片 (`clearImages()`)，确保输入框内不再有图片。
3. **发送逻辑**：
   - 因为关闭视觉模式后根本无法添加图片，发送时自然只会有纯文本，顺理成章地绕过后端缓慢的多模态推理。

### 界面设计 (UI/UX)
- 在现有 `ChatInput.vue` 的图片附件上传按钮旁边，新增一个微型按钮（包含一个眼睛图标 👁️ ）。
- 开启时：正常高亮显示，提示“已开启图片理解”。
- 关闭时：置灰且加上删除线/关闭图标，同时隐藏或禁用旁边的图片上传按钮。

## 3. 详细代码变更 (Detailed Changes)

### 3.1 `src/components/chat/ChatInput.vue` 修改
1. **添加状态与监视器**：
   ```typescript
   const visionMode = ref(true);

   // 当开关关闭时，如果有已选中的图片，自动清空
   watch(visionMode, (newVal) => {
     if (!newVal && images.value.length > 0) {
       clearImages();
     }
   });
   ```

2. **拦截粘贴事件**：
   在 `handlePaste` 方法中：
   ```typescript
   function handlePaste(event: ClipboardEvent) {
     if (!visionMode.value) return; // 关闭视觉模式时，直接忽略粘贴的图片
     // ... 现有逻辑
   }
   ```

3. **UI 修改**：
   在输入框的外层或按钮栏增加 `visionMode` 切换按钮，并将现有的 `<button>` (添加图片) 绑定 `:disabled="!visionMode"` 或者 `v-if="visionMode"`。
   推荐将“视觉模式”开关按钮和“添加图片”按钮并排放在一起：
   ```vue
   <!-- 视觉模式开关 -->
   <button
     @click="visionMode = !visionMode"
     class="p-2 transition-colors rounded-lg flex items-center justify-center"
     :class="visionMode ? 'text-brand hover:bg-brand/10' : 'text-text-muted hover:bg-surface-bg'"
     :title="visionMode ? '视觉分析已开启 (点击关闭并禁用上传)' : '视觉分析已关闭 (点击开启)'"
   >
     <svg v-if="visionMode" .../> <!-- 睁眼图标 -->
     <svg v-else .../> <!-- 闭眼图标 -->
   </button>

   <!-- 原有的添加图片按钮 -->
   <button
     v-if="visionMode"
     @click="fileInputRef?.click()"
     class="p-2 text-text-muted hover:text-brand transition-colors rounded-lg hover:bg-brand/10"
     title="添加图片"
   >
     <!-- ... -->
   </button>
   ```

## 4. 验证方案 (Verification Plan)
1. **功能可用性**：
   - 默认开启时，可以点击上传图片，可以粘贴图片。
   - 点击开关关闭后，上传图片按钮隐藏/禁用，粘贴图片无任何反应。
2. **状态联动**：
   - 先上传几张图片，然后点击关闭视觉开关，界面上已选的图片预览应立即清空。
3. **发送速度验证**：
   - 在关闭状态下，发送纯文本，应立即得到秒回（不调用后端 Vision 模型）。
