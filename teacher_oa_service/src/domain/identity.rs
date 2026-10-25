use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub enum DataScope {
    #[serde(rename = "SELF")]
    SelfOnly,
    Department,
    All,
}

impl DataScope {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "SELF" => Some(Self::SelfOnly),
            "DEPARTMENT" => Some(Self::Department),
            "ALL" => Some(Self::All),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PermissionGrant {
    pub permission: String,
    pub scope: DataScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct TeacherProfile {
    pub id: i64,
    pub user_id: i64,
    pub employee_no: String,
    pub name: String,
    pub department_id: i64,
    pub department_name: String,
    pub subject_code: String,
    pub employment_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CurrentUser {
    pub id: i64,
    pub display_name: String,
    pub roles: Vec<String>,
    pub grants: Vec<PermissionGrant>,
    pub teacher: TeacherProfile,
}

impl CurrentUser {
    pub fn can_read_teacher(&self, target: &TeacherProfile) -> bool {
        self.grants.iter().any(|grant| {
            grant.permission == "teachers:read"
                && match grant.scope {
                    DataScope::SelfOnly => self.teacher.id == target.id,
                    DataScope::Department => self.teacher.department_id == target.department_id,
                    DataScope::All => true,
                }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_role_scope_cannot_broaden_teacher_read_permission() {
        let profile = TeacherProfile {
            id: 1,
            user_id: 1,
            employee_no: "T1".into(),
            name: "测试".into(),
            department_id: 1,
            department_name: "测试组".into(),
            subject_code: "math".into(),
            employment_status: "active".into(),
        };
        let user = CurrentUser {
            id: 1,
            display_name: "测试".into(),
            roles: vec![],
            grants: vec![
                PermissionGrant {
                    permission: "teachers:read".into(),
                    scope: DataScope::SelfOnly,
                },
                PermissionGrant {
                    permission: "reviews:write".into(),
                    scope: DataScope::All,
                },
            ],
            teacher: profile.clone(),
        };
        assert!(user.can_read_teacher(&profile));
        assert!(!user.can_read_teacher(&TeacherProfile {
            id: 2,
            department_id: 2,
            ..profile
        }));
    }
}
