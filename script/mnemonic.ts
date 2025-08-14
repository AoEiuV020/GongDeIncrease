import * as bip39 from "bip39";
import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import * as readline from "readline/promises";
import { stdin as input, stdout as output } from "process";

async function main() {
  const rl = readline.createInterface({ input, output });

  // 输入助记词
  const inputMnemonic = await rl.question("请输入助记词: ");

  // 判断是否为ASCII字符（英文助记词）
  const isAscii = /^[\x00-\x7F]+$/.test(inputMnemonic);
  let mnemonic: string;
  let entropy: string;

  if (isAscii) {
    // 英文助记词
    if (!bip39.validateMnemonic(inputMnemonic)) {
      console.error("无效的英文助记词");
      process.exit(1);
    }
    mnemonic = inputMnemonic;
    entropy = bip39.mnemonicToEntropy(mnemonic);
  } else {
    // 中文助记词
    if (
      !bip39.validateMnemonic(inputMnemonic, bip39.wordlists.chinese_simplified)
    ) {
      console.error("无效的中文助记词");
      process.exit(1);
    }
    entropy = bip39.mnemonicToEntropy(
      inputMnemonic,
      bip39.wordlists.chinese_simplified
    );
    mnemonic = bip39.entropyToMnemonic(entropy); // 转换为英文助记词
  }

  // 输入密码（可选）
  const password = await rl.question("请输入密码（可选，直接回车跳过）: ");

  // 显示助记词信息
  console.log("\n助记词的熵:", entropy);
  console.log("英文助记词:", mnemonic);
  const chineseMnemonic = bip39.entropyToMnemonic(
    entropy,
    bip39.wordlists.chinese_simplified
  );
  console.log("中文助记词:", chineseMnemonic);

  // 生成种子
  const seed = bip39.mnemonicToSeedSync(mnemonic, password);
  console.log("生成的种子:", seed.toString("hex"));

  // 创建 Solana 密钥对
  // 注意：我们只使用种子的前32字节，因为 Solana 密钥对需要32字节的种子
  const keypair = Keypair.fromSeed(seed.subarray(0, 32));

  // 打印公钥和私钥（Base58格式）
  console.log("\nSolana 公钥 (Base58):", keypair.publicKey.toBase58());
  console.log("Solana 私钥 (Base58):", bs58.encode(keypair.secretKey));

  // 打印私钥（JSON数组格式）
  console.log("\nSolana 私钥 (JSON数组):");
  console.log(JSON.stringify(Array.from(keypair.secretKey)));

  rl.close();
}

main().catch(console.error);
