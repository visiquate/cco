#!/usr/bin/env node

/**
 * Secure Credential Manager
 *
 * Manages credentials for the Claude Orchestra, ensuring secure storage,
 * retrieval, and tracking of all secrets used across agents.
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

class CredentialManager {
  constructor() {
    // Use /tmp for temporary credential storage during development
    this.credentialPath = '/tmp/credentials.json';
    this.inventoryPath = path.join(__dirname, '../config/credential-inventory.json');
    this.encryptionKey = this.deriveKey();
  }

  /**
   * Derive encryption key from environment or generate temporary one
   */
  deriveKey() {
    const envKey = process.env.CREDENTIAL_ENCRYPTION_KEY;
    if (envKey) {
      return crypto.createHash('sha256').update(envKey).digest();
    }

    // Temporary key for development (NOT for production)
    console.warn('⚠️  Using temporary encryption key. Set CREDENTIAL_ENCRYPTION_KEY for production.');
    return crypto.randomBytes(32);
  }

  /**
   * Encrypt a credential
   */
  encrypt(text) {
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv('aes-256-cbc', this.encryptionKey, iv);

    let encrypted = cipher.update(text, 'utf8', 'hex');
    encrypted += cipher.final('hex');

    return {
      iv: iv.toString('hex'),
      data: encrypted
    };
  }

  /**
   * Decrypt a credential
   */
  decrypt(encrypted) {
    const iv = Buffer.from(encrypted.iv, 'hex');
    const decipher = crypto.createDecipheriv('aes-256-cbc', this.encryptionKey, iv);

    let decrypted = decipher.update(encrypted.data, 'hex', 'utf8');
    decrypted += decipher.final('utf8');

    return decrypted;
  }

  /**
   * Store a credential securely
   */
  async storeCredential(key, value, metadata = {}) {
    let credentials = {};

    // Load existing credentials
    if (fs.existsSync(this.credentialPath)) {
      const content = fs.readFileSync(this.credentialPath, 'utf8');
      credentials = JSON.parse(content);
    }

    // Encrypt and store
    credentials[key] = {
      encrypted: this.encrypt(value),
      metadata: {
        ...metadata,
        created: new Date().toISOString(),
        lastAccessed: new Date().toISOString()
      }
    };

    // Write securely
    fs.writeFileSync(this.credentialPath, JSON.stringify(credentials, null, 2), {
      mode: 0o600 // Only owner can read/write
    });

    // Update inventory
    await this.updateInventory(key, metadata);

    console.log(`✅ Credential '${key}' stored securely`);
    return true;
  }

  /**
   * Retrieve a credential
   */
  async retrieveCredential(key) {
    if (!fs.existsSync(this.credentialPath)) {
      throw new Error('No credentials file found');
    }

    const content = fs.readFileSync(this.credentialPath, 'utf8');
    const credentials = JSON.parse(content);

    if (!credentials[key]) {
      throw new Error(`Credential '${key}' not found`);
    }

    // Update last accessed time
    credentials[key].metadata.lastAccessed = new Date().toISOString();
    fs.writeFileSync(this.credentialPath, JSON.stringify(credentials, null, 2), {
      mode: 0o600
    });

    // Decrypt and return
    const decrypted = this.decrypt(credentials[key].encrypted);
    console.log(`📖 Retrieved credential '${key}'`);

    return decrypted;
  }

  /**
   * Update credential inventory
   */
  async updateInventory(key, metadata) {
    let inventory = { credentials: {} };

    if (fs.existsSync(this.inventoryPath)) {
      inventory = JSON.parse(fs.readFileSync(this.inventoryPath, 'utf8'));
    }

    inventory.credentials[key] = {
      type: metadata.type || 'generic',
      service: metadata.service || 'unknown',
      description: metadata.description || '',
      rotationRequired: metadata.rotationRequired || false,
      lastRotation: metadata.lastRotation || null,
      expiresAt: metadata.expiresAt || null,
      addedAt: new Date().toISOString()
    };

    // Ensure config directory exists
    const configDir = path.dirname(this.inventoryPath);
    if (!fs.existsSync(configDir)) {
      fs.mkdirSync(configDir, { recursive: true });
    }

    fs.writeFileSync(this.inventoryPath, JSON.stringify(inventory, null, 2));
  }

  /**
   * List all credential keys (not values)
   */
  async listCredentials() {
    if (!fs.existsSync(this.credentialPath)) {
      return [];
    }

    const content = fs.readFileSync(this.credentialPath, 'utf8');
    const credentials = JSON.parse(content);

    return Object.keys(credentials).map(key => ({
      key,
      metadata: credentials[key].metadata
    }));
  }

  /**
   * Get inventory with rotation status
   */
  async getInventory() {
    if (!fs.existsSync(this.inventoryPath)) {
      return { credentials: {} };
    }

    return JSON.parse(fs.readFileSync(this.inventoryPath, 'utf8'));
  }

  /**
   * Delete a credential
   */
  async deleteCredential(key) {
    if (!fs.existsSync(this.credentialPath)) {
      return false;
    }

    const content = fs.readFileSync(this.credentialPath, 'utf8');
    const credentials = JSON.parse(content);

    if (!credentials[key]) {
      return false;
    }

    delete credentials[key];
    fs.writeFileSync(this.credentialPath, JSON.stringify(credentials, null, 2), {
      mode: 0o600
    });

    console.log(`🗑️  Deleted credential '${key}'`);
    return true;
  }

  /**
   * Check for credentials needing rotation
   */
  async checkRotationNeeded() {
    const inventory = await this.getInventory();
    const needsRotation = [];

    for (const [key, info] of Object.entries(inventory.credentials)) {
      if (info.rotationRequired) {
        const daysSinceRotation = info.lastRotation
          ? (Date.now() - new Date(info.lastRotation).getTime()) / (1000 * 60 * 60 * 24)
          : Infinity;

        if (daysSinceRotation > 90) { // 90 days
          needsRotation.push({
            key,
            daysSinceRotation: Math.floor(daysSinceRotation),
            ...info
          });
        }
      }

      // Check expiration
      if (info.expiresAt && new Date(info.expiresAt) < new Date()) {
        needsRotation.push({
          key,
          reason: 'expired',
          expiresAt: info.expiresAt,
          ...info
        });
      }
    }

    return needsRotation;
  }
}

