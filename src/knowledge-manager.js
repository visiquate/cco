#!/usr/bin/env node

/**
 * Knowledge Manager - LanceDB Integration
 *
 * Provides persistent knowledge storage and retrieval across compactions
 * using vector embeddings for semantic search.
 */

const lancedb = require('vectordb');
const crypto = require('crypto');
const fs = require('fs').promises;
const path = require('path');

class KnowledgeManager {
  constructor(options = {}) {
    // Support per-repository databases
    this.repoPath = options.repoPath || process.cwd();
    this.repoName = options.repoName || this.getRepoName(this.repoPath);

    // Each repo gets its own database
    const baseDir = options.baseDir || path.join(__dirname, '../data/knowledge');
    this.dbPath = path.join(baseDir, this.repoName);
    this.tableName = options.tableName || 'orchestra_knowledge';

    this.db = null;
    this.table = null;
    this.embeddingDim = 384; // Default for small embedding models

    console.log(`📦 Knowledge Manager initialized for repository: ${this.repoName}`);
    console.log(`📁 Database path: ${this.dbPath}`);
  }

  /**
   * Extract repository name from path
   */
  getRepoName(repoPath) {
    const parts = repoPath.split(path.sep);
    return parts[parts.length - 1] || 'default';
  }

  /**
   * Initialize the LanceDB connection and table
   */
  async initialize() {
    try {
      // Ensure data directory exists (recursive for repo-specific subdirs)
      await fs.mkdir(this.dbPath, { recursive: true });

      // Connect to LanceDB (repo-specific database)
      this.db = await lancedb.connect(this.dbPath);

      // Try to open existing table, or create new one
      try {
        this.table = await this.db.openTable(this.tableName);
        console.log(`✅ Connected to existing knowledge base for ${this.repoName}`);
      } catch (error) {
        // Table doesn't exist, create it
        console.log(`📝 Creating new knowledge base for ${this.repoName}`);
        await this.createTable();
      }

      return true;
    } catch (error) {
      console.error(`❌ Failed to initialize Knowledge Manager for ${this.repoName}:`, error.message);
      throw error;
    }
  }

  /**
   * Create the knowledge table with schema
   */
  async createTable() {
    const schema = [
      {
        id: 'init-' + Date.now(),
        vector: Array(this.embeddingDim).fill(0),
        text: 'Initialization record',
        type: 'system',
        project_id: 'system',
        session_id: 'init',
        agent: 'system',
        timestamp: new Date().toISOString(),
        metadata: JSON.stringify({})
      }
    ];

    this.table = await this.db.createTable(this.tableName, schema);
    console.log('✅ Knowledge table created successfully');
  }

  /**
   * Generate a simple embedding from text
   * In production, use a real embedding model like sentence-transformers
   */
  generateEmbedding(text) {
    // Simple hash-based embedding for demonstration
    // In production, replace with actual embedding model
    const hash = crypto.createHash('sha256').update(text).digest();
    const embedding = [];

    for (let i = 0; i < this.embeddingDim; i++) {
      // Normalize to [-1, 1] range
      embedding.push((hash[i % hash.length] / 128.0) - 1.0);
    }

    return embedding;
  }

  /**
   * Store knowledge with automatic embedding generation
   */
  async store(knowledge) {
    try {
      const {
        text,
        type = 'decision',
        project_id = this.repoName, // Default to current repo
        session_id = 'unknown',
        agent = 'unknown',
        metadata = {}
      } = knowledge;

      if (!text || typeof text !== 'string') {
        throw new Error('Knowledge text is required');
      }

      // Generate embedding
      const vector = this.generateEmbedding(text);

      // Create record
      const record = {
        id: `${type}-${Date.now()}-${Math.random().toString(36).substring(7)}`,
        vector,
        text,
        type,
        project_id,
        session_id,
        agent,
        timestamp: new Date().toISOString(),
        metadata: JSON.stringify(metadata)
      };

      // Add to table
      await this.table.add([record]);

      console.log(`✅ Stored knowledge: ${type} from ${agent}`);
      return record.id;
    } catch (error) {
      console.error('❌ Failed to store knowledge:', error.message);
      throw error;
    }
  }

