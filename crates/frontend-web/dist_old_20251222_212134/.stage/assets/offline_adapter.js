
// This file assumes sqlite3.js is loaded globally or ES modules are used.
// We'll use the global object if loaded via script tag, or dynamic import.

export async function initOfflineDb() {
    console.log("Initializing Offline DB...");
    try {
        // Load sqlite3
        // sqlite3.js is loaded via <script> tag in index.html
        // So sqlite3InitModule is available globally

        if (typeof sqlite3InitModule === 'undefined') {
            console.error("sqlite3InitModule not found. Ensure sqlite3.js is loaded.");
            return false;
        }

        const sqlite3 = await sqlite3InitModule({
            print: console.log,
            printErr: console.error,
        });

        console.log('Running SQLite3 version', sqlite3.version.libVersion);

        if ('opfs' in sqlite3) {
            const db = new sqlite3.oo1.OpfsDb('/jirsi_offline.sqlite3');
            console.log('OPFS Database opened: /jirsi_offline.sqlite3');

            // Create sync table if not exists
            db.exec(`
                CREATE TABLE IF NOT EXISTS local_entity_records (
                    id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    entity_type TEXT NOT NULL,
                    data JSON NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    is_dirty BOOLEAN DEFAULT 0,
                    is_deleted BOOLEAN DEFAULT 0,
                    last_synced_at TEXT
                );
            `);

            // Store db instance on window for easier access (or return handle)
            // But we can't pass the DB object directly to Rust easily if it's not a simple JsValue.
            // We'll wrap operations in this adapter.
            window._jirsi_offline_db = db;
            return true;
        } else {
            console.error('OPFS is not available.');
            return false;
        }
    } catch (err) {
        console.error('Failed to initialize offline DB:', err);
        return false;
    }
}

export function executeSql(sql, params) {
    if (!window._jirsi_offline_db) {
        throw new Error("DB not initialized");
    }
    const db = window._jirsi_offline_db;
    // sqlite3.oo1.OpfsDb.exec returns void or result
    // We want to return rows if it's a SELECT

    // Using exec with callback for results
    const results = [];
    db.exec({
        sql: sql,
        bind: params,
        rowMode: 'object',
        callback: function (row) {
            results.push(row);
        }
    });
    return results;
}
