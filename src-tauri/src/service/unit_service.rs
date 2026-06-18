// Unit use-cases: validation, unique code, soft-delete, active toggle.

use crate::domain::{Page, PageRequest, Unit};
use crate::error::{AppError, AppResult};
use crate::repository::UnitRepository;

#[derive(Clone)]
pub struct UnitService {
    repo: UnitRepository,
}

impl UnitService {
    pub fn new(repo: UnitRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, req: PageRequest) -> AppResult<Page<Unit>> {
        self.repo.list(&req).await
    }

    pub async fn create(&self, unit_name: String, unit_code: String) -> AppResult<Unit> {
        let name = unit_name.trim().to_string();
        let code = unit_code.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("Unit name is required".into()));
        }
        if code.is_empty() {
            return Err(AppError::Validation("Unit code is required".into()));
        }
        if self.repo.exists_code(&code, None).await? {
            return Err(AppError::Validation(format!(
                "A unit with code '{code}' already exists"
            )));
        }
        self.repo.insert(&name, &code).await
    }

    pub async fn update(
        &self,
        id: i64,
        unit_name: Option<String>,
        unit_code: Option<String>,
    ) -> AppResult<Unit> {
        let name = match unit_name {
            Some(n) => {
                let n = n.trim().to_string();
                if n.is_empty() {
                    return Err(AppError::Validation("Unit name cannot be empty".into()));
                }
                Some(n)
            }
            None => None,
        };
        let code = match unit_code {
            Some(c) => {
                let c = c.trim().to_string();
                if c.is_empty() {
                    return Err(AppError::Validation("Unit code cannot be empty".into()));
                }
                if self.repo.exists_code(&c, Some(id)).await? {
                    return Err(AppError::Validation(format!(
                        "A unit with code '{c}' already exists"
                    )));
                }
                Some(c)
            }
            None => None,
        };
        self.repo.update(id, name.as_deref(), code.as_deref()).await
    }

    pub async fn set_active(&self, id: i64, is_active: bool) -> AppResult<Unit> {
        self.repo.set_active(id, is_active).await
    }

    pub async fn delete(&self, id: i64) -> AppResult<Unit> {
        self.repo.set_active(id, false).await
    }
}
