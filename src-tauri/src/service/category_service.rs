// Category use-cases: validation, uniqueness, soft-delete, active toggle.

use crate::domain::{Category, Page, PageRequest};
use crate::error::{AppError, AppResult};
use crate::repository::CategoryRepository;

#[derive(Clone)]
pub struct CategoryService {
    repo: CategoryRepository,
}

impl CategoryService {
    pub fn new(repo: CategoryRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, req: PageRequest) -> AppResult<Page<Category>> {
        self.repo.list(&req).await
    }

    pub async fn create(&self, name: String, hsn_code: Option<String>) -> AppResult<Category> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("Category name is required".into()));
        }
        if self.repo.exists_name(&name, None).await? {
            return Err(AppError::Validation(format!(
                "A category named '{name}' already exists"
            )));
        }
        let hsn = normalize_opt(hsn_code);
        self.repo.insert(&name, hsn.as_deref()).await
    }

    pub async fn update(
        &self,
        id: i64,
        name: Option<String>,
        hsn_code: Option<Option<String>>,
    ) -> AppResult<Category> {
        let name = match name {
            Some(n) => {
                let n = n.trim().to_string();
                if n.is_empty() {
                    return Err(AppError::Validation("Category name cannot be empty".into()));
                }
                if self.repo.exists_name(&n, Some(id)).await? {
                    return Err(AppError::Validation(format!(
                        "A category named '{n}' already exists"
                    )));
                }
                Some(n)
            }
            None => None,
        };
        let hsn = hsn_code.map(normalize_opt);
        self.repo
            .update(id, name.as_deref(), hsn.as_ref().map(|o| o.as_deref()))
            .await
    }

    pub async fn set_active(&self, id: i64, is_active: bool) -> AppResult<Category> {
        self.repo.set_active(id, is_active).await
    }

    /// Soft delete: deactivate rather than removing, so items referencing this
    /// category keep their link.
    pub async fn delete(&self, id: i64) -> AppResult<Category> {
        self.repo.set_active(id, false).await
    }
}

/// Trim an optional string; treat blank as "no value".
fn normalize_opt(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
