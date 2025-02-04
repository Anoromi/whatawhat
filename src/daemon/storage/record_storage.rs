use std::{collections::{BTreeMap, BTreeSet}, future::Future};

use anyhow::Result;
use chrono::NaiveDate;

use super::{
    application_storage::{UsageIntervalEntity, UsageRecordEntity},
    record_event::Color,
};

pub trait RecordStorage {
    type RecordFile: RecordFileHandle;

    fn compact_files(&self) -> impl Future<Output = Result<()>>;
    fn compact_file(
        &self,
        _record_file: Self::RecordFile,
    ) -> impl Future<Output = Result<()>> {
        async { self.compact_files().await }
    }
    fn create_or_append_record(&self, date: NaiveDate) -> impl Future<Output = Result<Self::RecordFile>>;

    fn get_data_for(&self, date: NaiveDate) -> impl Future<Output = Result<Vec<UsageIntervalEntity>>>;
}

pub trait IndexStorage {
    fn update_color_index(&self, process_name: String, color: Color) -> impl Future<Output = Result<()>>;

    fn get_colors_for(
        &self,
        names: BTreeSet<String>,
    ) -> impl Future<Output = Result<BTreeMap<String, Option<Color>>>>;
}

pub trait RecordFileHandle {
    fn append(&self, usage_record: UsageRecordEntity) -> impl Future<Output = Result<()>>;
    fn get_date(&self) -> NaiveDate;
}

pub trait ColorIndexStorage {
    fn add_element(&self, key: &str, value: Color) -> impl Future<Output = Result<()>>;


    fn finalize(&self) -> impl Future<Output = Result<()>> ;
    


}
