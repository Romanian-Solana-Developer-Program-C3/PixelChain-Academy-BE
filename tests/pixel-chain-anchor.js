import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("pixel-chain-anchor", () => {
  // 1) setează provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 2) reference la program
  const program = anchor.workspace.PixelChainAnchor;

  // 3) PDA-ul jucătorului
  const [playerPda, playerBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  it("Initializes a player", async () => {
    // 4) apelăm metoda, trimitem toate conturile
    await program.methods
      .initPlayer()
      .accounts({
        player: playerPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // 5) citim contul și verificăm xp
    const playerAccount = await program.account.player.fetch(playerPda);
    expect(playerAccount.xp).to.equal(0);
  });
});
