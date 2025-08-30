import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import fs from "fs";
import path from "path";
import { homedir } from "os";

// 获取命令行参数或使用默认路径
const keyPath =
  process.argv[2] || path.join(homedir(), ".config", "solana", "id.json");

// 从文件读取私钥
function loadKeypairFromFile(filePath: string): Keypair {
  const content = fs.readFileSync(filePath, { encoding: "utf-8" });
  const secretKey = Uint8Array.from(JSON.parse(content));
  return Keypair.fromSecretKey(secretKey);
}

// 主函数
async function main() {
  try {
    const keypair = loadKeypairFromFile(keyPath);

    console.log("=== Solana 密钥信息 ===");
    console.log("公钥 (Public Key):", keypair.publicKey.toBase58());
    console.log("私钥 (Base58):", bs58.encode(keypair.secretKey));
  } catch (error) {
    console.error("错误：", error);
    console.log("请确保 id.json 文件存在并包含有效的 Solana 私钥");
  }
}

main().catch(console.error);