  /**
   * Store multiple knowledge items in batch
   */
  async storeBatch(knowledgeItems) {
    const ids = [];

    for (const item of knowledgeItems) {
      try {
        const id = await this.store(item);
        ids.push(id);
      } catch (error) {
        console.error(`⚠️  Failed to store item: ${error.message}`);
      }
    }

    console.log(`✅ Stored ${ids.length}/${knowledgeItems.length} knowledge items`);
    return ids;
  }

  /**
   * Search knowledge base using semantic similarity
   */
  async search(query, options = {}) {
    try {
      const {
        limit = 10,
        threshold = 0.5,
        project_id = null,
        type = null,
        agent = null
      } = options;

      // Generate query embedding
      const queryVector = this.generateEmbedding(query);

      // Perform vector search
      let results = await this.table
        .search(queryVector)
        .limit(limit)
        .execute();

      // Filter by metadata if specified
      if (project_id || type || agent) {
        results = results.filter(result => {
          if (project_id && result.project_id !== project_id) return false;
          if (type && result.type !== type) return false;
          if (agent && result.agent !== agent) return false;
          return true;
        });
      }

      // Parse metadata and format results
      const formatted = results.map(result => ({
        id: result.id,
        text: result.text,
        type: result.type,
        project_id: result.project_id,
        session_id: result.session_id,
        agent: result.agent,
        timestamp: result.timestamp,
        metadata: JSON.parse(result.metadata || '{}'),
        score: result._distance // Similarity score
      }));

      console.log(`🔍 Found ${formatted.length} relevant knowledge items`);
      return formatted;
    } catch (error) {
      console.error('❌ Search failed:', error.message);
      throw error;
    }
  }

  /**
   * Retrieve all knowledge for a specific project
   */
  async getProjectKnowledge(project_id, options = {}) {
    try {
      const { type = null, limit = 100 } = options;

      // Use search with a generic query to get all records
      // Since we can't use toArray(), search with empty vector or retrieve via scan
      const dummyVector = Array(this.embeddingDim).fill(0);

      // Get many records (up to 1000)
      const allRecords = await this.table
        .search(dummyVector)
        .limit(1000)
        .execute();

      let filtered = allRecords.filter(r => r.project_id === project_id);

      if (type) {
        filtered = filtered.filter(r => r.type === type);
      }

      // Sort by timestamp (newest first)
      filtered.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

      // Limit results
      const results = filtered.slice(0, limit);

      // Format results
      const formatted = results.map(result => ({
        id: result.id,
        text: result.text,
        type: result.type,
        project_id: result.project_id,
        session_id: result.session_id,
        agent: result.agent,
        timestamp: result.timestamp,
        metadata: JSON.parse(result.metadata || '{}')
      }));

      console.log(`📚 Retrieved ${formatted.length} knowledge items for project: ${project_id}`);
      return formatted;
    } catch (error) {
      console.error('❌ Failed to retrieve project knowledge:', error.message);
      throw error;
    }
  }

  /**
   * Pre-compaction hook: Extract and store critical knowledge
   */
  async preCompaction(conversation, context = {}) {
    console.log('🔄 Running pre-compaction knowledge capture...');

    try {
      const knowledge = this.extractCriticalKnowledge(conversation, context);
      const ids = await this.storeBatch(knowledge);

      console.log(`✅ Pre-compaction complete: Captured ${ids.length} knowledge items`);
      return { success: true, count: ids.length, ids };
    } catch (error) {
      console.error('❌ Pre-compaction failed:', error.message);
      return { success: false, error: error.message };
    }
  }

  /**
   * Post-compaction hook: Retrieve relevant context
   */
  async postCompaction(currentTask, context = {}) {
    console.log('🔄 Running post-compaction knowledge retrieval...');

    try {
      const { project_id = 'default', limit = 10 } = context;

      // Search for relevant knowledge based on current task
      const results = await this.search(currentTask, {
        limit,
        project_id
      });

      // Also get recent project knowledge
      const recentKnowledge = await this.getProjectKnowledge(project_id, {
        limit: 5
      });

      const combined = {
        searchResults: results,
        recentKnowledge,
        summary: this.generateContextSummary(results, recentKnowledge)
      };

      console.log(`✅ Post-compaction complete: Retrieved ${results.length} relevant items`);
      return combined;
    } catch (error) {
      console.error('❌ Post-compaction retrieval failed:', error.message);
      return { error: error.message };
    }
  }

