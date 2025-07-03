use crate::texel::Tuner;
use crate::texel::{data_file, data_point};

impl Tuner {
    pub(in super::super::texel) fn print_data_file_read_result(
        &self,
        data_file_store: &data_file::Store,
    ) {
        println!(
            "Reading lines from: {}",
            self.data_file_name
                .clone()
                .into_os_string()
                .into_string()
                .unwrap_or_default()
        );
        println!("Lines read: {}", data_file_store.count_all());
        println!("Lines successful: {}", data_file_store.count_successful());

        if data_file_store.count_failed() > 0 {
            println!("Lines failed: {}", data_file_store.count_failed());
            for line in data_file_store.get_failed() {
                println!("\tLine number: {}", line.get_nr());
            }
        }
    }

    pub(in super::super::texel) fn print_data_point_conversion_result(
        &self,
        data_point_store: &data_point::Store,
    ) {
        const CONVERSIONS: &str = "Line to Data Point conversions";
        const SUCCESS: &str = "Line to Data Point success";
        const FAILURES: &str = "Line to Data Point failures";

        println!("{CONVERSIONS}: {}", data_point_store.count_all());
        println!("{SUCCESS}: {}", data_point_store.count_successful());

        if data_point_store.count_failed() > 0 {
            println!("{FAILURES}: {}", data_point_store.count_failed());
            for data in data_point_store.get_failed() {
                println!("\t{data}");
            }
        }
    }
}
