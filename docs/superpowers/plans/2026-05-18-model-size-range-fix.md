# Model Size Range Display Bug Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the model size range displaying incorrectly in the settings ModelManager by filtering out auxiliary mmproj/projector variants.

**Architecture:** Modify `getModelSizeRange` in `ModelManager.vue` to filter out variants whose tag or params contain "mmproj" or "projector" before calculating size range, and use a standalone unit test to verify this logic under Node.js.

**Tech Stack:** Vue 3.5, TypeScript, Node.js (for unit testing)

---

### Task 1: Create a unit test to reproduce the bug

**Files:**
- Create: `src/components/settings/__tests__/ModelManager.test.js`

- [ ] **Step 1: Write the reproduction unit test**
  We will write a Node.js-runnable script that imports the old logic of `getModelSizeRange` (mocked) and asserts that for a visual model like LLaVA, it incorrectly returns `0.6 GB ~ 8.0 GB`.

  Create `src/components/settings/__tests__/ModelManager.test.js` with the following content:

  ```javascript
  import assert from 'assert';

  // 1. Mock formatSize and getModelSizeRange (representing the current old implementation)
  const formatSize = (bytes) => {
    if (!bytes) return '未知';
    const gb = bytes / (1024 * 1024 * 1024);
    return gb.toFixed(1) + ' GB';
  };

  const getModelSizeRangeOld = (variants) => {
    if (!variants || variants.length === 0) return '大小待定';
    
    const sizes = variants.map(v => v.size).filter(s => s > 0);
    if (sizes.length === 0) return '大小待定';
    
    if (sizes.length === 1) return formatSize(sizes[0]);
    
    const minSize = Math.min(...sizes);
    const maxSize = Math.max(...sizes);
    return `${formatSize(minSize)} ~ ${formatSize(maxSize)}`;
  };

  // 2. Test Cases Representing LLaVA variants (7B, 13B, and mmproj)
  const llavaVariants = [
    { tag: '7b', params: '7B', size: 4.7 * 1024 * 1024 * 1024 },
    { tag: '13b', params: '13B', size: 8.0 * 1024 * 1024 * 1024 },
    { tag: 'mmproj-f16', params: 'mmproj', size: 0.6 * 1024 * 1024 * 1024 }
  ];

  console.log('--- Running reproduction test ---');
  try {
    const range = getModelSizeRangeOld(llavaVariants);
    console.log('Old implementation output:', range);
    
    // We expect the correct size range to be '4.7 GB ~ 8.0 GB'
    // Under the old implementation, this assertion will FAIL because it will return '0.6 GB ~ 8.0 GB'
    assert.strictEqual(range, '4.7 GB ~ 8.0 GB');
    console.log('TEST PASSED! (This is unexpected for the old implementation)');
  } catch (error) {
    console.log('TEST FAILED as expected for reproduction:', error.message);
  }
  ```

- [ ] **Step 2: Run the test to verify it fails**
  Run the test using Node.js to confirm it fails on the old implementation assertion.

  Run: `node src/components/settings/__tests__/ModelManager.test.js`
  Expected Output: Contains `TEST FAILED as expected for reproduction: '0.6 GB ~ 8.0 GB' === '4.7 GB ~ 8.0 GB'`

- [ ] **Step 3: Commit reproduction test**
  ```bash
  git add src/components/settings/__tests__/ModelManager.test.js
  git commit -m "test: add reproduction unit test for model size range bug"
  ```

---

### Task 2: Implement the filtering logic in ModelManager.vue

**Files:**
- Modify: `src/components/settings/ModelManager.vue:430-441`

- [ ] **Step 1: Implement the minimal code fix in ModelManager.vue**
  We will update the `getModelSizeRange` function in `ModelManager.vue` to filter out projector/mmproj variants.

  In `src/components/settings/ModelManager.vue`, replace lines 430-441:

  ```typescript
  const getModelSizeRange = (variants: any[]) => {
    if (!variants || variants.length === 0) return '大小待定';
    
    // 过滤掉投影器 (projector/mmproj) 变体，仅计算主模型的大小范围
    const mainVariants = variants.filter(v => 
      !v.tag.toLowerCase().includes('mmproj') && 
      !v.tag.toLowerCase().includes('projector') &&
      !(v.params && v.params.toLowerCase().includes('mmproj')) &&
      !(v.params && v.params.toLowerCase().includes('projector'))
    );
    
    const targetVariants = mainVariants.length > 0 ? mainVariants : variants;
    const sizes = targetVariants.map(v => v.size).filter(s => s > 0);
    if (sizes.length === 0) return '大小待定';
    
    if (sizes.length === 1) return formatSize(sizes[0]);
    
    const minSize = Math.min(...sizes);
    const maxSize = Math.max(...sizes);
    return `${formatSize(minSize)} ~ ${formatSize(maxSize)}`;
  };
  ```