  /**
   * Extract critical knowledge from conversation
   */
  extractCriticalKnowledge(conversation, context = {}) {
    const { project_id = 'default', session_id = 'unknown' } = context;
    const knowledge = [];

    // This is a simplified extraction - in production, use more sophisticated NLP
    const patterns = {
      architecture: /architecture|design pattern|system design/i,
      decision: /decided|chose|selected|will use/i,
      implementation: /implemented|built|created|added/i,
      configuration: /configured|setup|initialized/i,
      credential: /api key|secret|token|password|credential/i,
      issue: /bug|issue|problem|error|fix/i
    };

    // Split conversation into messages or paragraphs
    const messages = conversation.split('\n\n');

    messages.forEach((message, index) => {
      // Skip very short messages
      if (message.length < 50) return;

      // Detect knowledge type
      let type = 'general';
      for (const [pattern, regex] of Object.entries(patterns)) {
        if (regex.test(message)) {
          type = pattern;
          break;
        }
      }

      // Extract agent if mentioned
      const agentMatch = message.match(/\b(architect|python|swift|go|rust|flutter|qa|security|devops)\b/i);
      const agent = agentMatch ? agentMatch[1].toLowerCase() : 'unknown';

      knowledge.push({
        text: message.trim(),
        type,
        project_id,
        session_id,
        agent,
        metadata: {
          conversationIndex: index,
          extractedAt: new Date().toISOString()
        }
      });
    });

    console.log(`📊 Extracted ${knowledge.length} knowledge items from conversation`);
    return knowledge;
  }

  /**
   * Generate a summary of retrieved context
   */
  generateContextSummary(searchResults, recentKnowledge) {
    const summary = {
      totalItems: searchResults.length + recentKnowledge.length,
      byType: {},
      byAgent: {},
      topDecisions: [],
      recentActivity: []
    };

    // Combine all items
    const allItems = [...searchResults, ...recentKnowledge];

    // Count by type and agent
    allItems.forEach(item => {
      summary.byType[item.type] = (summary.byType[item.type] || 0) + 1;
      summary.byAgent[item.agent] = (summary.byAgent[item.agent] || 0) + 1;
    });

    // Extract top decisions
    summary.topDecisions = searchResults
      .filter(item => item.type === 'decision')
      .slice(0, 5)
      .map(item => item.text.substring(0, 100) + '...');

    // Recent activity
    summary.recentActivity = recentKnowledge
      .slice(0, 3)
      .map(item => ({
        type: item.type,
        agent: item.agent,
        timestamp: item.timestamp,
        preview: item.text.substring(0, 80) + '...'
      }));

    return summary;
  }

  /**
   * Clean up old knowledge (optional maintenance)
   */
  async cleanup(options = {}) {
    const {
      olderThanDays = 90,
      project_id = null
    } = options;

    console.log(`🧹 Cleaning up knowledge older than ${olderThanDays} days...`);

    try {
      const cutoffDate = new Date();
      cutoffDate.setDate(cutoffDate.getDate() - olderThanDays);

      // Use search to get records
      const dummyVector = Array(this.embeddingDim).fill(0);
      const allRecords = await this.table.search(dummyVector).limit(1000).execute();

      let toDelete = allRecords.filter(record => {
        const recordDate = new Date(record.timestamp);
        return recordDate < cutoffDate;
      });

      if (project_id) {
        toDelete = toDelete.filter(r => r.project_id === project_id);
      }

      // Note: LanceDB doesn't have built-in delete, would need to recreate table
      // This is a simplified version
      console.log(`⚠️  Found ${toDelete.length} old records (cleanup not yet implemented)`);

      return { count: toDelete.length };
    } catch (error) {
      console.error('❌ Cleanup failed:', error.message);
      throw error;
    }
  }

