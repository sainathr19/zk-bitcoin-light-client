import * as bip39 from 'bip39';
import * as bitcoin from 'bitcoinjs-lib';
import { BIP32Factory } from 'bip32';
import * as ecc from 'tiny-secp256k1';
import { ethers } from 'ethers';

// Initialize BIP32 with secp256k1
const bip32 = BIP32Factory(ecc);

export class AddressGenerator {
  private static mnemonic: string;

  static initialize(mnemonic: string): void {
    if (!bip39.validateMnemonic(mnemonic)) {
      throw new Error('Invalid mnemonic phrase');
    }
    this.mnemonic = mnemonic;
    console.log('✅ Address generator initialized with mnemonic');
  }

  static generateBitcoinAddress(index: number): string {
    if (!this.mnemonic) {
      throw new Error('Address generator not initialized. Call initialize() first.');
    }

    try {
      // Generate seed from mnemonic
      const seed = bip39.mnemonicToSeedSync(this.mnemonic);
      
      // Create HD wallet using BIP32
      const hdWallet = bip32.fromSeed(seed);
      
      // Derive address using BIP44 path: m/44'/0'/0'/0/index
      const path = `m/44'/0'/0'/0/${index}`;
      const child = hdWallet.derivePath(path);
      
      // Generate P2PKH address (legacy)
      const { address } = bitcoin.payments.p2pkh({ 
        pubkey: child.publicKey,
        network: bitcoin.networks.bitcoin 
      });
      
      return address!;
    } catch (error) {
      console.error('Error generating Bitcoin address:', error);
      throw new Error('Failed to generate Bitcoin address');
    }
  }

  static generateEthereumAddress(index: number): string {
    if (!this.mnemonic) {
      throw new Error('Address generator not initialized. Call initialize() first.');
    }

    try {
      // Generate HD wallet from mnemonic
      const hdNode = ethers.HDNodeWallet.fromPhrase(this.mnemonic);
      
      // Derive address using BIP44 path: m/44'/60'/0'/0/index
      const path = `m/44'/60'/0'/0/${index}`;
      const derivedWallet = hdNode.derivePath(path);
      
      return derivedWallet.address;
    } catch (error) {
      console.error('Error generating Ethereum address:', error);
      throw new Error('Failed to generate Ethereum address');
    }
  }

  static generateAddressesForOrder(orderIndex: number): { source_address: string; destination_address: string } {
    // Generate Bitcoin address for source (even indices)
    const sourceAddress = this.generateBitcoinAddress(orderIndex * 2);
    
    // Generate Ethereum address for destination (odd indices)
    const destinationAddress = this.generateEthereumAddress(orderIndex * 2 + 1);
    
    console.log(`🔑 Generated addresses for order ${orderIndex}:`);
    console.log(`   Source (BTC): ${sourceAddress}`);
    console.log(`   Destination (ETH): ${destinationAddress}`);
    
    return {
      source_address: sourceAddress,
      destination_address: destinationAddress
    };
  }
}
