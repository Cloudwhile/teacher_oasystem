use lopdf::{Document, Object, Stream, dictionary};
use teacher_oa_service::services::file_service::inspect;

fn pdf(active: bool) -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages = doc.new_object_id();
    let stream = doc.add_object(Stream::new(dictionary! {}, Vec::new()));
    let page=doc.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"Contents"=>stream,"MediaBox"=>vec![0.into(),0.into(),20.into(),20.into()]});
    doc.objects.insert(
        pages,
        Object::Dictionary(dictionary! {"Type"=>"Pages","Kids"=>vec![page.into()],"Count"=>1}),
    );
    let mut catalog = dictionary! {"Type"=>"Catalog","Pages"=>pages};
    if active {
        catalog.set(
            "OpenAction",
            dictionary! {"S"=>"JavaScript","JS"=>Object::string_literal("alert(1)")},
        );
    }
    let root = doc.add_object(catalog);
    doc.trailer.set("Root", root);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).expect("save PDF");
    bytes
}
#[test]
fn parses_pdf_content_and_rejects_active_or_mislabeled_documents() {
    assert_eq!(
        inspect("证明.pdf", "application/pdf", &pdf(false)).expect("valid PDF"),
        "application/pdf"
    );
    assert!(inspect("证明.pdf", "application/pdf", &pdf(true)).is_err());
    assert!(inspect("证明.png", "image/png", &pdf(false)).is_err());
    assert!(inspect("证明.pdf", "application/pdf", b"%PDF-1.5 invalid").is_err());
}
