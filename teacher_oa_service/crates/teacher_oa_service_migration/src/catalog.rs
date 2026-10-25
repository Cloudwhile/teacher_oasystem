//! Immutable P1 seed definitions. New published versions belong in a new migration.
use serde_json::{Value, json};

fn field(key: &str, label: &str, kind: &str, dictionary: Option<&str>) -> Value {
    let mut value = json!({"key":key,"label":label,"type":kind,"required":false,"visible":true});
    if let Some(dictionary) = dictionary {
        value["dictionary"] = dictionary.into();
    }
    if matches!(kind, "text" | "textarea") {
        value["maxLength"] = json!(if kind == "textarea" { 2000 } else { 500 });
    }
    value
}

fn text(key: &str, label: &str) -> Value {
    field(key, label, "text", None)
}
fn date(key: &str, label: &str) -> Value {
    field(key, label, "date", None)
}
fn select(key: &str, label: &str, dictionary: &str) -> Value {
    field(key, label, "select", Some(dictionary))
}

pub fn schemas() -> Vec<Value> {
    let definitions = [
        (
            "open_class",
            "公开课",
            "teaching",
            "名称",
            "时间",
            true,
            true,
            vec![
                text("venue", "地点"),
                text("organization", "组织机构"),
                select("evaluation", "评价", "evaluations"),
            ],
            vec![],
        ),
        (
            "special_lecture",
            "专题讲座",
            "teaching",
            "名称",
            "时间",
            true,
            true,
            vec![
                text("venue", "地点"),
                text("organization", "组织机构"),
                select("evaluation", "评价", "evaluations"),
            ],
            vec![],
        ),
        (
            "paper_award",
            "论文获奖",
            "research",
            "论文名称",
            "获奖时间",
            true,
            true,
            vec![
                text("organization", "落款单位"),
                select("author_type", "作者类型", "author_types"),
            ],
            vec!["author"],
        ),
        (
            "paper_publication",
            "论文发表",
            "research",
            "论文名称",
            "发表时间",
            true,
            false,
            vec![
                text("journal", "发表刊名"),
                text("isbn", "发表刊号 / ISBN"),
                select("author_type", "作者类型", "author_types"),
            ],
            vec!["author"],
        ),
        (
            "monograph",
            "专著",
            "research",
            "专著名称",
            "出版时间",
            true,
            false,
            vec![
                text("isbn", "出版刊号 / ISBN"),
                text("publisher", "出版社"),
                select("author_type", "作者类型", "author_types"),
            ],
            vec!["author"],
        ),
        (
            "research_project",
            "课题研究",
            "research",
            "课题名称",
            "立项时间",
            true,
            true,
            vec![
                date("completed_at", "结题时间"),
                text("organization", "立项单位"),
                text("project_number", "项目编号"),
                json!({"key":"project_funding","label":"项目经费（元）","type":"number","format":"decimal","required":false,"visible":true}),
            ],
            vec!["host", "participant"],
        ),
        (
            "teaching_result",
            "教学成果",
            "research",
            "成果名称",
            "获得时间",
            true,
            true,
            vec![],
            vec!["host", "participant"],
        ),
        (
            "patent",
            "专利发明",
            "research",
            "专利名称",
            "获得时间",
            true,
            false,
            vec![
                date("valid_until", "有效期至"),
                text("patent_number", "专利号"),
                text("organization", "颁发机构"),
            ],
            vec!["holder"],
        ),
        (
            "research_org_member",
            "教研机构成员",
            "profession",
            "机构名称",
            "开始时间",
            true,
            true,
            vec![
                date("ended_at", "结束时间"),
                field("duration_months", "担任时长（月）", "number", None),
                text("position", "担任职务"),
            ],
            vec![],
        ),
        (
            "master_studio",
            "名师工作室",
            "profession",
            "工作室名称",
            "开始时间",
            true,
            true,
            vec![
                date("ended_at", "结束时间"),
                field("duration_months", "担任时长（月）", "number", None),
                text("position", "担任职务"),
            ],
            vec![],
        ),
        (
            "backbone_teacher",
            "骨干教师",
            "profession",
            "名称",
            "获得时间",
            true,
            true,
            vec![text("batch", "批次"), text("organization", "组织机构")],
            vec![],
        ),
        (
            "subject_honor",
            "学科荣誉",
            "honor",
            "荣誉名称",
            "获奖时间",
            true,
            true,
            vec![
                select("award_grade_code", "奖项等级", "award_grades"),
                text("organization", "落款单位"),
            ],
            vec!["awardee"],
        ),
        (
            "personal_honor",
            "个人先进（优秀）",
            "honor",
            "荣誉名称",
            "获奖时间",
            false,
            true,
            vec![
                select("award_grade_code", "奖项等级", "award_grades"),
                text("organization", "落款单位"),
            ],
            vec!["awardee"],
        ),
        (
            "collective_honor",
            "集体荣誉",
            "honor",
            "荣誉名称",
            "获奖时间",
            true,
            true,
            vec![
                select("award_grade_code", "奖项等级", "award_grades"),
                text("organization", "落款单位"),
            ],
            vec!["awardee"],
        ),
        (
            "student_award",
            "指导学生获奖",
            "student",
            "奖项名称",
            "获奖时间",
            true,
            true,
            vec![
                select("award_grade_code", "奖项等级", "award_grades"),
                text("organization", "落款单位"),
            ],
            vec!["advisor", "student"],
        ),
    ];
    definitions.into_iter().map(|(code, name, category, title, occurred, subject, level, extra, roles)| {
        let mut title_field = text("title", title);
        title_field["required"] = true.into();
        let mut fields = vec![title_field];
        if subject { fields.push(select("subject_code", "学科", "subjects")); }
        fields.push(date("occurred_at", occurred));
        let term_label = match code {
            "research_project" => "立项学期",
            "paper_award" | "subject_honor" | "personal_honor" | "collective_honor" | "student_award" => "获奖学期",
            "paper_publication" => "发表学期",
            "monograph" => "出版学期",
            "teaching_result" | "patent" => "获得学期",
            _ => "学期",
        };
        fields.push(select("academic_term_id", term_label, "academic_terms"));
        if level {
            let level_label = match code {
                "research_project" => "立项级别",
                "paper_award" | "student_award" => "奖项级别",
                "teaching_result" => "成果等级",
                "research_org_member" => "机构级别",
                "master_studio" => "工作室级别",
                "subject_honor" | "personal_honor" | "collective_honor" => "荣誉级别",
                _ => "级别",
            };
            fields.push(select("level_code", level_label, "achievement_levels"));
        }
        fields.extend(extra);
        if !roles.is_empty() {
            let mut participants = field("participants", "相关人员与署名顺序", "participant_list", Some("participant_roles"));
            participants["roles"] = json!(roles);
            fields.push(participants);
        }
        fields.push(field("remarks", "备注", "textarea", None));
        let mut files = field("files", "证明材料", "file_list", None);
        files["required"] = true.into();
        fields.push(files);
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        for (index, item) in fields.iter_mut().enumerate() {
            item["order"] = json!(index + 1);
            let key = item["key"].as_str().unwrap_or_default();
            let mut property = match item["type"].as_str().unwrap_or_default() {
                "date" => json!({"type":"string","format":"date"}),
                "number" if item["format"] == "decimal" => json!({"type":"string","pattern":"^[0-9]{1,14}(\\.[0-9]{1,2})?$"}),
                "number" => json!({"type":"integer","minimum":0}),
                "file_list" => json!({"type":"array","minItems":1,"maxItems":20,"items":{"type":"integer"}}),
                "participant_list" => json!({"type":"array","maxItems":50,"items":{"type":"object","required":["external_name","role_code","sort_order"],"properties":{"teacher_id":{"type":["integer","null"]},"external_name":{"type":"string","maxLength":100},"role_code":{"type":"string"},"sort_order":{"type":"integer","minimum":0}},"additionalProperties":false}}),
                _ if key == "academic_term_id" => json!({"type":"integer"}),
                _ => json!({"type":"string"}),
            };
            if let Some(limit) = item.get("maxLength") { property["maxLength"] = limit.clone(); }
            if item["required"] == true { required.push(key.to_string()); }
            properties.insert(key.to_string(), property);
        }
        json!({"code":code,"name":name,"category":category,"version":1,"fields":fields,
            "json_schema":{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":properties,"required":required,"additionalProperties":false}})
    }).collect()
}

pub fn dictionaries() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    vec![
        (
            "subjects",
            vec![
                ("chinese", "语文"),
                ("math", "数学"),
                ("english", "英语"),
                ("information_technology", "信息技术"),
                ("physics", "物理"),
                ("chemistry", "化学"),
                ("biology", "生物"),
                ("history", "历史"),
                ("geography", "地理"),
                ("politics", "政治"),
                ("pe", "体育"),
                ("music", "音乐"),
                ("art", "美术"),
                ("education_management", "教育管理"),
                ("moral_education", "德育"),
                ("other", "其他"),
            ],
        ),
        (
            "achievement_levels",
            vec![
                ("national", "国家级"),
                ("province", "省级"),
                ("city", "市级"),
                ("district", "区级"),
                ("town", "镇街级"),
                ("school", "校级"),
            ],
        ),
        (
            "award_grades",
            vec![
                ("special", "特等奖"),
                ("first", "一等奖"),
                ("second", "二等奖"),
                ("third", "三等奖"),
                ("excellent", "优秀奖"),
            ],
        ),
        (
            "evaluations",
            vec![
                ("excellent", "优秀"),
                ("good", "良好"),
                ("qualified", "合格"),
            ],
        ),
        (
            "author_types",
            vec![
                ("sole", "唯一作者"),
                ("first", "第一作者"),
                ("second", "第二作者"),
                ("other", "其他"),
            ],
        ),
        (
            "participant_roles",
            vec![
                ("author", "作者"),
                ("host", "主持人"),
                ("participant", "参与人"),
                ("holder", "持有人"),
                ("awardee", "获奖人员"),
                ("advisor", "指导教师"),
                ("student", "学生"),
            ],
        ),
    ]
}
