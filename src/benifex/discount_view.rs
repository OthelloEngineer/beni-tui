// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::userAppView;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: userAppView = serde_json::from_str(&json).unwrap();
// }

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscountView {
    pub function_data: FunctionData,
    pub empty: bool,
    pub messages: Option<serde_json::Value>,
    pub errors: Option<serde_json::Value>,
    pub navigation_point: NavigationPoint,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct FunctionData {
    pub result: Result,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub id: i64,
    pub name: String,
    pub description_highlight: String,
    pub description: String,
    pub description_long_html: String,
    pub sort_order: i64,
    pub global_sort_order: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub usage_score: f64,
    pub icon_url: String,
    pub icon_retina_url: String,
    pub visible_keywords: Vec<VisibleKeyword>,
}

#[derive(Serialize, Deserialize)]
pub struct VisibleKeyword {
    pub visible: bool,
    pub keyword: String,
    pub color: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationPoint {
    pub url: String,
    pub name: String,
    pub title: String,
    pub company_id: i64,
}
