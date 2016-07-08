extern crate kvtagger_rs_parser;
extern crate syslog_ng_common;

use kvtagger_rs_parser::KVTaggerBuilder;
use kvtagger_rs_parser::utils::make_expected_value_for_test_file;

use syslog_ng_common::{ParserBuilder, GlobalConfig, SYSLOG_NG_INITIALIZED, syslog_ng_global_init};
use syslog_ng_common::mock::MockPipe;

#[test]
fn test_csv_records_can_be_read_from_file() {
    let expected = make_expected_value_for_test_file();
    let records = KVTaggerBuilder::<MockPipe>::load_database("tests/test.csv").ok().unwrap();
    assert_eq!(&records, &expected);
}

#[test]
fn test_unparseable_csv_file_is_reported_as_an_error() {
    let _ = KVTaggerBuilder::<MockPipe>::load_database("tests/unparseable.csv").err().unwrap();
}

#[test]
fn test_csv_file_with_more_than_three_columns_is_reported_as_an_error() {
    let _ = KVTaggerBuilder::<MockPipe>::load_database("tests/more_columns.csv").err().unwrap();
}

#[test]
fn test_csv_file_is_read_in_set_csv_file() {
    SYSLOG_NG_INITIALIZED.call_once(|| {
        unsafe { syslog_ng_global_init() };
    });

    let cfg = GlobalConfig::new(0x0308);
    let mut builder = KVTaggerBuilder::<MockPipe>::new(cfg);
    let expected = make_expected_value_for_test_file();

    builder.set_database("tests/test.csv").ok().unwrap();

    assert_eq!(builder.records(), Some(&expected));
}

#[test]
fn test_non_exisint_csv_file_does_not_cause_panic() {
    SYSLOG_NG_INITIALIZED.call_once(|| {
        unsafe { syslog_ng_global_init() };
    });

    let cfg = GlobalConfig::new(0x0308);
    let mut builder = KVTaggerBuilder::<MockPipe>::new(cfg);
    builder.set_database("tests/non_existing.csv").err().unwrap();
}

#[test]
fn test_exotic_csv_file_can_be_loaded() {
    SYSLOG_NG_INITIALIZED.call_once(|| {
        unsafe { syslog_ng_global_init() };
    });

    let expected = vec![
        ("key1".to_string(), "name1, name2".to_string(), "value1".to_string()),
        ("key1".to_string(), "name11".to_string(), "value11".to_string()),
    ];

    let cfg = GlobalConfig::new(0x0308);
    let mut builder = KVTaggerBuilder::<MockPipe>::new(cfg);
    builder.set_database("tests/exotic.csv").ok().unwrap();

    assert_eq!(builder.records(), Some(&expected));
}
