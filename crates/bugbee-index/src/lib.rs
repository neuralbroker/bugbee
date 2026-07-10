//! Codebase indexer: file inventory, symbols (regex-based MVP), repo map.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use bugbee_core::{BugbeeError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Python,
    JavaScript,
    TypeScript,
    Go,
    Other,
}

impl Lang {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "py" => Lang::Python,
            "js" | "jsx" | "mjs" | "cjs" => Lang::JavaScript,
            "ts" | "tsx" => Lang::TypeScript,
            "go" => Lang::Go,
            _ => Lang::Other,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Go => "go",
            Lang::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub path: String,
    pub lang: Lang,
    pub hash: String,
    pub lines: u32,
    pub size: u64,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoIndex {
    pub root: String,
    pub files: Vec<IndexedFile>,
    pub symbol_index: HashMap<String, Vec<String>>,
}

impl RepoIndex {
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn by_lang(&self, lang: Lang) -> impl Iterator<Item = &IndexedFile> {
        self.files.iter().filter(move |f| f.lang == lang)
    }

    /// Simple repo map: top files by symbol density + size rank for LLM context packing.
    pub fn repomap(&self, limit: usize) -> Vec<&IndexedFile> {
        let mut scored: Vec<_> = self
            .files
            .iter()
            .map(|f| {
                let score = f.symbols.len() as f64 * 2.0
                    + (f.imports.len() as f64)
                    + (f.lines as f64).sqrt() * 0.1;
                (score, f)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(_, f)| f).collect()
    }

    pub fn read_file(&self, rel: &str) -> Result<String> {
        let path = Path::new(&self.root).join(rel);
        if bugbee_core::Redactor::is_sensitive_path(rel) {
            return Err(BugbeeError::Engine(format!(
                "refusing to read sensitive path: {rel}"
            )));
        }
        Ok(fs::read_to_string(path)?)
    }
}

pub struct Indexer {
    pub root: PathBuf,
}

impl Indexer {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn build(&self) -> Result<RepoIndex> {
        let mut index = RepoIndex {
            root: self.root.display().to_string(),
            files: Vec::new(),
            symbol_index: HashMap::new(),
        };

        let root = self.root.clone();
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .filter_entry(move |entry| !is_excluded_path(entry.path(), &root))
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let lang = Lang::from_path(path);
            if lang == Lang::Other {
                continue;
            }
            let rel = path
                .strip_prefix(&self.root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if bugbee_core::Redactor::is_sensitive_path(&rel) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if content.len() > 1_500_000 {
                continue;
            }

            let hash = {
                let mut h = Sha256::new();
                h.update(content.as_bytes());
                hex::encode(h.finalize())
            };
            let lines = content.lines().count() as u32;
            let (symbols, imports) = extract_symbols_and_imports(&rel, lang, &content);

            for s in &symbols {
                index
                    .symbol_index
                    .entry(s.name.clone())
                    .or_default()
                    .push(rel.clone());
            }

            index.files.push(IndexedFile {
                path: rel,
                lang,
                hash,
                lines,
                size: content.len() as u64,
                symbols,
                imports,
            });
        }

