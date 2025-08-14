import * as bip39 from 'bip39';
import { Keypair } from '@solana/web3.js';
import bs58 from 'bs58';
import * as readline from 'readline/promises';
import { stdin as input, stdout as output } from 'process';

async function main() {
    const rl = readline.createInterface({ input, output });
    
    // 输入助记词
    const mnemonic = await rl.question('请输入助记词: ');
    if (!bip39.validateMnemonic(mnemonic)) {
        console.error('无效的助记词');
        process.exit(1);
    }

    // 输入密码（可选）
    const password = await rl.question('请输入密码（可选，直接回车跳过）: ');
    
    // 获取助记词的熵
    const entropy = bip39.mnemonicToEntropy(mnemonic);
    console.log('\n助记词的熵:', entropy);
    
    // 生成种子
    const seed = bip39.mnemonicToSeedSync(mnemonic, password);
    console.log('生成的种子:', seed.toString('hex'));
    
    // 创建 Solana 密钥对
    // 注意：我们只使用种子的前32字节，因为 Solana 密钥对需要32字节的种子
    const keypair = Keypair.fromSeed(seed.slice(0, 32));
    
    // 打印公钥和私钥（Base58格式）
    console.log('\nSolana 公钥 (Base58):', keypair.publicKey.toBase58());
    console.log('Solana 私钥 (Base58):', bs58.encode(keypair.secretKey));
    
    // 打印私钥（JSON数组格式）
    console.log('\nSolana 私钥 (JSON数组):');
    console.log(JSON.stringify(Array.from(keypair.secretKey)));
    
    rl.close();
}

main().catch(console.error);
