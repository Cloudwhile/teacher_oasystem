use super::{
    achievement_entities::{achievement, achievement_file, file, participant},
    entities::{department, identity, role_permission, teacher, user, user_role},
};
use crate::{
    api::error::ApiError,
    domain::identity::{
        CurrentUser, DataScope, DepartmentOption, PermissionGrant, TeacherDirectoryItem,
        TeacherDirectoryPage, TeacherDossier, TeacherProfile,
    },
};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
    sea_query::{Expr, LikeExpr, Query},
};
use std::collections::{BTreeMap, BTreeSet};

pub struct DirectoryFilter<'a> {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<&'a str>,
    pub department_id: Option<i64>,
    pub subject_code: Option<&'a str>,
    pub employment_status: Option<&'a str>,
}

pub async fn teacher_profile(
    db: &impl ConnectionTrait,
    id: i64,
) -> Result<TeacherProfile, ApiError> {
    let teacher = teacher::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::NotFound)?;
    let department = department::Entity::find_by_id(teacher.department_id)
        .one(db)
        .await?
        .ok_or(ApiError::Internal)?;
    Ok(TeacherProfile {
        id: teacher.id,
        user_id: teacher.user_id,
        employee_no: teacher.employee_no,
        name: teacher.name,
        department_id: teacher.department_id,
        department_name: department.name,
        subject_code: teacher.subject_code,
        employment_status: teacher.employment_status,
    })
}

