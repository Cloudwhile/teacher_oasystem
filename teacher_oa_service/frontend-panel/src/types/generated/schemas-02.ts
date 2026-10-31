/** Generated from the Rust OpenAPI contract. Do not edit. */
import type { components } from "../api.generated";
export interface SchemasPart2 {
    LoginAnnouncement: {
        content: string;
        id: string;
        published: boolean;
        published_at?: string | null;
        title: string;
    };
    LoginOptions: {
        mock_enabled: boolean;
        password_enabled: boolean;
        wecom_enabled: boolean;
    };
    LoginPageAsset: {
        url: string;
    };
    LoginPageSettings: {
        /** @default [] */
        announcements: components["schemas"]["LoginAnnouncement"][];
        /** @default  */
        background_url: string;
        /** @default TeacherOA · 教师工作空间 */
        footer: string;
        /** @default  */
        logo_url: string;
        /** @default TeacherOA · 教师综合 OA */
        title: string;
        /**
         * Format: int32
         * @default 0
         */
        version: number;
    };
    MockLogin: {
        identity: string;
    };
    ParticipantInput: {
        external_name: string;
        role_code: string;
        /** Format: int32 */
        sort_order: number;
        /** Format: int64 */
        teacher_id?: number | null;
    };
    PasswordChange: {
        current_password: string;
        new_password: string;
    };
    PasswordLogin: {
        password: string;
        username: string;
    };
    PermissionGrant: {
        permission: string;
        scope: components["schemas"]["DataScope"];
    };
    PermissionGrants: components["schemas"]["PermissionGrant"][];
    ReviewEvent: {
        action: string;
        /** Format: date-time */
        created_at: string;
        /** Format: int64 */
        id: number;
        new_status: string;
        old_status: string;
        opinion: string;
        /** Format: int64 */
        reviewer_id: number;
        reviewer_name: string;
    };
    ReviewRequest: {
        opinion?: string;
        /** Format: int32 */
        version: number;
    };
    RevisionItem: {
        /** Format: date-time */
        created_at: string;
        files: components["schemas"]["FileMetadata"][];
        /** Format: int64 */
        modified_by: number;
        /** Format: int32 */
        schema_version: number;
        snapshot: components["schemas"]["AchievementSnapshot"];
        /** Format: int32 */
        version: number;
    };
    SaveWecomSettings: {
        agent_id: string;
        corp_id: string;
        enabled: boolean;
        secret?: string | null;
        /** Format: int32 */
        version: number;
    };
    /** @description Basic status response returned by the generated skeleton. */
    StatusResponse: {
        /** @description Cargo package name. */
        service: string;
        /** @description Public health or readiness status. */
        status: string;
    };
    Success_AcademicTerms: components["schemas"]["ApiResponse_AcademicTerms_ApiErrorCode"] & {
        request_id: string;
    };
    Success_Accounts: components["schemas"]["ApiResponse_Accounts_ApiErrorCode"] & {
        request_id: string;
    };
    Success_AchievementDetail: components["schemas"]["ApiResponse_AchievementDetail_ApiErrorCode"] & {
        request_id: string;
    };
    Success_AchievementPage: components["schemas"]["ApiResponse_AchievementPage_ApiErrorCode"] & {
        request_id: string;
    };
    Success_AchievementRevisions: components["schemas"]["ApiResponse_AchievementRevisions_ApiErrorCode"] & {
        request_id: string;
    };
    Success_AchievementSchema: components["schemas"]["ApiResponse_AchievementSchema_ApiErrorCode"] & {
        request_id: string;
    };
    Success_AchievementTypes: components["schemas"]["ApiResponse_AchievementTypes_ApiErrorCode"] & {
        request_id: string;
    };
    Success_AuthResponse: components["schemas"]["ApiResponse_AuthResponse_ApiErrorCode"] & {
        request_id: string;
    };
    Success_Dashboard: components["schemas"]["ApiResponse_Dashboard_ApiErrorCode"] & {
        request_id: string;
    };
    Success_DictionaryItems: components["schemas"]["ApiResponse_DictionaryItems_ApiErrorCode"] & {
        request_id: string;
    };
    Success_FileMetadata: components["schemas"]["ApiResponse_FileMetadata_ApiErrorCode"] & {
        request_id: string;
    };
    Success_FilePage: components["schemas"]["ApiResponse_FilePage_ApiErrorCode"] & {
        request_id: string;
    };
    Success_LoginOptions: components["schemas"]["ApiResponse_LoginOptions_ApiErrorCode"] & {
        request_id: string;
    };
    Success_LoginPageAsset: components["schemas"]["ApiResponse_LoginPageAsset_ApiErrorCode"] & {
        request_id: string;
    };
    Success_LoginPageSettings: components["schemas"]["ApiResponse_LoginPageSettings_ApiErrorCode"] & {
        request_id: string;
    };
    Success_PermissionGrants: components["schemas"]["ApiResponse_PermissionGrants_ApiErrorCode"] & {
        request_id: string;
    };
    Success_TeacherOptions: components["schemas"]["ApiResponse_TeacherOptions_ApiErrorCode"] & {
        request_id: string;
    };
    Success_TeacherProfile: components["schemas"]["ApiResponse_TeacherProfile_ApiErrorCode"] & {
        request_id: string;
    };
    Success_WecomSettings: components["schemas"]["ApiResponse_WecomSettings_ApiErrorCode"] & {
        request_id: string;
    };
    TeacherOption: {
        /** Format: int64 */
        department_id: number;
        /** Format: int64 */
        id: number;
        name: string;
    };
    TeacherOptions: components["schemas"]["TeacherOption"][];
    TeacherProfile: {
        /** Format: int64 */
        department_id: number;
        department_name: string;
        employee_no: string;
        employment_status: string;
        /** Format: int64 */
        id: number;
        name: string;
        subject_code: string;
        /** Format: int64 */
        user_id: number;
    };
    UpdateAchievement: {
        data: components["schemas"]["AchievementInput"];
        /** Format: int32 */
        version: number;
    };
    UploadBody: {
        /** Format: binary */
        file: string;
    };
    ValidationDetails: {
        fields: {
            [key: string]: string;
        };
    };
    VersionRequest: {
        /** Format: int32 */
        version: number;
    };
    WecomSettings: {
        agent_id: string;
        callback_url: string;
        corp_id: string;
        enabled: boolean;
        secret_configured: boolean;
        /** Format: int32 */
        version: number;
    };
}
