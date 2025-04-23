import * as anchor from "@project-serum/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

// 假设 program 已经初始化并且 provider 是当前连接的钱包
const musicId = new anchor.BN(11); // 示例音乐ID
const name = "Sample Song";
const price = new anchor.BN(1000000); // 示例价格

// 计算音乐PDA和bump
// 将musicId转换为与合约中一致的格式 (to_be_bytes)
// 在Rust中，to_be_bytes()返回固定长度的大端序字节数组
// 正确转换musicId为字节数组，匹配Rust的to_be_bytes()
const musicIdBuffer = Buffer.alloc(8);
musicId.toArrayLike(Buffer, "be", 8).copy(musicIdBuffer); // 使用大端序(be)来匹配Rust的to_be_bytes()
const [musicPda, musicBump] = await PublicKey.findProgramAddress(
  [Buffer.from("music"), musicIdBuffer],
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

// 新增：使用 Token 购买音乐的函数
async function buyMusicWithToken(
  buyerTokenAccount: PublicKey,
  ownerTokenAccount: PublicKey
) {
  try {
    await pg.program.methods
      .buyMusicToken(musicId)
      .accounts({
        music: musicPda,
        buyer: buyerPda,
        payer: pg.wallet.publicKey,
        buyerTokenAccount: buyerTokenAccount,
        ownerTokenAccount: ownerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([pg.wallet.keypair])
      .rpc();

    console.log("Music purchased successfully with token.");
    return true;
  } catch (error) {
    console.error("Error purchasing music with token:", error);
    return false;
  }
}

// 不再需要sleep函数

// 执行上传和购买操作
async function main() {
  // 假设已经获取到买家和所有者的 Token 账户
  const buyerTokenAccount = new PublicKey(
    "iMqV2UhrefEibCem3TvBwe95K1boFAHmPqN5SWZXnVM"
  );
  const ownerTokenAccount = new PublicKey(
    "iMqV2UhrefEibCem3TvBwe95K1boFAHmPqN5SWZXnVM"
  );
  await buyMusicWithToken(buyerTokenAccount, ownerTokenAccount);
  // // 使用回调方式处理上传结果
  // uploadMusic(async (uploaded) => {
  //   if (uploaded) {
  //     // 确保上传成功后再购买
  //     console.log("Proceeding to purchase music...");
  //     // 假设已经获取到买家和所有者的 Token 账户
  //     const buyerTokenAccount = new PublicKey("iMqV2UhrefEibCem3TvBwe95K1boFAHmPqN5SWZXnVM");
  //     const ownerTokenAccount = new PublicKey("iMqV2UhrefEibCem3TvBwe95K1boFAHmPqN5SWZXnVM");
  //     await buyMusicWithToken(buyerTokenAccount, ownerTokenAccount);
  //   } else {
  //     console.log("Skipping purchase because upload failed.");
  //   }
  // });
}

main().catch((err) => console.error(err));