pub async fn teacher_directory(
    db: &impl ConnectionTrait,
    filter: DirectoryFilter<'_>,
) -> Result<TeacherDirectoryPage, ApiError> {
    let dept_rows = department::Entity::find()
        .order_by_asc(department::Column::Name)
        .order_by_asc(department::Column::Id)
        .all(db)
        .await?;
    let dept_names: BTreeMap<i64, String> = dept_rows
        .iter()
        .map(|row| (row.id, row.name.clone()))
        .collect();
    let departments = dept_rows
        .into_iter()
        .map(|row| DepartmentOption {
            id: row.id,
            name: row.name,
        })
        .collect();

    let mut select = teacher::Entity::find();
    if let Some(dept_id) = filter.department_id {
        select = select.filter(teacher::Column::DepartmentId.eq(dept_id));
    }
    if let Some(subject) = filter.subject_code.map(str::trim).filter(|s| !s.is_empty()) {
        select = select.filter(teacher::Column::SubjectCode.eq(subject));
    }
    if let Some(status) = filter
        .employment_status
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !["active", "inactive"].contains(&status) {
            return Err(ApiError::Invalid("在职状态筛选值无效"));
        }
        select = select.filter(teacher::Column::EmploymentStatus.eq(status));
    }
    if let Some(keyword) = filter.search.map(str::trim).filter(|s| !s.is_empty()) {
        if keyword.chars().count() > 100 {
            return Err(ApiError::Invalid("搜索内容过长"));
        }
        let escaped = keyword
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_");
        let pattern = format!("%{escaped}%");
        select = select.filter(
            Condition::any()
                .add(teacher::Column::Name.like(LikeExpr::new(pattern.clone()).escape('!')))
                .add(teacher::Column::EmployeeNo.like(LikeExpr::new(pattern.clone()).escape('!')))
                .add(
                    teacher::Column::DepartmentId.in_subquery(
                        Query::select()
                            .column(department::Column::Id)
                            .from(department::Entity)
                            .and_where(
                                Expr::col(department::Column::Name)
                                    .like(LikeExpr::new(pattern).escape('!')),
                            )
                            .to_owned(),
                    ),
                ),
        );
    }

    let pager = select
        .order_by_asc(teacher::Column::Name)
        .order_by_asc(teacher::Column::Id)
        .paginate(db, filter.page_size);
    let total = pager.num_items().await?;
    let page = std::cmp::Ord::min(
        filter.page,
        std::cmp::Ord::max(total.div_ceil(filter.page_size), 1),
    );
    let rows = pager.fetch_page(page - 1).await?;

    if rows.is_empty() {
        return Ok(TeacherDirectoryPage {
            departments,
            items: Vec::new(),
            page,
            page_size: filter.page_size,
            total,
        });
    }

    let teacher_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let user_ids: Vec<i64> = rows.iter().map(|r| r.user_id).collect();

    let users: BTreeMap<i64, user::Model> = user::Entity::find()
        .filter(user::Column::Id.is_in(user_ids.clone()))
        .all(db)
        .await?
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    let mut roles_by_user: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for row in user_role::Entity::find()
        .filter(user_role::Column::UserId.is_in(user_ids))
        .order_by_asc(user_role::Column::RoleCode)
        .all(db)
        .await?
    {
        roles_by_user
            .entry(row.user_id)
            .or_default()
            .push(row.role_code);
    }

    let visible_records = achievement::Entity::find()
        .filter(achievement::Column::Status.ne("draft"))
        .filter(
            Condition::any()
                .add(achievement::Column::TeacherId.is_in(teacher_ids.clone()))
                .add(
                    achievement::Column::Id.in_subquery(
                        Query::select()
                            .column(participant::Column::AchievementId)
                            .from(participant::Entity)
                            .and_where(
                                Expr::col(participant::Column::TeacherId)
                                    .is_in(teacher_ids.clone()),
                            )
                            .to_owned(),
                    ),
                ),
        )
        .all(db)
        .await?;

    let record_ids: Vec<i64> = visible_records.iter().map(|r| r.id).collect();
    let record_status: BTreeMap<i64, String> = visible_records
        .iter()
        .map(|r| (r.id, r.status.clone()))
        .collect();

    let mut teacher_records: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    for record in &visible_records {
        if teacher_ids.contains(&record.teacher_id) {
            teacher_records
                .entry(record.teacher_id)
                .or_default()
                .insert(record.id);
        }
    }
    if !record_ids.is_empty() {
        for part in participant::Entity::find()
            .filter(participant::Column::AchievementId.is_in(record_ids.clone()))
            .filter(participant::Column::TeacherId.is_in(teacher_ids))
            .all(db)
            .await?
        {
            if let Some(tid) = part.teacher_id {
                teacher_records
                    .entry(tid)
                    .or_default()
                    .insert(part.achievement_id);
            }
        }
    }

    let mut files_by_record: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    if !record_ids.is_empty() {
        let links = achievement_file::Entity::find()
            .filter(achievement_file::Column::AchievementId.is_in(record_ids))
            .all(db)
            .await?;
        let candidate_file_ids: Vec<i64> = links.iter().map(|l| l.file_id).collect();
        let ready_file_ids: BTreeSet<i64> = if candidate_file_ids.is_empty() {
            BTreeSet::new()
        } else {
            file::Entity::find()
                .filter(file::Column::Id.is_in(candidate_file_ids))
                .filter(file::Column::UploadStatus.eq("ready"))
                .all(db)
                .await?
                .into_iter()
                .map(|f| f.id)
                .collect()
        };
        for link in links {
            if ready_file_ids.contains(&link.file_id) {
                files_by_record
                    .entry(link.achievement_id)
                    .or_default()
                    .push(link.file_id);
            }
        }
    }

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let account = users.get(&row.user_id).ok_or(ApiError::Internal)?;
        let department_name = dept_names
            .get(&row.department_id)
            .cloned()
            .ok_or(ApiError::Internal)?;
        let owned_set = teacher_records.remove(&row.id).unwrap_or_default();
        let achievement_count = u64::try_from(owned_set.len()).map_err(|_| ApiError::Internal)?;
        let approved_count = u64::try_from(
            owned_set
                .iter()
                .filter(|id| {
                    record_status
                        .get(id)
                        .is_some_and(|s| s == "approved" || s == "archived")
                })
                .count(),
        )
        .map_err(|_| ApiError::Internal)?;
        let mut distinct_files = BTreeSet::new();
        for ach_id in &owned_set {
            if let Some(fids) = files_by_record.get(ach_id) {
                for &fid in fids {
                    distinct_files.insert(fid);
                }
            }
        }
        let material_count = u64::try_from(distinct_files.len()).map_err(|_| ApiError::Internal)?;
        items.push(TeacherDirectoryItem {
            id: row.id,
            user_id: row.user_id,
            employee_no: row.employee_no,
            name: row.name,
            department_id: row.department_id,
            department_name,
            subject_code: row.subject_code,
            employment_status: row.employment_status,
            account_status: account.status.clone(),
            roles: roles_by_user.remove(&row.user_id).unwrap_or_default(),
            achievement_count,
            approved_count,
            material_count,
        });
    }

    Ok(TeacherDirectoryPage {
        departments,
        items,
        page,
        page_size: filter.page_size,
        total,
    })
}

