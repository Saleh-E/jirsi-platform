// Service Worker - Background Sync
// Handles offline sync when connection is restored

const CACHE_NAME = 'jirsi-v1';
const SYNC_TAG = 'jirsi-sync';

// Install event - cache static assets
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll([
                '/',
                '/index.html',
                '/pkg/frontend_web.js',
                '/pkg/frontend_web_bg.wasm',
                '/css/main.css',
            ]);
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
                    .filter((name) => name !== CACHE_NAME)
                    .map((name) => caches.delete(name))
            );
        })
    );
    self.clients.claim();
});

// Fetch event - network first, fallback to cache
self.addEventListener('fetch', (event) => {
    const { request } = event;

    // API requests - network only, queue for background sync if offline
    if (request.url.includes('/api/')) {
        event.respondWith(
            fetch(request)
                .catch(() => {
                    // Queue for background sync
                    return queueForSync(request).then(() => {
                        return new Response(
                            JSON.stringify({ queued: true }),
                            { headers: { 'Content-Type': 'application/json' } }
                        );
                    });
                })
        );
        return;
    }

    // Static assets - cache first, fallback to network
    event.respondWith(
        caches.match(request).then((response) => {
            return response || fetch(request).then((fetchResponse) => {
                return caches.open(CACHE_NAME).then((cache) => {
                    cache.put(request, fetchResponse.clone());
                    return fetchResponse;
                });
            });
        })
    );
});

// Background sync event
self.addEventListener('sync', (event) => {
    if (event.tag === SYNC_TAG) {
        event.waitUntil(performSync());
    }
});

// Queue request for background sync
async function queueForSync(request) {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readwrite');
    const store = tx.objectStore('sync_queue');

    const requestData = {
        id: Date.now(),
        url: request.url,
        method: request.method,
        headers: Object.fromEntries(request.headers.entries()),
        body: await request.text(),
        timestamp: new Date().toISOString(),
    };

    await store.add(requestData);

    // Register background sync
    if ('sync' in self.registration) {
        await self.registration.sync.register(SYNC_TAG);
    }
}

// Perform background sync
async function performSync() {
    const db = await openSyncDB();
    const tx = db.transaction('sync_queue', 'readonly');
    const store = tx.objectStore('sync_queue');
    const requests = await store.getAll();

    for (const reqData of requests) {
        try {
            const response = await fetch(reqData.url, {
                method: reqData.method,
                headers: reqData.headers,
                body: reqData.body,
            });

            if (response.ok) {
                // Remove from queue
                const deleteTx = db.transaction('sync_queue', 'readwrite');
                await deleteTx.objectStore('sync_queue').delete(reqData.id);

                // Notify client
                notifyClients({
                    type: 'sync_success',
                    requestId: reqData.id,
                });
            }
        } catch (error) {
            console.error('Sync failed for request:', reqData.id, error);

            // Retry count logic
            reqData.retryCount = (reqData.retryCount || 0) + 1;
            if (reqData.retryCount >= 3) {
                // Give up after 3 retries
                const deleteTx = db.transaction('sync_queue', 'readwrite');
                await deleteTx.objectStore('sync_queue').delete(reqData.id);

                notifyClients({
                    type: 'sync_failed',
                    requestId: reqData.id,
                    error: error.message,
                });
            } else {
                // Update retry count
                const updateTx = db.transaction('sync_queue', 'readwrite');
                await updateTx.objectStore('sync_queue').put(reqData);
            }
        }
    }
}

// Open IndexedDB for sync queue
function openSyncDB() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open('jirsi_sync', 1);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);

        request.onupgradeneeded = (event) => {
            const db = event.target.result;
            if (!db.objectStoreNames.contains('sync_queue')) {
                db.createObjectStore('sync_queue', { keyPath: 'id' });
            }
        };
    });
}

// Notify all clients
async function notifyClients(message) {
    const clients = await self.clients.matchAll({ type: 'window' });
    clients.forEach((client) => {
        client.postMessage(message);
    });
}

// Periodic background sync (if supported)
self.addEventListener('periodicsync', (event) => {
    if (event.tag === 'sync-data') {
        event.waitUntil(performSync());
    }
});
