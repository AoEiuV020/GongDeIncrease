# Solana 智能合约开发

基于原生Rust的Solana智能合约开发项目。

## 环境安装

> https://solana.com/zh/docs/intro/installation

## 本地开发环境

### 启动本地测试节点

```bash
solana-test-validator
```

数据保存在test-ledger目录下，删除重启会给已有钱包直接打5亿初始资金。

### 连接本地服务器

```bash
solana config set --url localhost
```

### 查看配置

```bash
solana config get
```

## 钱包管理

### 创建钱包

```bash
solana-keygen new
```

私钥保存在 ~/.config/solana/id.json

### 查看地址

```bash
solana address
```

### 查看余额

```bash
solana balance
```

### 领空投

```bash
solana airdrop 2
```

### 转账

```bash
solana transfer <接收地址> <金额>
```

## 程序开发

### 构建程序

```bash
cargo build-sbf
```

原生rust程序只需依赖solana-program，程序体积更小，仅一行日志的程序打包后只有20K。

### 部署程序

```bash
solana program deploy ./target/deploy/gong_de_increase.so
```

部署会得到一个签名，可以查看详情：

```bash
solana confirm -v <SIGNATURE>
```

### 更新程序

直接使用首次部署完全一样的命令即可，  
注意账户中需要有足够完整部署的资金，事后旧版的押金会退回，  
也就是说为了开发一个押金为1sol的程序，我一共需要准备2sol的资金，  

### 测试程序

```bash
cargo test -- --no-capture
```

### 运行示例

#### 创建和增加功德
```bash
cargo run --example client
```

### 关闭程序

取回押金：

```bash
solana program close <程序ID> --bypass-warning
```

### 导出程序

```bash
solana program dump <程序ID> <输出文件路径>
```

## 程序管理

### 查看所有程序

```bash
solana program show --programs
```

### 查看程序地址

```bash
solana address -k target/deploy/gong_de_increase-keypair.json
```

## 私钥和安全

### 项目内部私钥

可以通过alias让solana读取相对路径的config：

```bash
alias solana='solana -C ./.config/solana/cli/config.yml'
```

创建私钥后需要手动转移：

```bash
mv ~/.config/solana/id.json ./id.json
```

### 导出私钥

钱包使用base58编码，cli使用json格式：

```bash
pnpx tsx script/exportPrivateKey.ts
```

### 解析助记词

```bash
pnpm tsx script/mnemonic.ts
```

**注意：直接打印私钥时要小心，切勿泄露，不要使用重要的钱包测试！**

## 开发工具

### 本地区块链浏览器

查看主链的网站也支持查看本地：
> https://explorer.solana.com/address/BvpjTs88TmXJrFfghPJmo1kEJXdtqXX8SdvW6jv8ng9R?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899

### 安全提醒

为了方便solana命令行使用，不可避免在本地明文保存私钥，并且可以被固定路径找到：
- `solana config get` 找到config文件
- 读取config文件中的keypair_path找到私钥文件  
- 读取私钥文件内容，得到私钥

配置项目内部私钥后，相关命令只能在项目根目录执行才能正确识别钱包。

### counter

可以通过PDA获取用户绑定的counter账户用来+1,  
但是这个功能太占用体积了，所以想办法把相关代码下放到客户端中而不是合约中。  

### counter账户租金rent

和智能合约程序的租金一样，根据数据量需要不同的租金，重点是非线性，  
counter账户从8字节调整到4字节，租金从 0.00094656 SOL 降到 0.00091872 SOL，没啥区别，  

```bash
solana rent 4
```

### PDA账户数据安全

创建PDA账户时要指定合约程序ID，后续所有对该账户的操作都必须由该合约程序发起，这样可以确保账户数据的安全性和一致性。  
```rust
    let gongde_pubkey = Pubkey::create_with_seed(
        &config.keypair.pubkey(),  // 基础地址（用户公钥）
        seed,                      // 种子字符串
        &config.program_id,        // 合约程序ID
    )?;
```

### 查询别人的功德

PDA账户只需要用户公钥+程序ID+种子字符串就能计算出来，不需要私钥。所以可以随便查询任何人的功德。  

### 给创作者打钱

这个打钱是系统指令， 一旦使用就会导致打包出的程序体积暴涨， 最终是81K，对应押金 0.57033024 SOL，  
主要是不能直接编辑账户的余额，因为没有权限，哪怕是增加都不行，  
还有就是这个地址不能只在合约中写死，客户端调用时传递的账户列表中必须包含所有设计到的账户，  

### 全局账户

必须在合约中创建的PDA账户才是合约能完全掌控的，  
而这个创建PDA账户同样是系统指令，就算没有前面加的转账功能， 这几十K的体积膨胀还是不可避免，  
