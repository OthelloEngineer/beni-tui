// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::fetchStructureResponse;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: fetchStructureResponse = serde_json::from_str(&json).unwrap();
// }

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchStructureResponse {
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
    pub discount_structure: DiscountStructure,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscountStructure {
    pub discount_small_logo: String,
    pub categories: Vec<Category>,
    pub support_mode: bool,
    pub distinct_number_of_discounts: i64,
    pub user: User,
    pub promoted_elements: Vec<PromotedElement>,
    pub beta_mode: bool,
    pub discounts_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub discounts: Vec<Discount>,
    pub sort_order: i64,
    pub create_user_activity_url: String,
    pub body_fragment: Option<String>,
    pub child_categories: Option<Vec<Category>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
    pub id: i64,
    pub name: String,
    pub description_highlight: String,
    pub description: String,
    pub sort_order: i64,
    pub global_sort_order: i64,
    pub start_date: String,
    pub usage_score: f64,
    pub icon_url: String,
    pub icon_retina_url: String,
    pub visible_keywords: Vec<VisibleKeyword>,
    pub end_date: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct VisibleKeyword {
    pub visible: bool,
    pub keyword: Keyword,
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub enum Color {
    #[serde(rename = "#ffffff")]
    Ffffff,
    #[serde(rename = "#3C3CFF")]
    The3C3Cff,
    #[serde(rename = "#3C8C33")]
    The3C8C33,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Keyword {
    Gavekort,
    #[serde(rename = "gælder i butik")]
    GlderIButik,
    #[serde(rename = "gælder på udsalg")]
    GlderPUdsalg,
    #[serde(rename = "kampagne!")]
    Kampagne,
    #[serde(rename = "midlertidigt tilbud!")]
    MidlertidigtTilbud,
    #[serde(rename = "nyhed!")]
    Nyhed,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotedElement {
    pub discount_id: i64,
    pub discount_category_id: i64,
    pub fragment: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub home_geo_position: HomeGeoPosition,
    pub home_address: String,
    pub company_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct HomeGeoPosition {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationPoint {
    pub url: String,
    pub name: String,
    pub title: String,
    pub company_id: i64,
}
