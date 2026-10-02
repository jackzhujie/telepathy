# [2026-04-15] 后台管理系统全面解锁方案 (Backend Admin System Unlock)

## 1. 背景 (Context)
用户在 `SmartVote` 小程序后台提交活动或进行其他管理操作时，遇到弹窗提示：“该功能暂不开放，如有需要请加作者微信：cclinux0730”。经排查，该限制是由云函数 Service 层硬编码的 `this.AppError` 调用引起的。

## 2. 目标 (Goals)
- 移除所有业务 Service 层中的硬编码报错限制。
- 恢复受限功能的正常业务逻辑（CRUD 操作）。
- 确保后台管理系统的完整可用性。

## 3. 详细方案 (Detailed Design)

### 3.1 解锁策略
我们将针对不同类型的受限方法，采用以下恢复逻辑：

| 功能类型 | 原始限制代码 | 恢复后的逻辑 (伪代码) |
| :--- | :--- | :--- |
| **状态修改** | `this.AppError(...)` | `await Model.edit(id, { STATUS: status })` |
| **排序设定** | `this.AppError(...)` | `await Model.edit(id, { ORDER: sort })` |
| **推荐/置顶** | `this.AppError(...)` | `await Model.edit(id, { VOUCH: vouch })` |
| **数据清理/删除** | `this.AppError(...)` | `await Model.del(id)` 或对应的逻辑操作 |
| **数据统计/导出**| `this.AppError(...)` | 调用底层 `exportUtil` 或计算逻辑 |

### 3.2 涉及文件清单
我们将对以下路径下的文件进行全面清理：
- `/cloudfunctions/mcloud/project/VOTE1/service/admin/admin_vote_service.js`
- `/cloudfunctions/mcloud/project/VOTE1/service/admin/admin_home_service.js`
- `/cloudfunctions/mcloud/project/VOTE1/service/admin/admin_news_service.js`
- `/cloudfunctions/mcloud/project/VOTE1/service/admin/admin_user_service.js`
- `/cloudfunctions/mcloud/project/VOTE1/service/admin/admin_mgr_service.js`

### 3.3 预期变更
以 `AdminVoteService.js` 为例：
- `statVoteAll`: 恢复统计逻辑。
- `sortVote`: 恢复 `VOTE_ORDER` 更新。
- `statusVote`: 恢复 `VOTE_STATUS` 更新。
- `exportVoteDataExcel`: 恢复调用 `exportUtil.exportDataExcel`。

## 4. 验证计划 (Verification Plan)
- **代码静态验证**: 确保替换后的 Model 方法调用语法正确且符合框架规范。
- **关联逻辑检查**: 确保 `insertVote` 后可能调用的所有后续逻辑路径均无硬编码阻断。

## 5. 用户关注点 (User Review Required)
> [!IMPORTANT]
> 解锁这些功能将直接操作云数据库。虽然目前的逻辑是标准的 CRUD，但为了安全起见，建议您在确认我的修改后，在真实环境操作前先备份现有的云数据库。
