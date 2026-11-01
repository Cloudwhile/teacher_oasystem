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
/// lopdf's writer drops object streams, so this PDF is assembled by hand.
fn with_object_stream(decoded_len: usize) -> Vec<u8> {
    let mut stream = Stream::new(
        dictionary! {"Type"=>"ObjStm","N"=>1,"First"=>5},
        [b"5 0 null".as_slice(), &vec![b' '; decoded_len]].concat(),
    );
    stream.compress().expect("compress object stream");
    let mut out = b"%PDF-1.5\n".to_vec();
    let mut offsets = Vec::new();
    for body in [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 20 20] >>".to_vec(),
    ] {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n", offsets.len()).bytes());
        out.extend(body);
        out.extend(b"\nendobj\n");
    }
    offsets.push(out.len());
    out.extend(
        format!(
            "4 0 obj\n<< /Type /ObjStm /N 1 /First 5 /Filter /FlateDecode /Length {} >>\nstream\n",
            stream.content.len()
        )
        .bytes(),
    );
    out.extend(&stream.content);
    out.extend(b"\nendstream\nendobj\n");
    let xref = out.len();
    out.extend(b"xref\n0 5\n0000000000 65535 f \n");
    for offset in offsets {
        out.extend(format!("{offset:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").bytes());
    out
}
#[test]
fn compressed_object_stream_bombs_are_rejected_without_full_inflation() {
    let small = with_object_stream(1024);
    assert_eq!(
        inspect("证明.pdf", "application/pdf", &small).expect("small object stream"),
        "application/pdf"
    );
    let bomb = with_object_stream(256 * 1024 * 1024);
    assert!(bomb.len() < 1024 * 1024, "bomb stays a small upload");
    assert!(matches!(
        inspect("证明.pdf", "application/pdf", &bomb),
        Err(teacher_oa_service::api::error::ApiError::Invalid(
            "PDF压缩内容过大"
        ))
    ));
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
