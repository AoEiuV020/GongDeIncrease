# GongDeIncrease
学习智能合约开发，做个带转账的counter,

1. 安装环境，
> https://solana.com/zh/docs/intro/installation
2. 创建项目，
```
anchor init --package-manager pnpm GongDeIncrease
```
3. 启动本地测试节点，
```
solana-test-validator
```
数据保存在test-ledger目录下， 很大， 啥也没干十几分钟就几百兆了，  
这体积有时候会变小，搞不懂，  
删除重启的话会给已有的钱包直接打5亿的初始资金，  

4. 连接本地服务器，
```
solana config set --url localhost
```
5. 查看配置，
```
solana config get
```
6. 创建钱包，
```
solana-keygen new
```
私钥保存在 ~/.config/solana/id.json  
查看地址，
```
solana address
```
查看余额，
```
solana balance
```
刚创建的没钱，  

7. 领空投，
```
solana airdrop 2
```
8. build，
```
anchor build
```
第一次会下载400M的platform-tools，  
最终产物 target/deploy/gong_de_increase.so 看起来就是个普通的64位动态库，186K，  

9. deploy,
```
anchor deploy
```
部署到 Anchor.toml 文件中指定的 cluster，默认localhost,  
消耗1.33sol，贵到离谱， 怎么回事，好像是存储费用，so文件太大？不应该用anchor而是应该直接写rust会小一些？  
部署会得到一个签名，可以使用solana命令查看详情，包括余额变化，  
```
solana confirm -v <SIGNATURE>
```
再调用就看不到了， 搞不懂，感觉像是会删除旧区块信息，  
最好能有图形化的工具查看本地，查看主链的网站也支持查看本地，  
> https://explorer.solana.com/address/3fENcwxPTHuHAu7jXkdCd7sFrEKkcUxqXwGc6bkwGDCK?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899

本地钱包换了之后就无法升级原本的程序，  
删除程序私钥 target/deploy/gong_de_increase-keypair.json 就能创建一个新的程序，  
那这gitignore的私钥很重要啊，得自己备份好，好像不能通过钱包恢复，  
用sync命令会创建新的并同步到 programs/gong-de-increase/src/lib.rs 和 Anchor.toml
```
anchor keys sync
```
可以直接查看地址，
```
solana address -k target/deploy/gong_de_increase-keypair.json
```
可以通过自己的钱包私钥查看自己名下所有程序，是有关联的， 就不知道私钥能不能重新生成，  
还能看到程序有余额，实际是在账户的Executable Data对应地址中，像是押金，  
```
solana program show --programs
```

10. test,
默认配置test会启动本地节点， 所以test之前要关闭已经运行了的本地节点，默认端口8899冲突了，  
或者加参数跳过启动服务就能连接已经启动了的本地服务，  
```
anchor test --skip-local-validator
```
11. 关闭程序，
取回押金，1.327sol，虽然可以退回，但是186K的程序要押金1.327sol目前行情价就是257.76usdt，太离谱了吧，  
```
solana program close <程序ID> --bypass-warning
```
12. dump程序，
导出部署的so文件，理所当然的关闭后就无法导出了，
```
solana program dump <程序ID> <输出文件路径>
