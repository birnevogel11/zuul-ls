/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from 'path';
import { workspace, ExtensionContext, window } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind,
    Executable
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    const traceOutputChannel = window.createOutputChannel("Zuul Language Server trace");
    const command = process.env.ZUUL_LS_PATH || context.asAbsolutePath(path.join("server", "zuul-ls"));
    const run: Executable = {
      command,
      options: {
        env: {
          ...process.env,
          // eslint-disable-next-line @typescript-eslint/naming-convention
          RUST_LOG: "debug",
        },
      },
    };
    const serverOptions: ServerOptions = {
      run,
      debug: run,
    };

	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [
            { scheme: 'file', language: 'yaml' },
            { scheme: "untitled", language: "yaml" }  // New unsaved YAML files
        ],
		synchronize: {
			// Notify the server about file changes to '.clientrc files contained in the workspace
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		},
        traceOutputChannel
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'zuul-ls',
		'Zuul Language Server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
