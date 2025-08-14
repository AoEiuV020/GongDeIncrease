import { readFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";
import * as bs58 from "bs58";

// 获取命令行参数或使用默认路径
const keyPath =
  process.argv[2] || join(homedir(), ".config", "solana", "id.json");

try {
  // 读取文件内容
  const fileContent = readFileSync(keyPath, "utf-8");

  // 解析 JSON 数组
  const numbers = JSON.parse(fileContent);

  // 转换为Uint8Array
  const bytes = new Uint8Array(numbers);

  // 转换为 base58
  const base58String = bs58.default.encode(bytes);

  console.log(base58String);
} catch (error) {
  console.error("错误:", error.message);
  process.exit(1);
}
