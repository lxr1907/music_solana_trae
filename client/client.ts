import * as anchor from "@project-serum/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";

// 假设 program 已经初始化并且 provider 是当前连接的钱包
const musicId = new anchor.BN(4); // 示例音乐ID
const name = "Sample Song";
const price = new anchor.BN(1000000); // 示例价格

// 计算音乐 PDA 和 bump
const [musicPda, musicBump] = await PublicKey.findProgramAddress(
  [Buffer.from("music"), musicId.toArrayLike(Buffer, "be", 8)],
  pg.program.programId
);

// 计算买家PDA和bump
const [buyerPda, buyerBump] = await PublicKey.findProgramAddress(
  [Buffer.from("buyer"), pg.wallet.publicKey.toBuffer()],
  pg.program.programId
);

// 注意：不再需要单独初始化买家账户，该功能已整合到buyMusic方法中

// 上传音乐函数 - 使用回调方式模拟同步执行
function uploadMusic(callback) {
  try {
    pg.program.methods
      .uploadMusic(musicId, name, price, pg.wallet.publicKey, musicBump)
      .accounts({
        signer: pg.wallet.publicKey,
        music: musicPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([pg.wallet.keypair])
      .rpc()
      .then(() => {
        console.log("Music uploaded successfully.");
        callback(true);
      })
      .catch((error) => {
        console.error("Error uploading music:", error);
        callback(false);
      });
  } catch (error) {
    console.error("Error initializing upload:", error);
    callback(false);
  }
}

// 购买音乐的函数
async function buyMusic() {
  try {
    // 调用buyMusic方法
    await pg.program.methods
      .buyMusic(musicId)
      .accounts({
        music: musicPda,
        buyer: buyerPda, // 使用初始化的买家PDA账户
        payer: pg.wallet.publicKey,

        systemProgram: SystemProgram.programId,
      })
      .signers([pg.wallet.keypair])
      .rpc();
    
    console.log("Music purchased successfully.");
    return true;
  } catch (error) {
    console.error("Error purchasing music:", error);
    return false;
  }
}

// 不再需要sleep函数

// 执行上传和购买操作
async function main() {
  // 使用回调方式处理上传结果
  uploadMusic((uploaded) => {
    if (uploaded) {
      // 确保上传成功后再购买
      console.log("Proceeding to purchase music...");
      // 直接调用购买方法，不再使用sleep延迟
      buyMusic();
    } else {
      console.log("Skipping purchase because upload failed.");
    }
  });
}

main().catch(err => console.error(err));