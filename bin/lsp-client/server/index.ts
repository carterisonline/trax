import {
	createConnection,
	ProposedFeatures,
	TextDocumentSyncKind,
} from 'vscode-languageserver/node';
import { TraxLspServerWasm } from '../dist/trax_lsp_server_wasm'; // eslint-disable-line node/no-unpublished-import

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

const pls = new TraxLspServerWasm();

connection.onNotification((...args) => {
	const result = pls.onNotification(...args);
	console.info(result);
	return result;
});

connection.onInitialize(() => {
	return {
		capabilities: {
			textDocumentSync: {
				openClose: true,
				save: true,
				change: TextDocumentSyncKind.Full,
			},
			workspace: {
				workspaceFolders: { supported: true },
				fileOperations: {
					didDelete: {
						filters: [{ pattern: { /* matches: 'folder', */ glob: '**' } }],
					},
				},
			},
		},
	};
});

connection.listen();