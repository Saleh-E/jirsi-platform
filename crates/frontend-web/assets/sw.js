// Service Worker for Jirsi Platform PWA
// Caches app shell, handles Background Sync for offline-first data

const CACHE_NAME = 'jirsi-v2';
const SYNC_TAG = 'jirsi-sync';
const API_BASE = '/api/v1';

const ASSETS_TO_CACHE = [
    '/',
    '/index.html',
    '/styles.css',
];

// Install event - cache core assets
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            console.log('[SW] Caching app shell');
            return cache.addAll(ASSETS_TO_CACHE);
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

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', (event) => {
    // Skip non-GET requests
    if (event.request.method !== 'GET') return;

    // Skip API requests - always fetch from network
    if (event.request.url.includes('/api/')) return;

    // For navigation requests, try network first, fall back to cached index.html
    if (event.request.mode === 'navigate') {
        event.respondWith(
            fetch(event.request).catch(() => caches.match('/index.html'))
        );
        return;
    }

    // For other assets, try cache first, then network
    event.respondWith(
        caches.match(event.request).then((cachedResponse) => {
            if (cachedResponse) {
                return cachedResponse;
            }
            return fetch(event.request).then((response) => {
                // Don't cache non-successful responses
                if (!response || response.status !== 200 || response.type !== 'basic') {
                    return response;
                }
                // Cache WASM and JS files for offline use
                const url = event.request.url;
                if (url.endsWith('.wasm') || url.endsWith('.js') || url.endsWith('.css')) {
                    const responseToCache = response.clone();
                    caches.open(CACHE_NAME).then((cache) => {
                        cache.put(event.request, responseToCache);
                    });
                }
                return response;
            });
        })
    );
});

// ============ Background Sync ============

// Listen for sync events
self.addEventListener('sync', (event) => {
    if (event.tag === SYNC_TAG) {
        console.log('[SW] Background sync triggered');
        event.waitUntil(syncPendingChanges());
    }
});

// Listen for messages from the app to queue sync
self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'QUEUE_SYNC') {
        console.log('[SW] Queueing sync request');
        queueSyncRequest(event.data.payload);
    }

    if (event.data && event.data.type === 'TRIGGER_SYNC') {
        console.log('[SW] Manual sync triggered');
        syncPendingChanges().then(() => {
            event.ports[0]?.postMessage({ success: true });
        }).catch((error) => {
            event.ports[0]?.postMessage({ success: false, error: error.message });
        });
    }
});

// Store pending changes in IndexedDB
async function queueSyncRequest(payload) {
    const db = await openSyncDB();
    const tx = db.transaction('pending', 'readwrite');
    const store = tx.objectStore('pending');

    await store.add({
        id: crypto.randomUUID(),
        timestamp: Date.now(),
        payload: payload,
        attempts: 0,
    });

    // Register for background sync if supported
    if ('sync' in self.registration) {
        try {
            await self.registration.sync.register(SYNC_TAG);
            console.log('[SW] Background sync registered');
        } catch (error) {
            console.warn('[SW] Background sync registration failed:', error);
        }
    }
}

// Sync all pending changes
async function syncPendingChanges() {
    const db = await openSyncDB();
    const tx = db.transaction('pending', 'readonly');
    const store = tx.objectStore('pending');
    const pendingItems = await store.getAll();

    console.log(`[SW] Processing ${pendingItems.length} pending items`);

    for (const item of pendingItems) {
        try {
            await syncItem(item);
            await removeSyncItem(item.id);
            console.log(`[SW] Synced item ${item.id}`);
        } catch (error) {
            console.error(`[SW] Failed to sync item ${item.id}:`, error);
            await incrementRetryCount(item.id);
        }
    }

    // Notify all clients that sync is complete
    const clients = await self.clients.matchAll();
    clients.forEach(client => {
        client.postMessage({
            type: 'SYNC_COMPLETE',
            timestamp: Date.now(),
        });
    });
}

// Sync a single item to the server
async function syncItem(item) {
    const { entityType, entityId, method, data, tenantId } = item.payload;

    let url = `${API_BASE}/entities/${entityType}`;
    if (entityId && method !== 'POST') {
        url += `/${entityId}`;
    }

    const response = await fetch(url, {
        method: method || 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Tenant-Id': tenantId || '',
            'X-Request-Id': crypto.randomUUID(),
        },
        body: JSON.stringify(data),
    });

    if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${await response.text()}`);
    }

    return response.json();
}

// Remove synced item from queue
async function removeSyncItem(id) {
    const db = await openSyncDB();
    const tx = db.transaction('pending', 'readwrite');
    const store = tx.objectStore('pending');
    await store.delete(id);
}

// Increment retry count for failed item
async function incrementRetryCount(id) {
    const db = await openSyncDB();
    const tx = db.transaction('pending', 'readwrite');
    const store = tx.objectStore('pending');
    const item = await store.get(id);

    if (item) {
        item.attempts++;
        item.lastAttempt = Date.now();

        // Max 5 retries
        if (item.attempts >= 5) {
            console.warn(`[SW] Removing item ${id} after 5 failed attempts`);
            await store.delete(id);
        } else {
            await store.put(item);
        }
    }
}

// Open/create IndexedDB for sync queue
function openSyncDB() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open('jirsi-sync-queue', 1);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);

        request.onupgradeneeded = (event) => {
            const db = event.target.result;

            if (!db.objectStoreNames.contains('pending')) {
                const store = db.createObjectStore('pending', { keyPath: 'id' });
                store.createIndex('timestamp', 'timestamp', { unique: false });
            }
        };
    });
}

// ============ Online/Offline Detection ============

// When coming back online, trigger sync
self.addEventListener('online', () => {
    console.log('[SW] Network restored - triggering sync');
    syncPendingChanges();
});

