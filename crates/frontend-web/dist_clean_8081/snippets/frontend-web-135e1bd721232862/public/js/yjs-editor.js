// Yjs Editor Integration
// This provides the JavaScript side of the collaborative editor

import * as Y from 'yjs';

const editors = new Map();

export function initEditor(elementId, initialText) {
    // Create Yjs document
    const ydoc = new Y.Doc();
    const ytext = ydoc.getText('content');

    // Set initial content
    if (initialText) {
        ytext.insert(0, initialText);
    }

    // Store editor instance
    editors.set(elementId, {
        doc: ydoc,
        text: ytext,
        element: document.getElementById(elementId)
    });

    // Bind to content editable
    const element = document.getElementById(elementId);
    if (element) {
        // Listen for local changes
        element.addEventListener('input', () => {
            const newText = element.textContent || '';
            const delta = ytext.toString();

            // Calculate diff and apply
            if (newText !== delta) {
                ytext.delete(0, ytext.length);
                ytext.insert(0, newText);
            }
        });

        // Listen for remote changes
        ytext.observe(() => {
            const currentText = element.textContent || '';
            const newText = ytext.toString();

            if (currentText !== newText) {
                element.textContent = newText;
            }
        });
    }

    return elementId;
}

export function getEditorUpdate(editorId) {
    const editor = editors.get(editorId);
    if (!editor) return new Uint8Array();

    // Get update since last sync
    const stateVector = Y.encodeStateVector(editor.doc);
    const update = Y.encodeStateAsUpdate(editor.doc);

    return update;
}

export function applyEditorUpdate(editorId, update) {
    const editor = editors.get(editorId);
    if (!editor) return;

    // Apply update from server
    Y.applyUpdate(editor.doc, new Uint8Array(update));
}

// Toolbar actions
document.addEventListener('DOMContentLoaded', () => {
    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.toolbar-btn');
        if (!btn) return;

        const action = btn.dataset.action;
        const editor = btn.closest('.rich-text-editor');
        const content = editor?.querySelector('.editor-content');

        if (!content) return;

        content.focus();

        switch (action) {
            case 'bold':
                document.execCommand('bold');
                break;
            case 'italic':
                document.execCommand('italic');
                break;
            case 'underline':
                document.execCommand('underline');
                break;
            case 'heading1':
                document.execCommand('formatBlock', false, 'h1');
                break;
            case 'heading2':
                document.execCommand('formatBlock', false, 'h2');
                break;
            case 'bullet-list':
                document.execCommand('insertUnorderedList');
                break;
            case 'ordered-list':
                document.execCommand('insertOrderedList');
                break;
        }
    });
});
