import * as vscode from 'vscode';

interface Finding {
  id: string;
  ruleId: string;
  title: string;
  message: string;
  severity: string;
  status: string;
  location: { path: string; startLine: number };
  cwe?: string;
  verified: boolean;
  evidence?: string[];
  chain?: string[];
}

interface SarifReport {
  runs?: Array<{
    results?: Array<{
      ruleId?: string;
      message?: { text?: string };
      level?: string;
      locations?: Array<{
        physicalLocation?: {
          artifactLocation?: { uri?: string };
          region?: { startLine?: number };
        };
      }>;
      properties?: Record<string, unknown>;
    }>;
  }>;
}

function severityOrder(s: string): number {
  const map: Record<string, number> = {
    critical: 0,
    high: 1,
    medium: 2,
    low: 3,
    info: 4,
    warning: 5,
    error: 5,
    none: 6,
  };
  return map[s.toLowerCase()] ?? 6;
}

function severityColor(s: string): string {
  const map: Record<string, string> = {
    critical: '#e74c3c',
    high: '#e67e22',
    medium: '#f1c40f',
    low: '#3498db',
    info: '#95a5a6',
  };
  return map[s.toLowerCase()] ?? '#95a5a6';
}

export class SecuritySidebarProvider implements vscode.WebviewViewProvider {
  private _view?: vscode.WebviewView;
  private _findings: Finding[] = [];

  constructor(private readonly _extensionUri: vscode.Uri) {}

  dispose() {
    this._view = undefined;
  }

  resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken,
  ): void {
    this._view = webviewView;
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri],
    };
    webviewView.webview.html = this._getHtml();
    webviewView.webview.onDidReceiveMessage((msg) => {
      switch (msg.type) {
        case 'jumpToLocation':
          this._jumpToLocation(msg.path, msg.line);
          break;
        case 'toggleVerified':
          this._toggleVerified(msg.id);
          break;
      }
    });
  }

  loadSarif(report: SarifReport) {
    this._findings = [];
    for (const run of report.runs ?? []) {
      for (const result of run.results ?? []) {
        const loc = result.locations?.[0]?.physicalLocation;
        this._findings.push({
          id: result.ruleId ?? 'unknown',
          ruleId: result.ruleId ?? '',
          title: result.message?.text ?? result.ruleId ?? 'Unknown',
          message: result.message?.text ?? '',
          severity: result.level ?? 'warning',
          status: 'open',
          location: {
            path: loc?.artifactLocation?.uri ?? '',
            startLine: loc?.region?.startLine ?? 1,
          },
          verified: false,
        });
      }
    }
    this._sortAndRender();
  }

  getSarif(): SarifReport {
    return {
      runs: [
        {
          results: this._findings.map((f) => ({
            ruleId: f.ruleId,
            message: { text: f.message },
            level: f.severity,
            locations: [
              {
                physicalLocation: {
                  artifactLocation: { uri: f.location.path },
                  region: { startLine: f.location.startLine },
                },
              },
            ],
            properties: { verified: f.verified, status: f.status },
          })),
        },
      ],
    };
  }

  private _sortAndRender() {
    this._findings.sort((a, b) => severityOrder(a.severity) - severityOrder(b.severity));
    this._render();
  }

  private _render() {
    if (!this._view) return;
    const items = this._findings
      .map(
        (f, i) => `
      <div class="finding" style="border-left: 4px solid ${severityColor(f.severity)}">
        <div class="finding-header">
          <span class="severity-badge" style="background:${severityColor(f.severity)}">
            ${f.severity.toUpperCase()}
          </span>
          <span class="finding-title">${this._escapeHtml(f.title)}</span>
          <span class="verification" data-verified="${f.verified}" data-index="${i}">
            ${f.verified ? '✓ Verified' : '○ Unverified'}
          </span>
        </div>
        <div class="finding-details">
          <span class="finding-location" data-path="${f.location.path}" data-line="${f.location.startLine}">
            ${f.location.path}:${f.location.startLine}
          </span>
          ${f.cwe ? `<span class="cwe-badge">${f.cwe}</span>` : ''}
        </div>
        <div class="finding-message">${this._escapeHtml(f.message)}</div>
        ${f.evidence ? `<div class="evidence">${f.evidence.map(e => `<code>${this._escapeHtml(e)}</code>`).join('')}</div>` : ''}
        ${f.chain ? `<div class="chain">Chain: ${f.chain.join(' → ')}</div>` : ''}
      </div>`,
      )
      .join('\n');

    const summary = {
      total: this._findings.length,
      critical: this._findings.filter((f) => f.severity === 'critical').length,
      high: this._findings.filter((f) => f.severity === 'high').length,
      verified: this._findings.filter((f) => f.verified).length,
    };

    this._view.webview.html = this._getHtml(`
      <div class="summary-bar">
        <span>${summary.total} findings</span>
        <span class="crit-count">🔴 ${summary.critical} crit</span>
        <span class="high-count">🟠 ${summary.high} high</span>
        <span>✓ ${summary.verified} verified</span>
      </div>
      <div class="findings-list">
        ${items || '<div class="empty">No findings imported. Run "BugBee: Import SARIF report"</div>'}
      </div>
    `);
  }

  private _jumpToLocation(path: string, line: number) {
    const uri = vscode.Uri.file(path);
    vscode.window.showTextDocument(uri, { selection: new vscode.Range(line - 1, 0, line - 1, 0) });
  }

  private _toggleVerified(id: string) {
    const f = this._findings.find((x) => x.id === id);
    if (f) {
      f.verified = !f.verified;
      this._sortAndRender();
    }
  }

  private _escapeHtml(s: string): string {
    return s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  private _getHtml(bodyContent?: string): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body {
      font-family: var(--vscode-font-family, -apple-system, BlinkMacSystemFont, sans-serif);
      font-size: var(--vscode-font-size, 13px);
      color: var(--vscode-foreground);
      background: var(--vscode-sideBar-background, #1e1e1e);
      padding: 0;
      margin: 0;
    }
    .summary-bar {
      display: flex;
      gap: 12px;
      padding: 8px 12px;
      background: var(--vscode-sideBarSectionHeader-background);
      border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border);
      font-weight: 600;
      font-size: 12px;
      position: sticky;
      top: 0;
      z-index: 10;
    }
    .summary-bar .crit-count { color: #e74c3c; }
    .summary-bar .high-count { color: #e67e22; }
    .findings-list { padding: 4px 8px; }
    .finding {
      background: var(--vscode-sideBar-background);
      border-radius: 4px;
      margin: 6px 0;
      padding: 8px 10px;
      cursor: pointer;
      transition: background 0.15s;
    }
    .finding:hover { background: var(--vscode-list-hoverBackground); }
    .finding-header {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 4px;
    }
    .severity-badge {
      display: inline-block;
      padding: 1px 6px;
      border-radius: 3px;
      color: #fff;
      font-size: 10px;
      font-weight: 700;
      letter-spacing: 0.5px;
      flex-shrink: 0;
    }
    .finding-title {
      flex: 1;
      font-weight: 600;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .verification {
      font-size: 11px;
      cursor: pointer;
      flex-shrink: 0;
    }
    .verification[data-verified="true"] { color: #27ae60; }
    .verification[data-verified="false"] { color: #95a5a6; }
    .finding-details {
      display: flex;
      gap: 8px;
      align-items: center;
      font-size: 11px;
      color: var(--vscode-descriptionForeground);
    }
    .finding-location {
      cursor: pointer;
      text-decoration: underline;
      text-decoration-style: dotted;
    }
    .finding-location:hover { color: var(--vscode-textLink-foreground); }
    .cwe-badge {
      background: var(--vscode-badge-background);
      color: var(--vscode-badge-foreground);
      padding: 1px 5px;
      border-radius: 3px;
      font-size: 10px;
    }
    .finding-message {
      font-size: 12px;
      margin-top: 4px;
      color: var(--vscode-descriptionForeground);
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
    .evidence code {
      display: block;
      background: var(--vscode-textCodeBlock-background);
      padding: 4px 6px;
      margin: 4px 0;
      border-radius: 3px;
      font-size: 11px;
      white-space: pre-wrap;
      word-break: break-all;
    }
    .chain {
      font-size: 11px;
      margin-top: 4px;
      color: #8e44ad;
      font-style: italic;
    }
    .empty {
      padding: 24px 12px;
      text-align: center;
      color: var(--vscode-descriptionForeground);
      font-style: italic;
    }
  </style>
</head>
<body>
  ${bodyContent ?? '<div class="empty">BugBee Security Sidebar — waiting for findings</div>'}
  <script>
    (function() {
      document.addEventListener('click', function(e) {
        const loc = e.target.closest('.finding-location');
        if (loc) {
          vscode.postMessage({
            type: 'jumpToLocation',
            path: loc.dataset.path,
            line: parseInt(loc.dataset.line, 10)
          });
        }
        const ver = e.target.closest('.verification');
        if (ver) {
          const index = parseInt(ver.dataset.index, 10);
          vscode.postMessage({
            type: 'toggleVerified',
            id: index.toString()
          });
        }
      });
    })();
  </script>
</body>
</html>`;
  }
}
