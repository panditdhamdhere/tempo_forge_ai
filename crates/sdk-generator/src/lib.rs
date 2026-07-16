//! Generate typed SDKs from contract ABI metadata.

use serde::{Deserialize, Serialize};
use tempoforge_common::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkLanguage {
    TypeScript,
    Rust,
    Python,
    Go,
    Java,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkBundle {
    pub language: SdkLanguage,
    pub package_name: String,
    pub files: Vec<SdkFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbiFunction {
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<AbiParam>,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbiParam {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

pub fn generate_sdk(
    language: SdkLanguage,
    package_name: &str,
    abi_json: &str,
) -> AppResult<SdkBundle> {
    let abi: Vec<AbiFunction> = serde_json::from_str(abi_json)
        .map_err(|e| AppError::Validation(format!("invalid ABI JSON: {e}")))?;

    let functions: Vec<&AbiFunction> = abi.iter().filter(|f| f.kind == "function").collect();

    let files = match language {
        SdkLanguage::TypeScript => vec![SdkFile {
            path: format!("src/{package_name}.ts"),
            content: ts_sdk(package_name, &functions),
        }],
        SdkLanguage::Rust => vec![SdkFile {
            path: "src/lib.rs".into(),
            content: rust_sdk(package_name, &functions),
        }],
        SdkLanguage::Python => vec![SdkFile {
            path: format!("{}.py", package_name.replace('-', "_")),
            content: python_sdk(package_name, &functions),
        }],
        SdkLanguage::Go => vec![SdkFile {
            path: format!("{}.go", package_name.replace('-', "_")),
            content: go_sdk(package_name, &functions),
        }],
        SdkLanguage::Java => vec![SdkFile {
            path: format!(
                "src/main/java/ai/tempoforge/{}.java",
                pascal(package_name)
            ),
            content: java_sdk(package_name, &functions),
        }],
    };

    Ok(SdkBundle {
        language,
        package_name: package_name.to_string(),
        files,
    })
}

fn ts_sdk(name: &str, functions: &[&AbiFunction]) -> String {
    let methods = functions
        .iter()
        .map(|f| {
            let args = f
                .inputs
                .iter()
                .map(|i| format!("{}: string", sanitize(&i.name)))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "  async {}({args}): Promise<unknown> {{\n    return this.client.write('{fname}', [{argnames}]);\n  }}",
                sanitize(&f.name),
                fname = f.name,
                argnames = f.inputs.iter().map(|i| sanitize(&i.name)).collect::<Vec<_>>().join(", "),
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "/** Auto-generated TempoForge SDK for {name} */\nexport class {cls} {{\n  constructor(private client: {{ write(method: string, args: unknown[]): Promise<unknown> }}) {{}}\n\n{methods}\n}}\n",
        cls = pascal(name)
    )
}

fn rust_sdk(name: &str, functions: &[&AbiFunction]) -> String {
    let methods = functions
        .iter()
        .map(|f| {
                format!(
                "    pub async fn {fname}(&self) -> anyhow::Result<()> {{\n        // Bind an alloy contract call for `{fname}` against `self.address`.\n        let _ = &self.address;\n        Ok(())\n    }}",
                fname = sanitize(&f.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "//! Auto-generated TempoForge Rust SDK for {name}\npub struct {cls} {{\n    pub address: String,\n}}\n\nimpl {cls} {{\n{methods}\n}}\n",
        cls = pascal(name)
    )
}

fn python_sdk(name: &str, functions: &[&AbiFunction]) -> String {
    let methods = functions
        .iter()
        .map(|f| {
            format!(
                "    async def {fname}(self):\n        return await self.client.write(\"{raw}\", [])\n",
                fname = sanitize(&f.name),
                raw = f.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Auto-generated TempoForge SDK for {name}\nclass {cls}:\n    def __init__(self, client):\n        self.client = client\n\n{methods}", cls = pascal(name))
}

fn go_sdk(name: &str, functions: &[&AbiFunction]) -> String {
    let methods = functions
        .iter()
        .map(|f| {
            format!(
                "func (c *{cls}) {fname}() error {{\n\treturn c.Client.Write(\"{raw}\", nil)\n}}\n",
                cls = pascal(name),
                fname = pascal(&f.name),
                raw = f.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("package {pkg}\n\ntype {cls} struct {{\n\tClient Writer\n}}\n\ntype Writer interface {{\n\tWrite(method string, args any) error\n}}\n\n{methods}", pkg = sanitize(name), cls = pascal(name))
}

fn java_sdk(name: &str, functions: &[&AbiFunction]) -> String {
    let methods = functions
        .iter()
        .map(|f| {
            format!(
                "    public Object {fname}() throws Exception {{\n        return client.write(\"{raw}\");\n    }}\n",
                fname = sanitize(&f.name),
                raw = f.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "package ai.tempoforge;\n\npublic class {cls} {{\n    private final Client client;\n    public {cls}(Client client) {{ this.client = client; }}\n\n{methods}\n    public interface Client {{ Object write(String method) throws Exception; }}\n}}\n",
        cls = pascal(name)
    )
}

fn sanitize(name: &str) -> String {
    let mut out = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if out.is_empty() {
        out = "fn".into();
    }
    out
}

fn pascal(name: &str) -> String {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_typescript() {
        let abi = r#"[{"name":"transfer","type":"function","inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}]}]"#;
        let bundle = generate_sdk(SdkLanguage::TypeScript, "payment-token", abi).unwrap();
        assert_eq!(bundle.files.len(), 1);
        assert!(bundle.files[0].content.contains("transfer"));
    }
}
