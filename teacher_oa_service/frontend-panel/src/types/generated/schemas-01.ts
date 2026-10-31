/** Generated from the Rust OpenAPI contract. Do not edit. */
import type { components } from "../api.generated";
export interface SchemasPart1 {
    AcademicTerm: {
        academic_year: string;
        enabled: boolean;
        /** Format: date */
        ends_on: string;
        /** Format: int64 */
        id: number;
        name: string;
        /** Format: date */
        starts_on: string;
        /** Format: int32 */
        term_number: number;
    };
    AcademicTerms: components["schemas"]["AcademicTerm"][];
    AccountInput: {
        active: boolean;
        department: string;
        /** Format: int64 */
        id?: number | null;
        name: string;
        password?: string | null;
        role: string;
        username: string;
        wecom_userid?: string | null;
    };
    AccountItem: {
        active: boolean;
        department: string;
        /** Format: int64 */
        id: number;
        must_change: boolean;
        name: string;
        role: string;
        username: string;
        wecom_userid?: string | null;
    };
    Accounts: {
        items: components["schemas"]["AccountItem"][];
    };
    AchievementDetail: {
        can_archive: boolean;
        can_edit: boolean;
        can_review: boolean;
        data: components["schemas"]["AchievementInput"];
        events: components["schemas"]["ReviewEvent"][];
        files: components["schemas"]["FileMetadata"][];
        record: components["schemas"]["AchievementItem"];
        schema: components["schemas"]["AchievementSchema"];
    };
    AchievementInput: {
        /** Format: int64 */
        academic_term_id?: number | null;
        attributes?: {
            [key: string]: unknown;
        };
        award_grade_code?: string | null;
        file_ids?: number[];
        level_code?: string | null;
        /** Format: date */
        occurred_at?: string | null;
        participants?: components["schemas"]["ParticipantInput"][];
        remarks?: string;
        /** Format: int32 */
        schema_version: number;
        subject_code?: string | null;
        term_mismatch_confirmed?: boolean;
        title: string;
        type_code: string;
    };
    AchievementItem: {
        /** Format: int64 */
        academic_term_id?: number | null;
        award_grade_code?: string | null;
        /** Format: date-time */
        created_at: string;
        /** Format: int64 */
        id: number;
        level_code?: string | null;
        /** Format: date */
        occurred_at?: string | null;
        status: string;
        subject_code?: string | null;
        /** Format: int64 */
        teacher_id: number;
        teacher_name: string;
        title: string;
        type_code: string;
        type_name: string;
        /** Format: date-time */
        updated_at: string;
        /** Format: int32 */
        version: number;
    };
    AchievementPage: {
        items: components["schemas"]["AchievementItem"][];
        /** Format: int64 */
        page: number;
        /** Format: int64 */
        page_size: number;
        /** Format: int64 */
        total: number;
    };
    AchievementRevisions: components["schemas"]["RevisionItem"][];
    AchievementSchema: {
        category: string;
        code: string;
        fields: components["schemas"]["FieldDefinition"][];
        json_schema: unknown;
        name: string;
        /** Format: int32 */
        version: number;
    };
    AchievementSnapshot: {
        data: components["schemas"]["AchievementInput"];
        status: string;
    };
    AchievementType: {
        category: string;
        code: string;
        /** Format: int32 */
        current_version: number;
        name: string;
    };
    AchievementTypes: components["schemas"]["AchievementType"][];
    /**
     * @description Product codes; Forge only checks the success/failure classification.
     * @enum {string}
     */
    ApiErrorCode: "success" | "endpoint_not_found" | "unauthorized" | "forbidden" | "invalid_request" | "unavailable" | "internal_error" | "validation_failed" | "conflict" | "payload_too_large";
    ApiErrorInfo_TupleUnit: {
        diagnostic?: null;
        retryable: boolean;
    };
    ApiErrorInfo_ValidationDetails: {
        diagnostic?: components["schemas"]["ValidationDetails"];
        retryable: boolean;
    };
    ApiResponse_AcademicTerms_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: components["schemas"]["AcademicTerm"][];
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_Accounts_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            items: components["schemas"]["AccountItem"][];
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_AchievementDetail_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            can_archive: boolean;
            can_edit: boolean;
            can_review: boolean;
            data: components["schemas"]["AchievementInput"];
            events: components["schemas"]["ReviewEvent"][];
            files: components["schemas"]["FileMetadata"][];
            record: components["schemas"]["AchievementItem"];
            schema: components["schemas"]["AchievementSchema"];
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_AchievementPage_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            items: components["schemas"]["AchievementItem"][];
            /** Format: int64 */
            page: number;
            /** Format: int64 */
            page_size: number;
            /** Format: int64 */
            total: number;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_AchievementRevisions_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: components["schemas"]["RevisionItem"][];
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_AchievementSchema_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            category: string;
            code: string;
            fields: components["schemas"]["FieldDefinition"][];
            json_schema: unknown;
            name: string;
            /** Format: int32 */
            version: number;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_AchievementTypes_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: components["schemas"]["AchievementType"][];
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_AuthResponse_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            csrf_token: string;
            must_change_password: boolean;
            user: components["schemas"]["CurrentUser"];
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_Dashboard_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            counts: {
                [key: string]: number;
            };
            recent: components["schemas"]["AchievementItem"][];
            /** Format: int64 */
            review_pending: number;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_DictionaryItems_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: components["schemas"]["DictionaryItem"][];
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_FileMetadata_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            association?: components["schemas"]["FileAssociation"] | null;
            /** Format: date-time */
            created_at: string;
            filename: string;
            /** Format: int64 */
            id: number;
            mime_type: string;
            sha256: string;
            /** Format: int64 */
            size_bytes: number;
            upload_status: string;
            /** Format: int64 */
            uploaded_by: number;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_FilePage_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            items: components["schemas"]["FileMetadata"][];
            /** Format: int64 */
            page: number;
            /** Format: int64 */
            page_size: number;
            /** Format: int64 */
            total: number;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_LoginOptions_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            mock_enabled: boolean;
            password_enabled: boolean;
            wecom_enabled: boolean;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_LoginPageAsset_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            url: string;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_LoginPageSettings_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
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
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_PermissionGrants_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: components["schemas"]["PermissionGrant"][];
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_TeacherOptions_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: components["schemas"]["TeacherOption"][];
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_TeacherProfile_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
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
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_TupleUnit_ApiErrorCode_TupleUnit: {
        code: components["schemas"]["ApiErrorCode"];
        data?: null;
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    ApiResponse_TupleUnit_ApiErrorCode_ValidationDetails: {
        code: components["schemas"]["ApiErrorCode"];
        data?: null;
        error?: components["schemas"]["ApiErrorInfo_ValidationDetails"];
        msg: string;
    };
    ApiResponse_WecomSettings_ApiErrorCode: {
        code: components["schemas"]["ApiErrorCode"];
        data?: {
            agent_id: string;
            callback_url: string;
            corp_id: string;
            enabled: boolean;
            secret_configured: boolean;
            /** Format: int32 */
            version: number;
        };
        error?: components["schemas"]["ApiErrorInfo_TupleUnit"];
        msg: string;
    };
    AuthResponse: {
        csrf_token: string;
        must_change_password: boolean;
        user: components["schemas"]["CurrentUser"];
    };
    CurrentUser: {
        display_name: string;
        grants: components["schemas"]["PermissionGrant"][];
        /** Format: int64 */
        id: number;
        roles: string[];
        teacher: components["schemas"]["TeacherProfile"];
    };
    Dashboard: {
        counts: {
            [key: string]: number;
        };
        recent: components["schemas"]["AchievementItem"][];
        /** Format: int64 */
        review_pending: number;
    };
    /** @enum {string} */
    DataScope: "SELF" | "DEPARTMENT" | "ALL";
    DictionaryItem: {
        code: string;
        group_code: string;
        name: string;
        /** Format: int32 */
        sort_order: number;
    };
    DictionaryItems: components["schemas"]["DictionaryItem"][];
    /** @description Product error adapter: injects the code/message and retains the stable ErrorResponse schema. */
    ErrorResponse: components["schemas"]["ApiResponse_TupleUnit_ApiErrorCode_TupleUnit"];
    Failure: components["schemas"]["ApiResponse_TupleUnit_ApiErrorCode_ValidationDetails"] & {
        request_id: string;
    };
    FieldDefinition: {
        dictionary?: string | null;
        format?: string | null;
        help?: string;
        key: string;
        label: string;
        maxLength?: number | null;
        options?: string[];
        /** Format: int32 */
        order: number;
        required: boolean;
        roles?: string[];
        type: components["schemas"]["FieldType"];
        visible: boolean;
    };
    /** @enum {string} */
    FieldType: "text" | "textarea" | "number" | "date" | "select" | "multi_select" | "participant_list" | "file_list";
    /** @enum {string} */
    FileAssociation: "linked" | "historical" | "unlinked";
    FileMetadata: {
        association?: components["schemas"]["FileAssociation"] | null;
        /** Format: date-time */
        created_at: string;
        filename: string;
        /** Format: int64 */
        id: number;
        mime_type: string;
        sha256: string;
        /** Format: int64 */
        size_bytes: number;
        upload_status: string;
        /** Format: int64 */
        uploaded_by: number;
    };
    FilePage: {
        items: components["schemas"]["FileMetadata"][];
        /** Format: int64 */
        page: number;
        /** Format: int64 */
        page_size: number;
        /** Format: int64 */
        total: number;
    };
}
