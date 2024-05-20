import * as path from "node:path";

import * as vscode from "vscode";

import { RelativePattern, Uri } from "vscode";
import { TransportKind, LanguageClient, InitializeRequest } from "vscode-languageclient/node"

import type { ExtensionContext, WorkspaceFolder, TextDocument, WorkspaceFoldersChangeEvent } from "vscode";
import type { ServerOptions, LanguageClientOptions } from "vscode-languageclient/node";

const extensionName = 'TRAX Language Support';
const outputChannel = vscode.window.createOutputChannel(extensionName);

const clients: Map<string, LanguageClient> = new Map();

async function reloadDocument(uri: Uri) {
	let doc = vscode.workspace.textDocuments.find((d: TextDocument) => d.uri.toString() === uri.toString());

	if (doc) {
		doc = await vscode.languages.setTextDocumentLanguage(doc, 'plaintext');
		await vscode.languages.setTextDocumentLanguage(doc, 'trax');
	} else {
		await vscode.workspace.openTextDocument(uri);
	}
}

function traxFilesInWorkspaceFolderPattern(folder: WorkspaceFolder) {
	return new RelativePattern(folder, '**/*.trax');
}

async function openTraxFilesInWorkspaceFolder(folder: WorkspaceFolder) {
	const pattern = traxFilesInWorkspaceFolderPattern(folder);
	const uris = await vscode.workspace.findFiles(pattern);
	return Promise.all(uris.map(reloadDocument));
}

function getServerOptions(context: ExtensionContext): ServerOptions {
	const serverModule = context.asAbsolutePath(path.join("dist", "server.cjs"));

	return {
		run: { module: serverModule, transport: TransportKind.ipc },
		debug: {
			module: serverModule,
			transport: TransportKind.ipc,
		}
	}
}

async function startClient(folder: WorkspaceFolder, context: ExtensionContext) {
	const deleteWatcher = vscode.workspace.createFileSystemWatcher(
		traxFilesInWorkspaceFolderPattern(folder),
		true,
		true
	);

	const createChangeWatcher = vscode.workspace.createFileSystemWatcher(
		traxFilesInWorkspaceFolderPattern(folder),
		false,
		false,
		true
	);

	context.subscriptions.push(deleteWatcher);
	context.subscriptions.push(createChangeWatcher);

	const serverOptions = getServerOptions(context);

	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{ language: 'trax', pattern: `${folder.uri.fsPath}/**/*.trax` },
		],
		synchronize: { fileEvents: deleteWatcher },
		diagnosticCollectionName: extensionName,
		workspaceFolder: folder,
		outputChannel,
	};

	const client = new LanguageClient(
		"trax-lsp-client",
		extensionName,
		serverOptions,
		clientOptions,
	)

	client.start();
	client.sendNotification("initialize");

	context.subscriptions.push(createChangeWatcher.onDidCreate(reloadDocument));
	context.subscriptions.push(createChangeWatcher.onDidChange(reloadDocument));

	await openTraxFilesInWorkspaceFolder(folder);

	clients.set(folder.uri.toString(), client);
}

async function stopClient(folder: string) {
	const client = clients.get(folder);
	if (client) {
		await client.stop();
	}
	clients.delete(folder);
}

function updateClients(context: ExtensionContext) {
	return async function ({ added, removed }: WorkspaceFoldersChangeEvent) {
		for (const folder of removed) await stopClient(folder.uri.toString());
		for (const folder of added) await startClient(folder, context);
	};
}


export async function activate(context: ExtensionContext) {
	const folders = vscode.workspace.workspaceFolders || [];
	for (const folder of folders) await startClient(folder, context);

	vscode.workspace.onDidChangeWorkspaceFolders(updateClients(context));

	const restartServer = vscode.commands.registerCommand(
		"extension.trax.restartServer",
		async () => {
			deactivate();
			for (const folder of folders) await startClient(folder, context);
		}
	);

	context.subscriptions.push(restartServer);
};


export async function deactivate() {
	await Promise.all(
		[...clients.values()].map((client) => {
			return client.stop();
		})
	);
}