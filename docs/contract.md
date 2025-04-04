# 音乐商店智能合约文档

## 合约概述

这是一个基于Solana区块链的音乐商店智能合约，实现了音乐NFT的铸造、购买和版税分配功能。合约使用PLAY代币作为支付媒介，并支持多方版税分成。

## 技术架构

### 代币系统
- 使用SPL Token标准
- PLAY代币作为平台支付媒介
- 支持用户使用SOL购买PLAY代币
- 使用PDA（Program Derived Address）作为代币铸造权限

### 账户结构

#### Music账户
```rust
pub struct Music {
    pub id: u64,            // 音乐唯一标识符
    pub name: String,        // 音乐名称
    pub price: u64,         // 音乐价格
    pub owner: Pubkey,      // 音乐所有者
    pub royalties: Vec<(Pubkey, u8)>, // 版税分配列表
}
```

#### Buyer账户
```rust
pub struct Buyer {
    pub purchased_music_ids: Vec<u64>, // 已购买的音乐ID列表
}
```

## 主要功能

### 1. 代币管理

#### 初始化代币 (initialize_token)
- 创建PLAY代币的Mint账户
- 设置代币精度为6位小数
- 使用PDA作为铸币权限

#### 购买PLAY代币 (buy_play_tokens)
- 用户支付0.1 SOL获取100,000 PLAY代币
- 验证用户SOL余额
- 执行SOL转账到金库账户
- 铸造PLAY代币到用户账户

### 2. 用户管理

#### 初始化买家账户 (initialize_buyer)
- 创建用户账户存储购买记录
- 验证用户PLAY代币余额（至少1,000,000 PLAY）

### 3. 音乐管理

#### 上传音乐 (upload_music)
- 创建音乐账户
- 设置音乐基本信息（ID、名称、价格）
- 配置版税分配方案
- 验证版税总和为100%

#### 购买音乐 (buy_music)
- 验证音乐ID和重复购买
- 验证分账账户信息
- 执行版税分配转账
- 更新用户购买记录

#### 查询购买记录 (has_purchased)
- 检查用户是否已购买特定音乐

## 错误处理

合约定义了以下错误类型：
- MusicNotFound: 音乐不存在
- AlreadyPurchased: 用户已购买该音乐
- InvalidRoyalties: 无效的版税配置（总和必须为100%）
- InvalidRoyaltyAccounts: 无效的版税账户
- InvalidRoyaltyAccount: 无效的版税账户地址
- AccountNotWritable: 版税账户不可写
- InsufficientTokenBalance: PLAY代币余额不足
- InsufficientFunds: SOL余额不足

## 使用示例

1. 初始化系统
```bash
# 初始化PLAY代币
solana program call initialize_token

# 初始化买家账户
solana program call initialize_buyer
```

2. 上传音乐
```bash
# 上传音乐并设置版税分配
solana program call upload_music \
    --music-id 1 \
    --name "My Song" \
    --price 1000 \
    --royalties '[{"address":"owner1","percent":60},{"address":"owner2","percent":40}]'
```

3. 购买音乐
```bash
# 购买音乐
solana program call buy_music --music-id 1
```

## 安全考虑

1. 权限控制
- 使用PDA控制代币铸造权限
- 验证所有账户签名和权限

2. 资金安全
- 严格的余额检查
- 精确的版税分配计算
- 防止重复购买

3. 数据验证
- 版税总和必须为100%
- 所有账户地址验证
- 交易金额验证

## 注意事项

1. PLAY代币
- 初始购买比例：0.1 SOL = 100,000 PLAY
- 最低账户余额要求：1,000,000 PLAY

2. 版税分配
- 支持多方分成
- 分成比例必须精确到100%
- 最后一个接收方获得余额处理