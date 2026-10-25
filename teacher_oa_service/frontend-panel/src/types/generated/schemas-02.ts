/** Generated from the Rust OpenAPI contract. Do not edit. */
import type { components } from "../api.generated";
export interface SchemasPart2 {
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
}
