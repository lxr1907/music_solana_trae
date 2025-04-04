import * as anchor from "@project-serum/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";

// 假设 program 已经初始化并且 provider 是当前连接的钱包
const musicId = new anchor.BN(4); // 示例音乐ID
const name = "Sample Song";
const price = new anchor.BN(1000000); // 示例价格
const beneficiary = new PublicKey(
  "94cb4RwtpHESGGfLeiDG4HtfrFDSMEH81Siox3LW2DUP"
); // 替换为实际的受益人公钥

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

// 上传音乐
try {
  await pg.program.methods
    .uploadMusic(musicId, name, price, beneficiary, musicBump)
    .accounts({
      signer: pg.wallet.publicKey,
      music: musicPda,
      systemProgram: SystemProgram.programId,
    })
    .signers([pg.wallet.keypair])
    .rpc();
  console.log("Music uploaded successfully.");
} catch (error) {
  console.error("Error uploading music:", error);
}
// 购买音乐的函数
try {
  // 调用buyMusic方法
  await pg.program.methods
    .buyMusic(musicId)
    .accounts({
      music: musicPda,
      buyer: buyerPda, // 使用初始化的买家PDA账户
      payer: pg.wallet.publicKey,
      beneficiary: beneficiary,
      systemProgram: SystemProgram.programId,
    })
    .signers([pg.wallet.keypair])
    .rpc();
  
  console.log("Music purchased successfully.");
} catch (error) {
  console.error("Error purchasing music:", error);
}