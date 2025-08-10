use mdns_responder_rs as mdns;
use std::thread;
use std::time::Duration;

#[test]
fn test_responder_creation_and_shutdown() {
    let responder = mdns::Responder::new().expect("Failed to create responder");
    
    thread::sleep(Duration::from_millis(100));
    
    drop(responder);
}

#[test] 
fn test_service_registration() {
    let responder = mdns::Responder::new().expect("Failed to create responder");
    
    let _service = responder.register(
        "_test._tcp".to_owned(),
        "Test Service".to_owned(), 
        12345,
        &["version=1.0", "path=/test"],
    );
    
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_multiple_service_registration() {
    let responder = mdns::Responder::new().expect("Failed to create responder");
    
    let _service1 = responder.register(
        "_http._tcp".to_owned(),
        "Web Server 1".to_owned(),
        8080,
        &["path=/api"],
    );
    
    let _service2 = responder.register(
        "_http._tcp".to_owned(),
        "Web Server 2".to_owned(),
        8081,
        &["path=/admin"],
    );
    
    let _service3 = responder.register(
        "_ssh._tcp".to_owned(),
        "SSH Server".to_owned(),
        22,
        &[],
    );
    
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_service_unregistration() {
    let responder = mdns::Responder::new().expect("Failed to create responder");
    
    let service = responder.register(
        "_test._tcp".to_owned(),
        "Temporary Service".to_owned(),
        9999,
        &["temp=true"],
    );
    
    thread::sleep(Duration::from_millis(50));
    
    drop(service);
    
    thread::sleep(Duration::from_millis(50));
}

#[test]
fn test_empty_txt_records() {
    let responder = mdns::Responder::new().expect("Failed to create responder");
    
    let _service = responder.register(
        "_empty._tcp".to_owned(),
        "No TXT Records".to_owned(),
        7777,
        &[],
    );
    
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_service_with_special_characters() {
    let responder = mdns::Responder::new().expect("Failed to create responder");
    
    let _service = responder.register(
        "_special._tcp".to_owned(),
        "Special-Service_123".to_owned(),
        5555,
        &["key=value with spaces", "url=http://example.com/path?query=1"],
    );
    
    thread::sleep(Duration::from_millis(100));
}