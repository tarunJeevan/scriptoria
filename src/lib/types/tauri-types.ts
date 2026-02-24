// src/lib/types/tauri-types.ts

// ==================================================================
// DOCUMENT TYPES
// ==================================================================

export interface DocumentMetadata {
	tags: string[];
	editor_state?: unknown;
	[key: string]: unknown;
}

export interface DecryptedDocument {
	id: number;
	project_id: number;
	title: string;
	content: string;
	word_count: number;
	char_count: number;
	created_at: string;
	updated_at: string;
	last_edited_at: string | null;
	doc_type: string;
	entity_type: string | null;
	parent_id: number | null;
	display_order: number;
	deleted: boolean;
	metadata: DocumentMetadata;
}

export interface DocumentListItem {
	id: number;
	title: string;
	doc_type: string;
	word_count: number;
	updated_at: string;
	parent_id: number | null;
	display_order: number;
}

export interface DocumentVersion {
	id: number;
	document_id: number;
	version_number: number;
	content_encrypted: number[]; // Vec<u8> serializes as number array
	word_count: number | null;
	char_count: number | null;
	created_at: string;
	label: string | null;
	diff_summary: string | null;
	is_delta: boolean;
}

// ==================================================================
// COMMAND PARAMETERS
// ==================================================================

export interface CreateDocumentParams {
	project_id: number;
	title: string;
	content: string;
	doc_type?: string;
	entity_type?: string;
	parent_id?: number;
}

export interface UpdateDocumentParams {
	title?: string;
	content?: string;
	entity_type?: string;
	parent_id?: number;
	display_order?: number;
	metadata?: DocumentMetadata;
}

// ==================================================================
// TAURI COMMAND WRAPPERS
// ==================================================================

import { invoke } from '@tauri-apps/api/core';

export const documentCommands = {
	// Create a new document
	async create(params: CreateDocumentParams): Promise<DecryptedDocument> {
		return await invoke('create_document', {
			project_id: params.project_id,
			title: params.title,
			content: params.content,
			docType: params.doc_type,
			entityType: params.entity_type,
			parentId: params.parent_id
		});
	},

	// Read a document by ID
	async read(documentId: number): Promise<DecryptedDocument> {
		return await invoke('read_document', { documentId });
	},

	// Update a document
	async update(documentId: number, params: UpdateDocumentParams): Promise<DecryptedDocument> {
		return await invoke('update_document', {
			documentId,
			title: params.title,
			content: params.content,
			entityType: params.entity_type,
			parentId: params.parent_id,
			displayOrder: params.display_order,
			metadata: params.metadata
		});
	},

	// Soft delete a document
	async delete(documentId: number): Promise<void> {
		await invoke('delete_document', { documentId });
	},

	// List all documents in a project
	async list(projectId: number): Promise<DocumentListItem[]> {
		return await invoke('list_documents', { projectId });
	},

	// List documents by entity type
	async listByType(projectId: number, entityType?: string): Promise<DocumentListItem[]> {
		return await invoke('list_documents_by_type', { projectId, entityType });
	},

	// Count documents by entity type
	async countByType(projectId: number, entityType?: string): Promise<number> {
		return await invoke('count_documents_by_type', { projectId, entityType });
	}
};

export const versionCommands = {
	// Create a version snapshot
	async create(documentId: number, label?: string): Promise<number> {
		return await invoke('create_version', { documentId, label });
	},

	// List all versions for a document
	async list(documentId: number): Promise<DocumentVersion[]> {
		return await invoke('list_versions', { documentId });
	},

	// Get a specific version with decrypted content
	async get(versionId: number): Promise<[DocumentVersion, string]> {
		return await invoke('get_version', { versionId });
	},

	// Restore a document from a version
	async restore(documentId: number, versionId: number): Promise<DecryptedDocument> {
		return await invoke('restore_from_version', { documentId, versionId });
	}
};
