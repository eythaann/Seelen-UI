use std::path::PathBuf;

use crate::{error::Result, resource::SluResource, state::Theme};

#[tokio::test]
async fn test_compatibility_with_older_schemas() -> Result<()> {
    Theme::load(&PathBuf::from("./mocks/themes/v2.3.0.yml"))
        .await
        .map_err(|e| format!("v2.3.0: {e}"))?;
    Theme::load(&PathBuf::from("./mocks/themes/v2.3.12.yml"))
        .await
        .map_err(|e| format!("v2.3.12: {e}"))?;
    Ok(())
}