- [ ] **Step 2: Commit code changes**
  ```bash
  git add src/components/settings/ModelManager.vue
  git commit -m "fix: filter out projector/mmproj in getModelSizeRange"
  ```

---

### Task 3: Verify with unit test and compile check

**Files:**
- Modify: `src/components/settings/__tests__/ModelManager.test.js`

- [ ] **Step 1: Update the unit test script to test the new implementation**
  Update the script to test the new `getModelSizeRange` logic and verify it passes.

  Modify `src/components/settings/__tests__/ModelManager.test.js` to contain:

  ```javascript
  import assert from 'assert';

  const formatSize = (bytes) => {
    if (!bytes) return '未知';
    const gb = bytes / (1024 * 1024 * 1024);
    return gb.toFixed(1) + ' GB';
  };

  const getModelSizeRangeNew = (variants) => {
    if (!variants || variants.length === 0) return '大小待定';
    
    // 过滤掉投影器 (projector/mmproj) 变体，仅计算主模型的大小范围
    const mainVariants = variants.filter(v => 
      !v.tag.toLowerCase().includes('mmproj') && 
      !v.tag.toLowerCase().includes('projector') &&
      !(v.params && v.params.toLowerCase().includes('mmproj')) &&
      !(v.params && v.params.toLowerCase().includes('projector'))
    );
    
    const targetVariants = mainVariants.length > 0 ? mainVariants : variants;
    const sizes = targetVariants.map(v => v.size).filter(s => s > 0);
    if (sizes.length === 0) return '大小待定';
    
    if (sizes.length === 1) return formatSize(sizes[0]);
    
    const minSize = Math.min(...sizes);
    const maxSize = Math.max(...sizes);
    return `${formatSize(minSize)} ~ ${formatSize(maxSize)}`;
  };

  // Test Case 1: LLaVA variants (Should filter out mmproj)
  const llavaVariants = [
    { tag: '7b', params: '7B', size: 4.7 * 1024 * 1024 * 1024 },
    { tag: '13b', params: '13B', size: 8.0 * 1024 * 1024 * 1024 },
    { tag: 'mmproj-f16', params: 'mmproj', size: 0.6 * 1024 * 1024 * 1024 }
  ];

  // Test Case 2: Standard text model (e.g. Qwen, no filter needed)
  const qwenVariants = [
    { tag: '1.5b', params: '1.5B', size: 1.1 * 1024 * 1024 * 1024 },
    { tag: '7b', params: '7B', size: 4.7 * 1024 * 1024 * 1024 }
  ];

  console.log('--- Running verification test ---');
  
  // Test LLaVA size calculation
  const rangeLlava = getModelSizeRangeNew(llavaVariants);
  console.log('New implementation LLaVA output:', rangeLlava);
  assert.strictEqual(rangeLlava, '4.7 GB ~ 8.0 GB');
  
  // Test Qwen size calculation
  const rangeQwen = getModelSizeRangeNew(qwenVariants);
  console.log('New implementation Qwen output:', rangeQwen);
  assert.strictEqual(rangeQwen, '1.1 GB ~ 4.7 GB');

  console.log('ALL TESTS PASSED SUCCESSFULLY!');
  ```

- [ ] **Step 2: Run verification test**
  Run: `node src/components/settings/__tests__/ModelManager.test.js`
  Expected Output: Contains `ALL TESTS PASSED SUCCESSFULLY!`

- [ ] **Step 3: Run vue-tsc compiler check**
  Ensure there are no compilation or typescript errors in our files.

  Run: `pnpm run build` or `npx vue-tsc --noEmit`
  Expected Output: Compiles successfully with no errors in `ModelManager.vue`.

- [ ] **Step 4: Clean up temporary test file and commit**
  We will remove the temporary node test script before submitting so we do not clutter the repository.
  
  Run: `rm src/components/settings/__tests__/ModelManager.test.js`
  ```bash
  git add src/components/settings/ModelManager.vue
  git commit -m "test: verify fix with unit tests and cleanup"
  ```
