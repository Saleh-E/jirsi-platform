// SQLite Web Worker - Offline-First Local Database
// Uses SQL.js with OPFS (Origin Private File System) for persistence

importScripts('https://cdn.jsdelivr.net/npm/sql.js@1.8.0/dist/sql-wasm.js');

let db = null;
let SQL = null;

// Initialize SQL.js
initSqlJs({
    locateFile: file => `https://cdn.jsdelivr.net/npm/sql.js@1.8.0/dist/${file}`
}).then(sqlInstance => {
    SQL = sqlInstance;
    loadDatabase();
});

// Load database from OPFS
async function loadDatabase() {
    try {
        const opfsRoot = await navigator.storage.getDirectory();
        const fileHandle = await opfsRoot.getFileHandle('jirsi.db', { create: true });
        const file = await fileHandle.getFile();

        if (file.size > 0) {
            const buffer = await file.arrayBuffer();
            db = new SQL.Database(new Uint8Array(buffer));
            console.log('Database loaded from OPFS');
        } else {
            db = new SQL.Database();
            await initializeSchema();
            console.log('New database initialized');
        }

        postMessage({ type: 'ready' });
    } catch (error) {
        console.error('Failed to load database:', error);
        db = new SQL.Database();
        await initializeSchema();
        postMessage({ type: 'ready' });
    }
}

// Save database to OPFS
async function saveDatabase() {
    try {
        const opfsRoot = await navigator.storage.getDirectory();
        const fileHandle = await opfsRoot.getFileHandle('jirsi.db', { create: true });
        const writable = await fileHandle.createWritable();

        const data = db.export();
        await writable.write(data);
        await writable.close();

        return { success: true };
    } catch (error) {
        console.error('Failed to save database:', error);
        return { success: false, error: error.message };
    }
}

// Initialize database schema
async function initializeSchema() {
    const schema = `
        CREATE TABLE IF NOT EXISTS entities (
            id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            field_values TEXT NOT NULL,
            version INTEGER DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            synced INTEGER DEFAULT 0
        );
        
        CREATE TABLE IF NOT EXISTS pending_mutations (
            id TEXT PRIMARY KEY,
            mutation_type TEXT NOT NULL,
            mutation_data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            retry_count INTEGER DEFAULT 0,
            last_error TEXT
        );
        
        CREATE TABLE IF NOT EXISTS sync_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
        CREATE INDEX IF NOT EXISTS idx_entities_synced ON entities(synced);
        CREATE INDEX IF NOT EXISTS idx_pending_mutations_created ON pending_mutations(created_at);
    `;

    db.run(schema);
    await saveDatabase();
}

// Handle messages from main thread
self.onmessage = async function (e) {
    const { type, payload, id } = e.data;

    try {
        let result;

        switch (type) {
            case 'query':
                result = executeQuery(payload.sql, payload.params);
                break;

            case 'exec':
                result = executeExec(payload.sql, payload.params);
                await saveDatabase();
                break;

            case 'create_entity':
                result = createEntity(payload);
                await saveDatabase();
                break;

            case 'update_entity':
                result = updateEntity(payload);
                await saveDatabase();
                break;

            case 'delete_entity':
                result = deleteEntity(payload);
                await saveDatabase();
                break;

            case 'get_pending_mutations':
                result = getPendingMutations();
                break;

            case 'add_pending_mutation':
                result = addPendingMutation(payload);
                await saveDatabase();
                break;

            case 'clear_pending_mutation':
                result = clearPendingMutation(payload.id);
                await saveDatabase();
                break;

            case 'get_sync_state':
                result = getSyncState();
                break;

            case 'update_sync_state':
                result = updateSyncState(payload);
                await saveDatabase();
                break;

            default:
                throw new Error(`Unknown message type: ${type}`);
        }

        postMessage({ id, type: 'success', result });

    } catch (error) {
        postMessage({ id, type: 'error', error: error.message });
    }
};

// Execute SELECT query
function executeQuery(sql, params = []) {
    const stmt = db.prepare(sql);
    stmt.bind(params);

    const results = [];
    while (stmt.step()) {
        results.push(stmt.getAsObject());
    }
    stmt.free();

    return results;
}

// Execute INSERT/UPDATE/DELETE
function executeExec(sql, params = []) {
    db.run(sql, params);
    return { changes: db.getRowsModified() };
}

// Create entity
function createEntity({ id, entity_type, field_values }) {
    const now = new Date().toISOString();

    db.run(
        `INSERT INTO entities (id, entity_type, field_values, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)`,
        [id, entity_type, JSON.stringify(field_values), now, now]
    );

    return { id };
}

// Update entity
function updateEntity({ id, field_values, version }) {
    const now = new Date().toISOString();

    db.run(
        `UPDATE entities 
         SET field_values = ?, version = version + 1, updated_at = ?, synced = 0
         WHERE id = ? AND version = ?`,
        [JSON.stringify(field_values), now, id, version]
    );

    const changes = db.getRowsModified();
    if (changes === 0) {
        throw new Error('Optimistic lock failure');
    }

    return { success: true };
}

// Delete entity
function deleteEntity({ id, version }) {
    const now = new Date().toISOString();

    db.run(
        `UPDATE entities 
         SET deleted_at = ?, synced = 0
         WHERE id = ? AND version = ?`,
        [now, id, version]
    );

    return { success: true };
}

// Get pending mutations
function getPendingMutations() {
    const stmt = db.prepare(`
        SELECT * FROM pending_mutations 
        ORDER BY created_at ASC
    `);

    const mutations = [];
    while (stmt.step()) {
        const row = stmt.getAsObject();
        mutations.push({
            ...row,
            mutation_data: JSON.parse(row.mutation_data)
        });
    }
    stmt.free();

    return mutations;
}

// Add pending mutation
function addPendingMutation(mutation) {
    const now = new Date().toISOString();

    db.run(
        `INSERT INTO pending_mutations (id, mutation_type, mutation_data, created_at)
         VALUES (?, ?, ?, ?)`,
        [mutation.id, mutation.type, JSON.stringify(mutation.data), now]
    );

    return { success: true };
}

// Clear pending mutation
function clearPendingMutation(id) {
    db.run(`DELETE FROM pending_mutations WHERE id = ?`, [id]);
    return { success: true };
}

// Get sync state
function getSyncState() {
    const stmt = db.prepare(`SELECT key, value FROM sync_state`);

    const state = {};
    while (stmt.step()) {
        const row = stmt.getAsObject();
        state[row.key] = JSON.parse(row.value);
    }
    stmt.free();

    return state;
}

// Update sync state
function updateSyncState({ key, value }) {
    const now = new Date().toISOString();

    db.run(
        `INSERT OR REPLACE INTO sync_state (key, value, updated_at)
         VALUES (?, ?, ?)`,
        [key, JSON.stringify(value), now]
    );

    return { success: true };
}
