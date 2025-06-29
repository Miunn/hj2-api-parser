use axum::{body::Bytes, extract::Multipart, http::StatusCode, Json};
use serde::{Serialize};
use log::{info, warn};

use crate::utils::parse_xmlhotelleriejobs;

#[derive(Serialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub city: String,
    pub postal_code: String,
    pub logo_url: String,
}

#[derive(Serialize)]
pub struct Translation {
    pub language: String,
    pub title: String,
    pub description: String,
    pub requirements: String,
}

#[derive(Serialize)]
pub struct Job {
    pub id: String,
    pub schedule: String,
    pub category: String,
    pub city: String,
    pub province: String,
    pub application_method: String,
    pub application_destination: String,
    pub company: Company,
    pub translations: Vec<Translation>,
}

#[derive(Serialize, Debug)]
pub struct XMLError {
    pub line: i32,
    pub column: i32,
    pub message: String,
    pub level: String,
    pub domain: String,
    pub code: i32,
}

#[derive(Serialize)]
pub struct ImportResponse {
    success: bool,
    errors: String,
    xml_errors: Vec<XMLError>,
    jobs: Vec<Job>,
}

pub async fn handler(multipart: Multipart) -> (StatusCode, Json<ImportResponse>) {
    let (format, file) = read_multipart(multipart).await;


    if format.is_none() || file.is_none() {
        warn!(target: "import", "Request to import, format or file is missing");
        return (StatusCode::BAD_REQUEST, Json(ImportResponse {
            success: false,
            errors: "Format or file is missing".to_string(),
            xml_errors: vec![],
            jobs: vec![],
        }));
    }

    info!(target: "import", "Request to parse an {:?} file", format.clone().unwrap());

    match format.unwrap().as_str() {
        "xml-hotelleriejobs" => {
            let jobs = parse_xmlhotelleriejobs::parse(file.as_ref().unwrap());
            if let Err(errors) = jobs {
                warn!(target: "import", "Error parsing file: {:?}", errors.message);
                return (StatusCode::BAD_REQUEST, Json(ImportResponse {
                    success: false,
                    errors: errors.message,
                    xml_errors: errors.xml_errors,
                    jobs: vec![],
                }));
            }

            info!(target: "import", "File parsed successfully");

            return (StatusCode::OK, Json(ImportResponse {
                success: true,
                errors: "".to_string(),
                xml_errors: vec![],
                jobs: jobs.unwrap(),
            }));
        }
        _ => {
            warn!(target: "import", "Format is not supported");
            return (StatusCode::BAD_REQUEST, Json(ImportResponse {
                success: false,
                errors: "Format is not supported".to_string(),
                xml_errors: vec![],
                jobs: vec![],
            }));
        }
    }
}

async fn read_multipart(mut multipart: Multipart) -> (Option<String>, Option<Bytes>) {
    let mut format: Option<String> = None;
    let mut file: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        if name == "format" {
            format = Some(field.text().await.unwrap());
        } else if name == "file" {
            file = Some(field.bytes().await.unwrap());
        }
    }

    (format, file)
}