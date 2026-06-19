// User type use-cases: validation, unique name, soft-delete, active toggle.

use crate::domain::{Page, PageRequest, UserType};
use crate::error::{AppError, AppResult};
use crate::repository::UserTypeRepository;

#[derive(Clone)]
pub struct UserTypeService {
    repo: UserTypeRepository,
}

impl UserTypeService {
    pub fn new(repo: UserTypeRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, req: PageRequest) -> AppResult<Page<UserType>> {
        self.repo.list(&req).await
    }

    pub async fn create(&self, name: String) -> AppResult<UserType> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("User type name is required".into()));
        }
        if self.repo.exists_name(&name, None).await? {
            return Err(AppError::Validation(format!(
                "A user type named '{name}' already exists"
            )));
        }
        self.repo.insert(&name).await
    }

    pub async fn update(&self, id: i64, name: String) -> AppResult<UserType> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("User type name cannot be empty".into()));
        }
        if self.repo.exists_name(&name, Some(id)).await? {
            return Err(AppError::Validation(format!(
                "A user type named '{name}' already exists"
            )));
        }
        self.repo.update(id, &name).await
    }

    pub async fn set_active(&self, id: i64, is_active: bool) -> AppResult<UserType> {
        self.repo.set_active(id, is_active).await
    }

    /// Soft delete: deactivate so users referencing this type keep their link.
    pub async fn delete(&self, id: i64) -> AppResult<UserType> {
        self.repo.set_active(id, false).await
    }
}
