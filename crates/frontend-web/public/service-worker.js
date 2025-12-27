// Service Worker - Enhanced Background Sync
// Handles offline sync with is_dirty tracking and exponential backoff
// Phase 5: Mobile PWA Optimization

const CACHE_NAME = 'jirsi-v2';
const SYNC_TAG = 'jirsi-sync';
const DIRTY_SYNC_TAG = 'jirsi-dirty-sync';
const MAX_RETRY_COUNT = 5;
const MAX_RETRY_DELAY_MS = 30000;

// Static assets to cache
const STATIC_ASSETS = [
    '/',
    '/index.html',
    '/pkg/frontend_web.js',
    '/pkg/frontend_web_bg.wasm',
    '/styles.css',
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(STATIC_ASSETS).catch(err => {
                console.warn('[SW] Failed to cache some assets:', err);
            });
        })
    );
    self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames
                    .filter((name) => name.startsWith('jirsi-') && name !== CACHE_NAME)
                    .map((name) => caches.delete(name))
            );
        })
    );
    self.clients.claim();
});

// Fetch event - network first for API, cache first for static
self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);

    // API requests - network only, queue for background sync if offline
    if (url.pathname.startsWith('/api/')) {
        event.respondWith(handleApiRequest(request));
        return;
    }

    // Static assets - cache first, network fallback
    event.respondWith(handleStaticRequest(request));
});

// Handle API requests with offline queueing
async function handleApiRequest(request) {
    try {
        const response = await fetch(request);
        return response;
    } catch (error) {
        // Network failed - queue for sync if it's a mutation
        if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(request.method)) {
            await queueForSync(request);
            return new Response(
                JSON.stringify({
                    queued: true,
                    message: 'Request queued for sync when online',
                    timestamp: new Date().toISOString()
                }),
                {
                    status: 202,
                    headers: { 'Content-Type': 'application/json' }
                }
            );
        }
        // GET requests - try cache as last resort
        const cached = await caches.match(request);
        if (cached) return cached;

        return new Response(
            JSON.stringify({ error: 'Offline', message: 'No cached data available' }),
            { status: 503, headers: { 'Content-Type': 'application/json' } }
        );
    }
}

// Handle static requests with caching
async function handleStaticRequest(request) {
    const cached = await caches.match(request);
    if (cached) return cached;

    try {
        const response = await fetch(request);
        if (response.ok) {
            const cache = await caches.open(CACHE_NAME);
            cache.put(request, response.clone());
        }
        return response;
    } catch (error) {
        return new Response('Offline', { status: 503 });
    }
}

// Background sync event
self.addEventListener('sync', (event) => {
    if (event.tag === SYNC_TAG || event.tag === DIRTY_SYNC_TAG) {
        event.waitUntil(performSync());
    }
});

// Queue request for background sync with is_dirty tracking
async function queueForSync(request) {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readwrite');
    const store = tx.objectStore('sync_queue');

    // Extract entity info from URL for is_dirty tracking
    const url = new URL(request.url);
    const pathParts = url.pathname.split('/').filter(Boolean);
    const entityInfo = extractEntityInfo(pathParts);

    const requestData = {
        id: Date.now() + Math.random().toString(36).substr(2, 9),
        url: request.url,
        method: request.method,
        headers: Object.fromEntries(request.headers.entries()),
        body: await request.clone().text(),
        timestamp: new Date().toISOString(),
        retryCount: 0,
        nextRetryAt: Date.now(),
        // is_dirty tracking
        is_dirty: true,
        entity_type: entityInfo.type,
        entity_id: entityInfo.id,
    };

    await store.add(requestData);
    await tx.complete;
    db.close();

    // Broadcast sync status
    broadcastSyncStatus('pending', {
        count: await getPendingCount(),
        entity: entityInfo
    });

    // Register background sync
    if ('sync' in self.registration) {
        await self.registration.sync.register(SYNC_TAG);
    }
}

// Extract entity type and ID from URL path
function extractEntityInfo(pathParts) {
    // Path patterns: /api/v1/entities/{type}/{id}
    // or /api/v1/{type}/{id}
    let type = null;
    let id = null;

    const entityIndex = pathParts.indexOf('entities');
    if (entityIndex !== -1 && pathParts.length > entityIndex + 1) {
        type = pathParts[entityIndex + 1];
        if (pathParts.length > entityIndex + 2) {
            id = pathParts[entityIndex + 2];
        }
    } else if (pathParts.length >= 3) {
        // /api/v1/contacts/uuid
        type = pathParts[pathParts.length - 2];
        id = pathParts[pathParts.length - 1];
    }

    return { type, id };
}

// Calculate exponential backoff delay
function getRetryDelay(retryCount) {
    const baseDelay = 1000; // 1 second
    const delay = Math.min(baseDelay * Math.pow(2, retryCount), MAX_RETRY_DELAY_MS);
    // Add jitter (±20%)
    return delay * (0.8 + Math.random() * 0.4);
}

