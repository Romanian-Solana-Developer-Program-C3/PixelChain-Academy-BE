import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("pixel-chain-anchor", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PixelChainAnchor;
  const [playerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  it("Initializes a player", async () => {
    await program.methods.initPlayer().accounts({ player: playerPda }).rpc();
    const player = await program.account.player.fetch(playerPda);
    expect(player.xp).to.equal(0);
  });
});
