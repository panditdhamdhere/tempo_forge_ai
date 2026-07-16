use crate::middleware::auth::AuthUser;
use axum::Json;
use serde::Deserialize;
use tempoforge_common::{ApiResponse, AppError, AppResult};
use tempoforge_sdk_generator::{SdkBundle, SdkLanguage, generate_sdk as build_sdk};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateSdkRequest {
    #[validate(length(min = 1, max = 80))]
    pub package_name: String,
    pub language: String,
    #[validate(length(min = 2, max = 2_000_000))]
    pub abi_json: String,
}

pub async fn generate_sdk(
    AuthUser(_user): AuthUser,
    Json(body): Json<GenerateSdkRequest>,
) -> AppResult<ApiResponse<SdkBundle>> {
    body.validate()?;
    let language = match body.language.to_lowercase().as_str() {
        "typescript" | "ts" => SdkLanguage::TypeScript,
        "rust" => SdkLanguage::Rust,
        "python" | "py" => SdkLanguage::Python,
        "go" | "golang" => SdkLanguage::Go,
        "java" => SdkLanguage::Java,
        other => {
            return Err(AppError::Validation(format!(
                "unsupported language: {other}"
            )))
        }
    };

    let bundle = build_sdk(language, &body.package_name, &body.abi_json)?;
    Ok(ApiResponse::new(bundle))
}
