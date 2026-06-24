//! User/Auth 子系统的 service 层。
//!
//! 编排 UserRepo + domain 的密码/JWT 逻辑。
//! service 单测 mock repo，完全不碰 DB。

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::auth::{
    self, User, generate_jwt, hash_password, parse_jwt, validate_password, validate_phone,
    verify_password,
};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::user_repo::UserRepo;

/// 注册结果：用户信息 + JWT token。
#[derive(Debug)]
pub struct AuthResult {
    pub user: User,
    pub token: String,
}

#[async_trait]
pub trait UserService: Send + Sync {
    async fn register(&self, phone: &str, password: &str, code: &str) -> ServiceResult<AuthResult>;
    async fn login(&self, phone: &str, password: &str) -> ServiceResult<AuthResult>;
    async fn login_with_code(&self, phone: &str, code: &str) -> ServiceResult<AuthResult>;
    async fn reset_password(
        &self,
        phone: &str,
        new_password: &str,
        code: &str,
    ) -> ServiceResult<()>;
    async fn verify_token(&self, token: &str) -> ServiceResult<User>;
    fn jwt_secret(&self) -> &str;
}

pub struct UserServiceImpl {
    pub repo: Arc<dyn UserRepo>,
    pub jwt_secret: String,
    /// 固定验证码（迁移期：未接入短信，固定 111111）。
    pub fixed_code: String,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<dyn UserRepo>, jwt_secret: String) -> Self {
        Self {
            repo,
            jwt_secret,
            fixed_code: "111111".to_string(),
        }
    }

    fn check_code(&self, code: &str) -> ServiceResult<()> {
        if code == self.fixed_code {
            Ok(())
        } else {
            Err(ServiceError::Validation("验证码错误".into()))
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register(&self, phone: &str, password: &str, code: &str) -> ServiceResult<AuthResult> {
        if !validate_phone(phone) {
            return Err(ServiceError::Validation("手机号格式错误".into()));
        }
        if !validate_password(password) {
            return Err(ServiceError::Validation("密码至少 6 位".into()));
        }
        self.check_code(code)?;
        let hash = hash_password(password).map_err(|e| ServiceError::Internal(e.to_string()))?;
        let user = self.repo.insert(phone, &hash).await?;
        let token = generate_jwt(user.id, &self.jwt_secret)
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        Ok(AuthResult { user, token })
    }

    async fn login(&self, phone: &str, password: &str) -> ServiceResult<AuthResult> {
        let (user, stored_hash) = self
            .repo
            .find_by_phone(phone)
            .await?
            .ok_or(ServiceError::Unauthorized)?;
        let valid = verify_password(password, &stored_hash)
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        if !valid {
            return Err(ServiceError::Unauthorized);
        }
        let token = generate_jwt(user.id, &self.jwt_secret)
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        Ok(AuthResult { user, token })
    }

    async fn login_with_code(&self, phone: &str, code: &str) -> ServiceResult<AuthResult> {
        if !validate_phone(phone) {
            return Err(ServiceError::Validation("手机号格式错误".into()));
        }
        self.check_code(code)?;
        
        let user_opt = self.repo.find_by_phone(phone).await?;
        if let Some((user, _)) = user_opt {
            let token = generate_jwt(user.id, &self.jwt_secret)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            Ok(AuthResult { user, token })
        } else {
            let hash = hash_password("123456").map_err(|e| ServiceError::Internal(e.to_string()))?;
            let user = self.repo.insert(phone, &hash).await?;
            let token = generate_jwt(user.id, &self.jwt_secret)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            Ok(AuthResult { user, token })
        }
    }

    async fn reset_password(
        &self,
        phone: &str,
        new_password: &str,
        code: &str,
    ) -> ServiceResult<()> {
        if !validate_password(new_password) {
            return Err(ServiceError::Validation("密码至少 6 位".into()));
        }
        self.check_code(code)?;
        let (user, _) = self
            .repo
            .find_by_phone(phone)
            .await?
            .ok_or(ServiceError::NotFound)?;
        let hash =
            hash_password(new_password).map_err(|e| ServiceError::Internal(e.to_string()))?;
        self.repo.update_password(user.id, &hash).await?;
        Ok(())
    }

    async fn verify_token(&self, token: &str) -> ServiceResult<User> {
        let claims = parse_jwt(token, &self.jwt_secret).map_err(|_| ServiceError::Unauthorized)?;
        let user = self
            .repo
            .find_by_id(claims.user_id)
            .await?
            .ok_or(ServiceError::Unauthorized)?;
        Ok(user)
    }

    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RepoError, RepoResult};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub UserRepo {}
        #[async_trait]
        impl UserRepo for UserRepo {
            async fn find_by_phone(&self, phone: &str) -> RepoResult<Option<(User, String)>>;
            async fn find_by_id(&self, id: i32) -> RepoResult<Option<User>>;
            async fn insert(&self, phone: &str, password_hash: &str) -> RepoResult<User>;
            async fn update_password(&self, id: i32, password_hash: &str) -> RepoResult<()>;
        }
    }

    fn build_service(repo: MockUserRepo) -> UserServiceImpl {
        UserServiceImpl {
            repo: Arc::new(repo),
            jwt_secret: "test-secret".into(),
            fixed_code: "111111".into(),
        }
    }

    // ── register ──
    #[tokio::test]
    async fn register_success_returns_token() {
        let mut repo = MockUserRepo::new();
        repo.expect_insert()
            .with(eq("13800000001"), always())
            .returning(|phone, _| {
                Ok(User {
                    id: 1,
                    phone: phone.into(),
                })
            });
        let svc = build_service(repo);
        let result = svc
            .register("13800000001", "password123", "111111")
            .await
            .unwrap();
        assert_eq!(result.user.id, 1);
        assert!(!result.token.is_empty());
    }

    #[tokio::test]
    async fn register_invalid_phone_rejected() {
        let mut repo = MockUserRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let err = svc
            .register("123", "password123", "111111")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn register_short_password_rejected() {
        let mut repo = MockUserRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let err = svc
            .register("13800000001", "12345", "111111")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn register_wrong_code_rejected() {
        let mut repo = MockUserRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let err = svc
            .register("13800000001", "password123", "000000")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn register_duplicate_phone_returns_conflict() {
        let mut repo = MockUserRepo::new();
        repo.expect_insert()
            .returning(|_, _| Err(RepoError::UniqueViolation));
        let svc = build_service(repo);
        let err = svc
            .register("13800000001", "password123", "111111")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    // ── login ──
    #[tokio::test]
    async fn login_success_returns_token() {
        let password = "password123";
        let hash = hash_password(password).unwrap();
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(move |_| {
            Ok(Some((
                User {
                    id: 1,
                    phone: "13800000001".into(),
                },
                hash.clone(),
            )))
        });
        let svc = build_service(repo);
        let result = svc.login("13800000001", password).await.unwrap();
        assert_eq!(result.user.id, 1);
    }

    #[tokio::test]
    async fn login_user_not_found_unauthorized() {
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(|_| Ok(None));
        let svc = build_service(repo);
        let err = svc.login("13800000001", "password").await.unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized));
    }

    #[tokio::test]
    async fn login_wrong_password_unauthorized() {
        let hash = hash_password("correct").unwrap();
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(move |_| {
            Ok(Some((
                User {
                    id: 1,
                    phone: "138".into(),
                },
                hash.clone(),
            )))
        });
        let svc = build_service(repo);
        let err = svc.login("138", "wrong").await.unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized));
    }

    // ── reset_password ──
    #[tokio::test]
    async fn reset_password_success() {
        let hash = hash_password("old").unwrap();
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(move |_| {
            Ok(Some((
                User {
                    id: 1,
                    phone: "13800000001".into(),
                },
                hash.clone(),
            )))
        });
        repo.expect_update_password()
            .with(eq(1), always())
            .returning(|_, _| Ok(()));
        let svc = build_service(repo);
        svc.reset_password("13800000001", "newpassword", "111111")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reset_password_user_not_found() {
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(|_| Ok(None));
        repo.expect_update_password().times(0);
        let svc = build_service(repo);
        let err = svc
            .reset_password("13800000001", "newpassword", "111111")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn reset_password_short_password_rejected() {
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().times(0);
        let svc = build_service(repo);
        let err = svc
            .reset_password("13800000001", "12345", "111111")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    // ── verify_token ──
    #[tokio::test]
    async fn verify_valid_token_returns_user() {
        let token = generate_jwt(42, "test-secret").unwrap();
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_id().with(eq(42)).returning(|id| {
            Ok(Some(User {
                id,
                phone: "138".into(),
            }))
        });
        let svc = build_service(repo);
        let user = svc.verify_token(&token).await.unwrap();
        assert_eq!(user.id, 42);
    }

    #[tokio::test]
    async fn verify_invalid_token_unauthorized() {
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_id().times(0);
        let svc = build_service(repo);
        let err = svc.verify_token("invalid.token.here").await.unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized));
    }

    #[tokio::test]
    async fn verify_token_for_deleted_user_unauthorized() {
        let token = generate_jwt(99, "test-secret").unwrap();
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo);
        let err = svc.verify_token(&token).await.unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized));
    }

    #[tokio::test]
    async fn login_with_code_existing_user_success() {
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(|_| {
            Ok(Some((
                User {
                    id: 1,
                    phone: "13800000001".into(),
                },
                "password_hash".into(),
            )))
        });
        let svc = build_service(repo);
        let result = svc.login_with_code("13800000001", "111111").await.unwrap();
        assert_eq!(result.user.id, 1);
        assert!(!result.token.is_empty());
    }

    #[tokio::test]
    async fn login_with_code_new_user_auto_registers() {
        let mut repo = MockUserRepo::new();
        repo.expect_find_by_phone().returning(|_| Ok(None));
        repo.expect_insert()
            .with(eq("13800000001"), always())
            .returning(|phone, _| {
                Ok(User {
                    id: 2,
                    phone: phone.into(),
                })
            });
        let svc = build_service(repo);
        let result = svc.login_with_code("13800000001", "111111").await.unwrap();
        assert_eq!(result.user.id, 2);
        assert!(!result.token.is_empty());
    }

    // 确认 auth domain 函数被引用（避免 unused import 警告）
    #[test]
    fn auth_module_referenced() {
        assert!(auth::validate_phone("13800000001"));
    }
}
