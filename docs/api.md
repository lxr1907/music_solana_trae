# 兑换合约 API 文档

## 概述
本文档描述了兑换合约的接口规范，包括所有指令的详细说明、所需账户列表、参数以及响应示例。

## 指令列表

### 1. 初始化兑换合约
**指令**: `Initialize`

**参数**:
- `rate`: u64 - 兑换比例 (1 SOL = rate 个自定义 token)

**所需账户**:
1. `[signer]` 管理员账户
2. `[writable]` 兑换状态账户
3. `[]` Token mint 账户
4. `[writable]` 合约 Token 账户
5. `[]` Token 程序 ID
6. `[]` 系统程序 ID
7. `[]` Rent sysvar

**成功响应**:
```
兑换合约初始化成功，兑换比例: 1 SOL = 100 个自定义 token
```

**错误响应**:
- `InvalidExchangeRate`: 兑换比例必须大于零
- `MissingRequiredSignature`: 管理员未签名

---

### 2. 更新兑换比例
**指令**: `UpdateRate`

**参数**:
- `new_rate`: u64 - 新的兑换比例

**所需账户**:
1. `[signer]` 管理员账户
2. `[writable]` 兑换状态账户

**成功响应**:
```
兑换比例已更新: 1 SOL = 150 个自定义 token
```

**错误响应**:
- `InvalidExchangeRate`: 兑换比例必须大于零
- `NotAdmin`: 非管理员账户
- `MissingRequiredSignature`: 管理员未签名

---

### 3. SOL 兑换为 Token
**指令**: `ExchangeSolToToken`

**参数**:
- `amount`: u64 - 要兑换的 SOL 数量 (lamports)

**所需账户**:
1. `[signer]` 用户账户
2. `[]` 兑换状态账户
3. `[writable]` 合约 Token 账户
4. `[writable]` 用户 Token 账户
5. `[]` Token 程序 ID

**兑换计算示例**:
- 当前汇率: 1 SOL = 100 token
- 用户兑换: 2 SOL
- 获得: 200 token

**成功响应**:
```
兑换成功: 2 SOL -> 200 个自定义 token
```

**错误响应**:
- `InsufficientTokenBalance`: Token 余额不足

---

### 4. Token 兑换为 SOL
**指令**: `ExchangeTokenToSol`

**参数**:
- `amount`: u64 - 要兑换的 Token 数量

**所需账户**:
1. `[signer]` 用户账户
2. `[]` 兑换状态账户
3. `[writable]` 合约 Token 账户
4. `[writable]` 用户 Token 账户
5. `[]` Token 程序 ID

**兑换计算示例**:
- 当前汇率: 1 SOL = 100 token
- 用户兑换: 300 token
- 获得: 3 SOL

**成功响应**:
```
兑换成功: 300 token -> 3 SOL (汇率: 1 SOL = 100 token)
```

**错误响应**:
- `InsufficientSolBalance`: SOL 余额不足
- `ArithmeticOverflow`: 兑换计算溢出

## 常见问题
1. **如何计算兑换金额**
   - SOL → Token: `token_amount = sol_amount * rate`
   - Token → SOL: `sol_amount = token_amount / rate`

2. **权限要求**
   - 只有管理员可以初始化和更新汇率
   - 任何用户都可以进行兑换操作