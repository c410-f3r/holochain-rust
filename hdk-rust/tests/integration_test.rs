extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;
extern crate holochain_dna;
extern crate tempfile;
extern crate test_utils;

use holochain_core_api::*;

use holochain_core_types::{
    cas::content::AddressableContent,
    entry::{test_entry_a, Entry, SerializedEntry},
    entry_type::test_entry_type,
    hash::HashString,
    json::{JsonString, RawString},
};
use holochain_dna::zome::{
    capabilities::{Capability, FnDeclaration, Membrane},
    entry_types::EntryTypeDef,
};
use std::sync::{Arc, Mutex};
use test_utils::*;
use holochain_core_types::cas::content::Address;

pub fn create_test_cap_with_fn_names(fn_names: Vec<&str>) -> Capability {
    let mut capability = Capability::new();
    capability.cap_type.membrane = Membrane::Public;

    for fn_name in fn_names {
        let mut fn_decl = FnDeclaration::new();
        fn_decl.name = String::from(fn_name);
        capability.functions.push(fn_decl);
    }
    capability
}

fn start_holochain_instance() -> (Holochain, Arc<Mutex<TestLogger>>) {
    // Setup the holochain instance
    let wasm =
        create_wasm_from_file("wasm-test/target/wasm32-unknown-unknown/release/test_globals.wasm");
    let capabability = create_test_cap_with_fn_names(vec![
        "check_global",
        "check_commit_entry",
        "check_commit_entry_macro",
        "check_get_entry_result",
        "check_get_entry",
        "send_tweet",
        "commit_validation_package_tester",
        "link_two_entries",
        "links_roundtrip",
        "check_query",
        "check_hash_app_entry",
        "check_hash_sys_entry",
        "check_call",
        "check_call_with_args",
    ]);
    let mut dna = create_test_dna_with_cap("test_zome", "test_cap", &capabability, &wasm);

    dna.zomes.get_mut("test_zome").unwrap().entry_types.insert(
        String::from("validation_package_tester"),
        EntryTypeDef::new(),
    );

    let (context, test_logger) = test_context_and_logger("alex");
    let mut hc =
        Holochain::new(dna.clone(), context).expect("could not create new Holochain instance.");

    // Run the holochain instance
    hc.start().expect("couldn't start");
    (hc, test_logger)
}

#[test]
fn can_use_globals() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the debug API function for printing all GLOBALS
    let result = hc.call("test_zome", "test_cap", "check_global", r#"{}"#);
    assert_eq!(
        result.clone(),
        Ok(JsonString::from(HashString::from(
            "QmQw3V41bAWkQA9kwpNfU3ZDNzr9YW4p9RV4QHhFD3BkqA"
        ))),
        "result = {:?}",
        result
    );
}

#[test]
fn can_commit_entry() {
    let (mut hc, _) = start_holochain_instance();

    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry",
        &String::from(JsonString::from(SerializedEntry::from(test_entry_a()))),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(format!(
            "{{\"address\":\"{}\"}}",
            String::from(SerializedEntry::from(test_entry_a()).address())
        )),
    );
}

#[test]
fn can_commit_entry_macro() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        // this works because the macro names the args the same as the SerializedEntry fields
        &String::from(JsonString::from(SerializedEntry::from(test_entry_a()))),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(format!(
            "{{\"Ok\":\"{}\"}}",
            String::from(SerializedEntry::from(test_entry_a()).address())
        )),
    );
}

#[test]
fn can_round_trip() {
    let (mut hc, test_logger) = start_holochain_instance();
    let result = hc.call(
        "test_zome",
        "test_cap",
        "send_tweet",
        r#"{ "author": "bob", "content": "had a boring day" }"#,
    );
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"first\":\"bob\",\"second\":\"had a boring day\"}"),
    );

    let test_logger = test_logger.lock().unwrap();

    println!("{:?}", *test_logger);
}

#[test]
fn can_get_entry() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        &String::from(JsonString::from(SerializedEntry::from(test_entry_a()))),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(format!(
            "{{\"Ok\":\"{}\"}}",
            String::from(SerializedEntry::from(test_entry_a()).address())
        )),
    );

    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry_result",
        r#"{"entry_hash":"QmZi7c1G2qAN6Y5wxHDB9fLhSaSVBJe28ZVkiPraLEcvou"}"#,
    );
    println!("\t can_get_entry_result result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"stuff\":\"non fail\"}")
    );

    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry",
        &format!("{{\"entry_address\":\"{}\"}}", test_entry_a().address()),
    );
    println!("\t can_get_entry result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(
            "{\"Ok\":{\"value\":\"\\\"test entry value\\\"\",\"entry_type\":\"testEntryType\"}}"
        )
    );

    // test the case with a bad hash
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry_result",
        r#"{"entry_hash":"QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx"}"#,
    );
    println!("\t can_get_entry_result result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"got back no entry\":true}")
    );

    // test the case with a bad hash
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_get_entry",
        r#"{"entry_address":"QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx"}"#,
    );
    println!("\t can_get_entry result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from("{\"Ok\":null}"));
}

