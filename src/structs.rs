use std::collections::HashMap;
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Question{
    pub answers: Vec<Answer>,
    pub exclusiontags: Vec<String>,
    pub buttontext: String,
    pub help: String,
    pub id: i32,
    pub important: bool,
    pub number: i32,
    pub singleanswer: bool,
    pub text: String,
    pub answered: bool
}

impl Question{
   pub  fn get_exclusiontags(&self,s: String) -> Vec<String> {
        let v: Vec<String> = serde_json::from_str(&s.to_owned()).unwrap();
        return v;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Answer{
    pub  id: i32,
    pub image: String,
    pub istext: bool,
    pub selected: bool,
    pub tags: Vec<String>,
    pub notags: Vec<String>,
    pub text: String
}

#[derive(Serialize, Deserialize)]
pub struct i18nValue{
    name: String,
    val: String
}

impl i18nValue {
    pub fn new(n: String, v: String) -> i18nValue {
        i18nValue { val: n.to_string(), name: v.to_string() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Stat{
    pub hits: i32,
    pub count: i32,
    pub MONTH: String,
    pub FullDate: String
}

#[derive(Serialize, Deserialize)]
pub struct Get{
    pub distros: Vec<Distro>,
    pub questions: Vec<Question>,
    pub i18n: HashMap<String,i18nValue>,
    pub visitor: i32
}


#[derive(Serialize, Deserialize)]
pub struct Rating{
    pub ID: i32,
    pub Rating: i32,
    pub UserAgent: String,
    pub Comment: String,
    pub Test: i32
}

/**
* Enums
*/
#[derive(Serialize, Deserialize)]
pub enum APIError {
    DistroNotFound
}
/**
* Types
*/
pub type APIResult = Result<Distro, APIError>;


/**
* Structs
*/
#[derive(Serialize, Deserialize)]
pub struct Distro {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub image: String,
    pub imagesource: String,
    pub textsource: String,
    pub colorcode: String,
    pub tags: Vec<String>
}

impl Distro{
   pub  fn get_tags(&self,s: String) -> Vec<String> {
        let v: Vec<String> = serde_json::from_str(&s.to_owned()).unwrap();
        return v;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Test{
    pub answers: Vec<i32>,
    pub important: Vec<i32>
}
impl Test{
   pub  fn get_tags(&self,s: String) -> Vec<i32> {
        let v: Vec<i32> = serde_json::from_str(&s.to_owned()).unwrap();
        return v;
    }
}