pub async fn teacher_dossier(
    db: &impl ConnectionTrait,
    viewer: &CurrentUser,
    target_id: i64,
) -> Result<TeacherDossier, ApiError> {
    let row = teacher::Entity::find_by_id(target_id)
        .one(db)
        .await?
        .ok_or(ApiError::NotFound)?;
    let profile = teacher_profile(db, target_id).await?;
    let account = user::Entity::find_by_id(row.user_id)
        .one(db)
        .await?
        .ok_or(ApiError::Internal)?;
    let roles: Vec<String> = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(row.user_id))
        .order_by_asc(user_role::Column::RoleCode)
        .all(db)
        .await?
        .into_iter()
        .map(|r| r.role_code)
        .collect();

    let records = achievement::Entity::find()
        .filter(achievement::Column::Status.ne("draft"))
        .filter(
            Condition::any()
                .add(achievement::Column::TeacherId.eq(target_id))
                .add(
                    achievement::Column::Id.in_subquery(
                        Query::select()
                            .column(participant::Column::AchievementId)
                            .from(participant::Entity)
                            .and_where(Expr::col(participant::Column::TeacherId).eq(target_id))
                            .to_owned(),
                    ),
                ),
        )
        .order_by_desc(achievement::Column::UpdatedAt)
        .order_by_desc(achievement::Column::Id)
        .all(db)
        .await?;

    let mut counts = BTreeMap::new();
    for status in ["pending", "returned", "approved", "archived"] {
        counts.insert(status.to_string(), 0);
    }
    for record in &records {
        if let Some(entry) = counts.get_mut(&record.status) {
            *entry += 1;
        }
    }
    let achievement_total = u64::try_from(records.len()).map_err(|_| ApiError::Internal)?;
    let record_ids: Vec<i64> = records.iter().map(|r| r.id).collect();

    let mut achievements = Vec::new();
    for record in records.iter().take(100) {
        achievements.push(super::achievements::item(db, record).await?);
    }

    let mut files = Vec::new();
    if !record_ids.is_empty() {
        let file_ids: Vec<i64> = achievement_file::Entity::find()
            .filter(achievement_file::Column::AchievementId.is_in(record_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|l| l.file_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !file_ids.is_empty() {
            let file_rows = file::Entity::find()
                .filter(file::Column::Id.is_in(file_ids))
                .filter(file::Column::UploadStatus.eq("ready"))
                .order_by_desc(file::Column::CreatedAt)
                .order_by_desc(file::Column::Id)
                .all(db)
                .await?;
            for f in file_rows {
                let mut meta = super::achievements::metadata(f);
                meta.association =
                    Some(super::achievements::file_association(db, viewer, meta.id).await?);
                files.push(meta);
            }
        }
    }

    Ok(TeacherDossier {
        profile,
        account_status: account.status,
        roles,
        created_at: row.created_at,
        counts,
        achievements,
        achievement_total,
        files,
    })
}

pub async fn current_user(db: &impl ConnectionTrait, id: i64) -> Result<CurrentUser, ApiError> {
    let user = user::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if user.status != "active" {
        return Err(ApiError::Forbidden);
    }
    let teacher = teacher::Entity::find()
        .filter(teacher::Column::UserId.eq(id))
        .one(db)
        .await?
        .ok_or(ApiError::Forbidden)?;
    if teacher.employment_status != "active" {
        return Err(ApiError::Forbidden);
    }
    let teacher = teacher_profile(db, teacher.id).await?;
    let roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(id))
        .all(db)
        .await?;
    let mut grants = Vec::new();
    for role in &roles {
        let scope = DataScope::parse(&role.data_scope).ok_or(ApiError::Internal)?;
        let permissions = role_permission::Entity::find()
            .filter(role_permission::Column::RoleCode.eq(&role.role_code))
            .all(db)
            .await?;
        for permission in permissions {
            let grant = PermissionGrant {
                permission: permission.permission_code,
                scope: scope.clone(),
            };
            if !grants.contains(&grant) {
                grants.push(grant);
            }
        }
    }
    Ok(CurrentUser {
        id,
        display_name: user.display_name,
        teacher,
        roles: roles.into_iter().map(|role| role.role_code).collect(),
        grants,
    })
}

pub async fn find_identity(
    db: &impl ConnectionTrait,
    provider: &str,
    corp: &str,
    member: &str,
) -> Result<CurrentUser, ApiError> {
    let identity = identity::Entity::find()
        .filter(identity::Column::Provider.eq(provider))
        .filter(identity::Column::CorpId.eq(corp))
        .filter(identity::Column::ProviderUserid.eq(member))
        .one(db)
        .await?
        .ok_or(ApiError::Forbidden)?;
    current_user(db, identity.user_id).await
}