#[test]
fn can_invalidate_invalid_commit() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        &String::from(JsonString::from(SerializedEntry::from(Entry::new(
            &test_entry_type(),
            &JsonString::from(RawString::from("FAIL")),
        )))),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"Err\":\"Validation failed: FAIL content is not allowed\"}"),
    );
}

#[test]
fn has_populated_validation_data() {
    let (mut hc, _) = start_holochain_instance();

    //
    // Add two entries to chain to have something to check ValidationData on
    //
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\":\"non fail\"}" }"#,
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(r#"{"address":"QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg"}"#),
    );
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        r#"{ "entry_type": "testEntryType", "value": "{\"stuff\":\"non fail\"}" }"#,
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(r#"{"address":"QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg"}"#),
    );

    //
    // Expect the commit in this zome function to fail with a serialized ValidationData struct
    //
    let result = hc.call(
        "test_zome",
        "test_cap",
        "commit_validation_package_tester",
        r#"{}"#,
    );

    assert!(result.is_ok(), "\t result = {:?}", result);

    //
    // Deactivating this test for now since ordering of contents change non-deterministically
    //
    /*
    assert_eq!(
        JsonString::from("{\"Err\":{\"Internal\":\"{\\\"package\\\":{\\\"chain_header\\\":{\\\"entry_type\\\":{\\\"App\\\":\\\"validation_package_tester\\\"},\\\"entry_address\\\":\\\"QmYQPp1fExXdKfmcmYTbkw88HnCr3DzMSFUZ4ncEd9iGBY\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmSQqKHPpYZbafF7PXPKx31UwAbNAmPVuSHHxcBoDcYsci\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},\\\"source_chain_entries\\\":[{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"alex\\\",\\\"entry_type\\\":\\\"%agent_id\\\"}],\\\"source_chain_headers\\\":[{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"link_same_type\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRYerwRRXYxmYoxq1LTZMVVRfjNMAeqmdELTNDxURtHEZ\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":\\\"AgentId\\\",\\\"entry_address\\\":\\\"QmQw3V41bAWkQA9kwpNfU3ZDNzr9YW4p9RV4QHhFD3BkqA\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmQJxUSfJe2QoxTyEwKQX9ypbkcNv3cw1vasGTx1CUpJFm\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"}],\\\"custom\\\":null},\\\"sources\\\":[\\\"<insert your agent key here>\\\"],\\\"lifecycle\\\":\\\"Chain\\\",\\\"action\\\":\\\"Commit\\\"}\"}}"),
        result.unwrap(),
    );
    */}

#[test]
fn can_link_entries() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "link_two_entries", r#"{}"#);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(r#"{"Ok":null}"#));
}

#[test]
fn can_roundtrip_links() {
    let (mut hc, _) = start_holochain_instance();
    let result = hc.call("test_zome", "test_cap", "links_roundtrip", r#"{}"#);
    assert!(result.is_ok(), "\t result = {:?}", result);
    let result_string = result.unwrap();
    let ordering1: bool = result_string == JsonString::from(r#"{"links":["QmStYP5FYC61PfKKMYZpqBSMRJCAUeuSS8Vuz4EQL5uvK2","QmW6vfGv7fWMPQsgwd63HJhtoZmHTrf9MSNXCkG6LZxyog"]}"#);
    let ordering2: bool = result_string == JsonString::from(r#"{"links":["QmW6vfGv7fWMPQsgwd63HJhtoZmHTrf9MSNXCkG6LZxyog","QmStYP5FYC61PfKKMYZpqBSMRJCAUeuSS8Vuz4EQL5uvK2"]}"#);
    assert!(ordering1 || ordering2);
}

#[test]
fn can_check_query() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_query",
        r#"{ "entry_type_name": "testEntryType", "limit": "0" }"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(r#"["QmStYP5FYC61PfKKMYZpqBSMRJCAUeuSS8Vuz4EQL5uvK2"]"#),
    );
}

#[test]
fn can_check_hash_app_entry() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "check_hash_app_entry", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from("QmbagHKV6kU89Z4FzQGMHpCYMxpR8WPxnse6KMArQ2wPJa")),
    );
}

#[test]
fn can_check_hash_sys_entry() {
    let (mut hc, _) = start_holochain_instance();

    let _result = hc.call("test_zome", "test_cap", "check_hash_sys_entry", r#"{}"#);
    // TODO
    //    assert!(result.is_ok(), "result = {:?}", result);
    //    assert_eq!(
    //        result.unwrap(),
    //        r#"{"result":"QmYmZyvDda3ygMhNnEjx8p9Q1TonHG9xhpn9drCptRT966"}"#,
    //    );
}

#[test]
fn can_check_call() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "check_call", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from("QmbagHKV6kU89Z4FzQGMHpCYMxpR8WPxnse6KMArQ2wPJa")),
    );
}

#[test]
fn can_check_call_with_args() {
    let (mut hc, _) = start_holochain_instance();

    let result = hc.call("test_zome", "test_cap", "check_call_with_args", r#"{}"#);
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(Address::from("QmSxw5mUkFfc2W95GK2xaNYRp4a8ZXxY8o7mPMDJv9pvJg")),
    );
}
