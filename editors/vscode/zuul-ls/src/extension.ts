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

function getServerExecutablePath(context: ExtensionContext): string {
    let platformBinary: string;
    const arch = process.arch;  // Use process.arch to check architecture

    switch (process.platform) {
        case 'win32':
            platformBinary = 'zuul-ls.exe';  // Windows binary
            break;
        case 'darwin':
            if (arch === 'x64') {
                platformBinary = 'zuul-ls.x86_64-apple-darwin';  // macOS x86_64 binary
            } else if (arch === 'arm64') {
                platformBinary = 'zuul-ls.aarch64-apple-darwin';  // macOS ARM64 binary
            } else {
                throw new Error(`Unsupported architecture for macOS: ${arch}`);
            }
            break;
        case 'linux':
            if (arch === 'x64') {
                platformBinary = 'zuul-ls.x86_64-unknown-linux-gnu';  // Linux x86_64 binary
            } else if (arch === 'arm64') {
                platformBinary = 'zuul-ls.aarch64-unknown-linux-gnu';  // Linux ARM64 binary
            } else {
                throw new Error(`Unsupported architecture for Linux: ${arch}`);
            }
            break;
        default:
            throw new Error(`Unsupported platform: ${process.platform}`);
    }

    // Resolve the binary path within the extension's folder
    return context.asAbsolutePath(path.join('server', platformBinary));
}

export function activate(context: ExtensionContext) {
    const traceOutputChannel = window.createOutputChannel("Zuul Language Server trace");
    const command = process.env.ZUUL_LS_PATH || getServerExecutablePath(context);
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
