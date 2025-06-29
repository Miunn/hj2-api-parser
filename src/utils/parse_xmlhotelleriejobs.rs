use std::collections::HashMap;

use axum::body::Bytes;
use libxml::{parser, schemas, tree::Document};

use crate::api::import::{Company, Job, Translation, XMLError};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub xml_errors: Vec<XMLError>,
}

pub fn parse(file: &Bytes) -> Result<Vec<Job>, ParseError> {
    let document = validate_against_xsd(file);
    if let Err(errors) = document {
        return Err(ParseError {
            message: "File is not valid".to_string(),
            xml_errors: errors,
        });
    }

    match parse_into_jobs(&document.unwrap()) {
        Ok(jobs) => {
            return Ok(jobs);
        },
        Err(e) => {
            return Err(ParseError {
                message: e,
                xml_errors: vec![],
            });
        },
    }
}

fn validate_against_xsd(file: &Bytes) -> Result<Document, Vec<XMLError>> {
    let mut schema_parser = schemas::SchemaParserContext::from_file("xsd-schemas/xml-hotelleriejobs.xsd");
    let mut schema_validation = schemas::SchemaValidationContext::from_parser(&mut schema_parser).unwrap();
    
    let parser = parser::Parser::default();
    let document = parser.parse_string(file).unwrap();

    match schema_validation.validate_document(&document) {
        Ok(_) => Ok(document),
        Err(e) => {
            println!("Error: {:?}", e);

            let errors = e.iter().map(|e| XMLError {
                line: e.line.unwrap_or(0),
                column: e.col.unwrap_or(0),
                message: e.message.clone().unwrap_or_default(),
                level: format!("{:?}", e.level),
                domain: e.domain.to_string(),
                code: e.code,
            }).collect();

            Err(errors)
        },
    }
}

fn parse_into_jobs(document: &Document) -> Result<Vec<Job>, String> {
    let root = document.get_root_element().unwrap();
    let children = root.get_child_elements();
    let jobs = children.iter().map(|job| {
        // Populate a dictionary, keyed by nodes names 
        // This will be used to populate the Job struct with unique fields only as the repeated fields are overwritten
        let mut dictionary = HashMap::new();
        let mut child = job.get_first_child();
        while let Some(current_child) = child {
            dictionary.insert(current_child.get_name().to_string(), current_child.get_content().clone().to_string());
            child = current_child.get_next_sibling();
        }

        let titles = job.findnodes("title").unwrap();
        let descriptions = job.findnodes("description").unwrap();
        let requirements = job.findnodes("requirements").unwrap();

        // Build translations by language
        let mut translations: Vec<Translation> = Vec::new();
        
        for title_node in titles {
            let lang = title_node.get_attribute("lang").unwrap_or("en".to_string());
            let title_content = title_node.get_content().to_string();
            
            // Find existing translation with this language or create new one
            let translation = translations.iter_mut().find(|t| t.language == lang.to_string());
            if let Some(existing_translation) = translation {
                existing_translation.title = title_content;
            } else {
                translations.push(Translation {
                    language: lang,
                    title: title_content,
                    description: String::new(),
                    requirements: String::new(),
                });
            }
        }
        
        for desc_node in descriptions {
            let lang = desc_node.get_attribute("lang").unwrap_or("en".to_string());
            let desc_content = desc_node.get_content().to_string();
            
            // Find existing translation with this language or create new one
            let translation = translations.iter_mut().find(|t| t.language == lang);
            if let Some(existing_translation) = translation {
                existing_translation.description = desc_content;
            } else {
                translations.push(Translation {
                    language: lang,
                    title: String::new(),
                    description: desc_content,
                    requirements: String::new(),
                });
            }
        }
        
        for req_node in requirements {
            let lang = req_node.get_attribute("lang").unwrap_or("en".to_string());
            let req_content = req_node.get_content().to_string();
            
            // Find existing translation with this language or create new one
            let translation = translations.iter_mut().find(|t| t.language == lang);
            if let Some(existing_translation) = translation {
                existing_translation.requirements = req_content;
            } else {
                translations.push(Translation {
                    language: lang,
                    title: String::new(),
                    description: String::new(),
                    requirements: req_content,
                });
            }
        }

        Job {
            id: dictionary.get("unique_id").unwrap().to_string(),
            schedule: dictionary.get("schedule").unwrap().to_string(),
            category: dictionary.get("category").unwrap().to_string(),
            city: dictionary.get("city").unwrap().to_string(),
            province: dictionary.get("province").unwrap().to_string(),
            application_method: dictionary.get("application_method").unwrap().to_string(),
            application_destination: dictionary.get("application_destination").unwrap().to_string(),
            company: Company {
                id: dictionary.get("company_id").unwrap().to_string(),
                name: dictionary.get("company").unwrap().to_string(),
                city: dictionary.get("company_city").unwrap().to_string(),
                postal_code: dictionary.get("company_postal_code").unwrap().to_string(),
                logo_url: dictionary.get("company_logo_url").unwrap().to_string(),
            },
            translations: translations,
        }
    }).collect();

    Ok(jobs)
}