        index.files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(index)
    }
}

/// Directories that are generated, third-party, or Bugbee's own local state.
/// These exclusions apply even when a repository has no `.gitignore`, preventing
/// dependency trees from consuming scan capacity or polluting evidence.
fn is_excluded_path(path: &Path, root: &Path) -> bool {
    const EXCLUDED: &[&str] = &[
        ".bugbee",
        ".git",
        ".hg",
        ".svn",
        ".venv",
        "__pycache__",
        "bower_components",
        "build",
        "coverage",
        "dist",
        "node_modules",
        "target",
        "vendor",
    ];

    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .any(|component| EXCLUDED.contains(&component))
}

fn extract_symbols_and_imports(
    file: &str,
    lang: Lang,
    content: &str,
) -> (Vec<Symbol>, Vec<String>) {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();

    let (fn_re, imp_res): (Regex, Vec<Regex>) = match lang {
        Lang::Python => (
            Regex::new(r"(?m)^(async\s+)?def\s+(\w+)\s*\(").unwrap(),
            vec![
                Regex::new(r"(?m)^import\s+([\w\.]+)").unwrap(),
                Regex::new(r"(?m)^from\s+([\w\.]+)\s+import").unwrap(),
            ],
        ),
        Lang::JavaScript | Lang::TypeScript => (
            Regex::new(
                r"(?m)^(?:export\s+)?(?:async\s+)?function\s+(\w+)|(?:export\s+)?const\s+(\w+)\s*=\s*(?:async\s*)?\(",
            )
            .unwrap(),
            vec![
                Regex::new(r#"(?m)^import\s+.*?from\s+['"]([^'"]+)['"]"#).unwrap(),
                Regex::new(r#"(?m)^const\s+\w+\s*=\s*require\(['"]([^'"]+)['"]\)"#).unwrap(),
            ],
        ),
        Lang::Go => (
            Regex::new(r"(?m)^func\s+(?:\([^)]+\)\s+)?(\w+)\s*\(").unwrap(),
            vec![Regex::new(r#"(?m)^\s*"([^"]+)"\s*$"#).unwrap()],
        ),
        Lang::Other => return (symbols, imports),
    };

    for (i, line) in content.lines().enumerate() {
        let line_no = (i + 1) as u32;
        if let Some(c) = fn_re.captures(line) {
            let name = c
                .get(2)
                .or_else(|| c.get(1))
                .or_else(|| c.get(3))
                .map(|m| m.as_str())
                .unwrap_or("");
            // Fix capture groups per language
            let name = match lang {
                Lang::Python => c.get(2).map(|m| m.as_str()).unwrap_or(""),
                Lang::JavaScript | Lang::TypeScript => c
                    .get(1)
                    .or_else(|| c.get(2))
                    .map(|m| m.as_str())
                    .unwrap_or(""),
                Lang::Go => c.get(1).map(|m| m.as_str()).unwrap_or(""),
                Lang::Other => name,
            };
            if !name.is_empty() {
                symbols.push(Symbol {
                    name: name.to_string(),
                    kind: "function".into(),
                    line: line_no,
                    file: file.to_string(),
                });
            }
        }
        for re in &imp_res {
            if let Some(c) = re.captures(line) {
                if let Some(m) = c.get(1) {
                    imports.push(m.as_str().to_string());
                }
            }
        }
    }

    // class detection python
    if lang == Lang::Python {
        let class_re = Regex::new(r"(?m)^class\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(c) = class_re.captures(line) {
                symbols.push(Symbol {
                    name: c[1].to_string(),
                    kind: "class".into(),
                    line: (i + 1) as u32,
                    file: file.to_string(),
                });
            }
        }
    }

    (symbols, imports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn indexes_python() {
        let dir = tempfile_dir();
        let mut f = fs::File::create(dir.join("app.py")).unwrap();
        writeln!(f, "import os\ndef hello():\n    pass\n").unwrap();
        let idx = Indexer::new(&dir).build().unwrap();
        assert_eq!(idx.file_count(), 1);
        assert!(idx.files[0].symbols.iter().any(|s| s.name == "hello"));
    }

    #[test]
    fn excludes_generated_dependency_and_local_state_directories() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
        fs::create_dir_all(dir.join(".bugbee/cache")).unwrap();
        fs::create_dir_all(dir.join("target/generated")).unwrap();
        fs::write(dir.join("app.py"), "def app():\n    pass\n").unwrap();
        fs::write(
            dir.join("node_modules/pkg/index.js"),
            "function dependency() {}\n",
        )
        .unwrap();
        fs::write(
            dir.join(".bugbee/cache/agent.py"),
            "def state():\n    pass\n",
        )
        .unwrap();
        fs::write(
            dir.join("target/generated/generated.go"),
            "func generated() {}\n",
        )
        .unwrap();

        let idx = Indexer::new(&dir).build().unwrap();
        assert_eq!(idx.file_count(), 1);
        assert_eq!(idx.files[0].path, "app.py");
    }

    fn tempfile_dir() -> PathBuf {
        let p = std::env::temp_dir().join(format!("bugbee_test_{}", uuid_like()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn uuid_like() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