// Perform background sync with exponential backoff
async function performSync() {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readonly');
    const store = tx.objectStore('sync_queue');
    const allRequests = await promisifyRequest(store.getAll());
    db.close();

    if (allRequests.length === 0) {
        broadcastSyncStatus('idle', { count: 0 });
        return;
    }

    broadcastSyncStatus('syncing', { count: allRequests.length });

    // Filter requests that are ready for retry
    const now = Date.now();
    const readyRequests = allRequests.filter(req => req.nextRetryAt <= now);

    let successCount = 0;
    let failCount = 0;

    for (const reqData of readyRequests) {
        try {
            const response = await fetch(reqData.url, {
                method: reqData.method,
                headers: reqData.headers,
                body: reqData.body || undefined,
            });

            if (response.ok) {
                // Success - remove from queue
                await removeFromQueue(reqData.id);
                successCount++;

                notifyClients({
                    type: 'SYNC_SUCCESS',
                    requestId: reqData.id,
                    entity_type: reqData.entity_type,
                    entity_id: reqData.entity_id,
                });
            } else if (response.status >= 400 && response.status < 500) {
                // Client error - don't retry, remove from queue
                await removeFromQueue(reqData.id);
                failCount++;

                notifyClients({
                    type: 'SYNC_FAILED',
                    requestId: reqData.id,
                    error: `Server returned ${response.status}`,
                    permanent: true,
                });
            } else {
                // Server error - schedule retry
                await scheduleRetry(reqData);
            }
        } catch (error) {
            console.error('[SW] Sync failed for request:', reqData.id, error);
            await scheduleRetry(reqData);
            failCount++;
        }
    }

    const remaining = await getPendingCount();
    broadcastSyncStatus(remaining > 0 ? 'pending' : 'complete', {
        count: remaining,
        synced: successCount,
        failed: failCount
    });
}

// Schedule a retry with exponential backoff
async function scheduleRetry(reqData) {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readwrite');
    const store = tx.objectStore('sync_queue');

    reqData.retryCount = (reqData.retryCount || 0) + 1;

    if (reqData.retryCount >= MAX_RETRY_COUNT) {
        // Give up after max retries
        await store.delete(reqData.id);

        notifyClients({
            type: 'SYNC_FAILED',
            requestId: reqData.id,
            error: 'Max retries exceeded',
            permanent: true,
            entity_type: reqData.entity_type,
            entity_id: reqData.entity_id,
        });
    } else {
        // Schedule next retry with backoff
        reqData.nextRetryAt = Date.now() + getRetryDelay(reqData.retryCount);
        reqData.is_dirty = true;
        await store.put(reqData);

        // Schedule sync for when the retry is due
        setTimeout(() => {
            if ('sync' in self.registration) {
                self.registration.sync.register(SYNC_TAG);
            }
        }, reqData.nextRetryAt - Date.now());
    }

    await tx.complete;
    db.close();
}

// Remove completed request from queue
async function removeFromQueue(id) {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readwrite');
    await tx.objectStore('sync_queue').delete(id);
    await tx.complete;
    db.close();
}

// Get count of pending (is_dirty) requests
async function getPendingCount() {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readonly');
    const count = await promisifyRequest(tx.objectStore('sync_queue').count());
    db.close();
    return count;
}

// Open IndexedDB for sync queue (v2 with entity indexes)
function openSyncDB() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open('jirsi_sync', 2);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);

        request.onupgradeneeded = (event) => {
            const db = event.target.result;

            if (!db.objectStoreNames.contains('sync_queue')) {
                const store = db.createObjectStore('sync_queue', { keyPath: 'id' });
                store.createIndex('is_dirty', 'is_dirty', { unique: false });
                store.createIndex('entity_type', 'entity_type', { unique: false });
                store.createIndex('nextRetryAt', 'nextRetryAt', { unique: false });
            } else if (event.oldVersion < 2) {
                // Upgrade from v1 to v2: add indexes
                const store = event.target.transaction.objectStore('sync_queue');
                if (!store.indexNames.contains('is_dirty')) {
                    store.createIndex('is_dirty', 'is_dirty', { unique: false });
                }
                if (!store.indexNames.contains('entity_type')) {
                    store.createIndex('entity_type', 'entity_type', { unique: false });
                }
                if (!store.indexNames.contains('nextRetryAt')) {
                    store.createIndex('nextRetryAt', 'nextRetryAt', { unique: false });
                }
            }
        };
    });
}

// Promisify IDBRequest
function promisifyRequest(request) {
    return new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}

// Broadcast sync status to all clients
async function broadcastSyncStatus(status, details = {}) {
    const clients = await self.clients.matchAll({ type: 'window' });
    const message = {
        type: 'SYNC_STATUS',
        status, // 'idle' | 'pending' | 'syncing' | 'complete' | 'error'
        ...details,
        timestamp: new Date().toISOString(),
    };

    clients.forEach((client) => {
        client.postMessage(message);
    });
}

// Notify specific clients
async function notifyClients(message) {
    const clients = await self.clients.matchAll({ type: 'window' });
    message.timestamp = new Date().toISOString();
    clients.forEach((client) => {
        client.postMessage(message);
    });
}

// Periodic background sync (if supported)
self.addEventListener('periodicsync', (event) => {
    if (event.tag === 'sync-data' || event.tag === 'jirsi-periodic') {
        event.waitUntil(performSync());
    }
});

// Online status change handler via message
self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'ONLINE') {
        // Trigger sync when coming back online
        performSync();
    }
    if (event.data && event.data.type === 'GET_PENDING_COUNT') {
        getPendingCount().then(count => {
            event.source.postMessage({
                type: 'PENDING_COUNT',
                count
            });
        });
    }
});