  /**
   * Get statistics about the knowledge base
   */
  async getStats() {
    try {
      // Use search to get records
      const dummyVector = Array(this.embeddingDim).fill(0);
      const allRecords = await this.table.search(dummyVector).limit(1000).execute();

      const stats = {
        repository: this.repoName,
        totalRecords: allRecords.length,
        byType: {},
        byAgent: {},
        byProject: {},
        oldestRecord: null,
        newestRecord: null
      };

      allRecords.forEach(record => {
        // Count by type
        stats.byType[record.type] = (stats.byType[record.type] || 0) + 1;

        // Count by agent
        stats.byAgent[record.agent] = (stats.byAgent[record.agent] || 0) + 1;

        // Count by project
        stats.byProject[record.project_id] = (stats.byProject[record.project_id] || 0) + 1;

        // Track oldest/newest
        const timestamp = new Date(record.timestamp);
        if (!stats.oldestRecord || timestamp < new Date(stats.oldestRecord)) {
          stats.oldestRecord = record.timestamp;
        }
        if (!stats.newestRecord || timestamp > new Date(stats.newestRecord)) {
          stats.newestRecord = record.timestamp;
        }
      });

      return stats;
    } catch (error) {
      console.error('❌ Failed to get stats:', error.message);
      throw error;
    }
  }

  /**
   * Close the database connection
   */
  async close() {
    // LanceDB doesn't require explicit closing
    console.log('✅ Knowledge Manager closed');
  }
}

// CLI interface
if (require.main === module) {
  const command = process.argv[2];
  const km = new KnowledgeManager();

  (async () => {
    await km.initialize();

    switch (command) {
      case 'store':
        {
          const text = process.argv[3];
          const type = process.argv[4] || 'general';
          if (!text) {
            console.error('Usage: node knowledge-manager.js store "<text>" [type]');
            process.exit(1);
          }
          const id = await km.store({ text, type, agent: 'cli' });
          console.log(`Stored with ID: ${id}`);
        }
        break;

      case 'search':
        {
          const query = process.argv[3];
          const limit = parseInt(process.argv[4]) || 10;
          if (!query) {
            console.error('Usage: node knowledge-manager.js search "<query>" [limit]');
            process.exit(1);
          }
          const results = await km.search(query, { limit });
          console.log(JSON.stringify(results, null, 2));
        }
        break;

      case 'stats':
        {
          const stats = await km.getStats();
          console.log(JSON.stringify(stats, null, 2));
        }
        break;

      case 'test':
        {
          console.log('Running Knowledge Manager test...');

          // Store some test knowledge
          console.log('\n1. Storing test knowledge...');
          await km.store({
            text: 'We decided to use FastAPI for the REST API because it has automatic OpenAPI documentation and excellent async support.',
            type: 'decision',
            agent: 'architect',
            project_id: 'test-project'
          });

          await km.store({
            text: 'Implemented JWT authentication with RS256 algorithm. Private key stored in /tmp/credentials.json.',
            type: 'implementation',
            agent: 'python',
            project_id: 'test-project'
          });

          await km.store({
            text: 'Security audit found no critical vulnerabilities. Recommended adding rate limiting to auth endpoints.',
            type: 'issue',
            agent: 'security',
            project_id: 'test-project'
          });

          // Search for relevant knowledge
          console.log('\n2. Searching for API-related knowledge...');
          const results = await km.search('API authentication', {
            limit: 5,
            project_id: 'test-project'
          });

          console.log(`Found ${results.length} results:`);
          results.forEach((r, i) => {
            console.log(`  ${i + 1}. [${r.type}] ${r.text.substring(0, 80)}...`);
          });

          // Get project knowledge
          console.log('\n3. Getting all project knowledge...');
          const projectKnowledge = await km.getProjectKnowledge('test-project');
          console.log(`Project has ${projectKnowledge.length} knowledge items`);

          // Get stats
          console.log('\n4. Knowledge base statistics:');
          const stats = await km.getStats();
          console.log(JSON.stringify(stats, null, 2));

          console.log('\n✅ Test complete!');
        }
        break;

      default:
        console.log(`
Knowledge Manager - LanceDB Integration

Usage:
  node knowledge-manager.js test                       - Run test suite
  node knowledge-manager.js store "<text>" [type]     - Store knowledge
  node knowledge-manager.js search "<query>" [limit]  - Search knowledge
  node knowledge-manager.js stats                     - Show statistics

Examples:
  node knowledge-manager.js test
  node knowledge-manager.js store "We chose Python" decision
  node knowledge-manager.js search "authentication" 5
  node knowledge-manager.js stats
        `);
    }

    await km.close();
  })().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
  });
}

module.exports = KnowledgeManager;
