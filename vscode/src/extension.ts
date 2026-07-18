import * as vscode from 'vscode';
import { SecuritySidebarProvider } from './securityPanel';

let sidebarProvider: SecuritySidebarProvider | undefined;

export function activate(context: vscode.ExtensionContext) {
  sidebarProvider = new SecuritySidebarProvider(context.extensionUri);
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      'bugbee.securitySidebar',
      sidebarProvider,
      { webviewOptions: { retainContextWhenHidden: true } },
    ),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('bugbee.importSarif', async () => {
      const uris = await vscode.window.showOpenDialog({
        canSelectMany: false,
        filters: { 'SARIF Reports': ['sarif', 'sarif.json'] },
      });
      if (uris && uris[0]) {
        const text = (await vscode.workspace.fs.readFile(uris[0])).toString();
        try {
          const report = JSON.parse(text);
          sidebarProvider?.loadSarif(report);
          vscode.window.showInformationMessage('BugBee: SARIF report loaded');
        } catch {
          vscode.window.showErrorMessage('BugBee: invalid SARIF file');
        }
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('bugbee.exportSarif', async () => {
      const uri = await vscode.window.showSaveDialog({
        filters: { 'SARIF Reports': ['sarif.json'] },
      });
      if (uri && sidebarProvider) {
        const sarif = sidebarProvider.getSarif();
        await vscode.workspace.fs.writeFile(uri, Buffer.from(JSON.stringify(sarif, null, 2)));
        vscode.window.showInformationMessage('BugBee: findings exported');
      }
    }),
  );
}

export function deactivate() {
  sidebarProvider?.dispose();
}