// Export for use
module.exports = CredentialManager;

// CLI usage
if (require.main === module) {
  const manager = new CredentialManager();
  const command = process.argv[2];

  (async () => {
    try {
      switch (command) {
        case 'store':
          const key = process.argv[3];
          const value = process.argv[4];
          const type = process.argv[5] || 'generic';
          await manager.storeCredential(key, value, { type });
          break;

        case 'retrieve':
          const retrieveKey = process.argv[3];
          const retrieved = await manager.retrieveCredential(retrieveKey);
          console.log(retrieved);
          break;

        case 'list':
          const list = await manager.listCredentials();
          console.log(JSON.stringify(list, null, 2));
          break;

        case 'inventory':
          const inventory = await manager.getInventory();
          console.log(JSON.stringify(inventory, null, 2));
          break;

        case 'check-rotation':
          const needsRotation = await manager.checkRotationNeeded();
          if (needsRotation.length > 0) {
            console.log('⚠️  Credentials needing rotation:');
            console.log(JSON.stringify(needsRotation, null, 2));
          } else {
            console.log('✅ All credentials are current');
          }
          break;

        default:
          console.log('Credential Manager');
          console.log('==================\n');
          console.log('Usage:');
          console.log('  node credential-manager.js store <key> <value> [type]');
          console.log('  node credential-manager.js retrieve <key>');
          console.log('  node credential-manager.js list');
          console.log('  node credential-manager.js inventory');
          console.log('  node credential-manager.js check-rotation');
      }
    } catch (error) {
      console.error('❌ Error:', error.message);
      process.exit(1);
    }
  })();
}
